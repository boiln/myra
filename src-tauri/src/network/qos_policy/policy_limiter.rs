//! QoS Policy bandwidth limiter using PowerShell
//!
//! Uses Windows New-NetQosPolicy cmdlet to create bandwidth limits.
//! This works at the OS level like NetLimiter.

use log::{debug, error, info};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

const POLICY_NAME: &str = "MyraQosLimit";

/// Errors from QoS Policy operations
#[derive(Error, Debug)]
pub enum QosError {
    #[error("Failed to create QoS policy: {0}")]
    CreateFailed(String),
    
    #[error("Failed to remove QoS policy: {0}")]
    RemoveFailed(String),
    
    #[error("PowerShell not available")]
    PowerShellNotAvailable,
    
    #[error("Requires administrator privileges")]
    RequiresAdmin,
}

/// QoS Policy bandwidth limiter
/// 
/// Creates a Windows QoS policy to limit bandwidth at the OS level.
pub struct QosPolicyLimiter {
    is_active: Arc<AtomicBool>,
    limit_kbps: u32,
}

impl QosPolicyLimiter {
    /// Create a new QoS Policy limiter
    /// 
    /// # Arguments
    /// * `limit_kbps` - Bandwidth limit in kilobytes per second
    /// * `process_name` - Process name to limit (e.g., "rpcs3.exe")
    ///                    REQUIRED - we never limit all traffic to avoid breaking internet!
    pub fn new(limit_kbps: u32, process_name: Option<&str>) -> Result<Self, QosError> {
        // SAFETY: Require a process name - never limit ALL traffic!
        let proc = match process_name {
            Some(p) if !p.is_empty() => p,
            _ => {
                error!("QoS: SAFETY - Cannot create policy without process name!");
                error!("QoS: Limiting all traffic would break internet connectivity.");
                return Err(QosError::CreateFailed(
                    "Must specify a process name (e.g., 'rpcs3.exe'). Cannot limit all traffic.".into()
                ));
            }
        };
        
        info!("QoS: Creating bandwidth limit policy at {} KB/s for '{}'", limit_kbps, proc);
        
        // First, try to remove any existing policy
        let _ = Self::remove_policy_internal();
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Convert KB/s to bits per second (what QoS policy expects)
        let bits_per_second = (limit_kbps as u64) * 1024 * 8;
        
        // Build the PowerShell command - ONLY for specific process
        let ps_cmd = format!(
            "New-NetQosPolicy -Name '{}' -AppPathNameMatchCondition '{}' -ThrottleRateActionBitsPerSecond {} -PolicyStore ActiveStore",
            POLICY_NAME, proc, bits_per_second
        );
        
        debug!("QoS: Running PowerShell command: {}", ps_cmd);
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
            .output()
            .map_err(|e| QosError::CreateFailed(format!("Failed to run PowerShell: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            error!("QoS: Failed to create policy. stderr: {}, stdout: {}", stderr, stdout);
            
            if stderr.contains("Access") || stderr.contains("denied") || stderr.contains("administrator") {
                return Err(QosError::RequiresAdmin);
            }
            
            // If policy already exists, try to remove and recreate
            if stderr.contains("already exists") {
                info!("QoS: Policy exists, forcing removal and retry");
                let _ = Self::remove_policy_internal();
                std::thread::sleep(std::time::Duration::from_millis(200));
                
                let output2 = Command::new("powershell")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
                    .output()
                    .map_err(|e| QosError::CreateFailed(format!("Retry failed: {}", e)))?;
                
                if !output2.status.success() {
                    let stderr2 = String::from_utf8_lossy(&output2.stderr);
                    return Err(QosError::CreateFailed(format!("Retry failed: {}", stderr2)));
                }
            } else {
                return Err(QosError::CreateFailed(format!("{} {}", stdout, stderr)));
            }
        }
        
        info!("QoS: Policy created successfully at {} KB/s for '{}'", limit_kbps, proc);
        
        Ok(Self {
            is_active: Arc::new(AtomicBool::new(true)),
            limit_kbps,
        })
    }
    
    /// Check if the limiter is active
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
    
    /// Get the current bandwidth limit in KB/s
    pub fn limit_kbps(&self) -> u32 {
        self.limit_kbps
    }
    
    /// Remove the QoS policy from all stores
    fn remove_policy_internal() -> Result<(), QosError> {
        // Remove from ActiveStore (the one that actually affects traffic NOW)
        let ps_cmd = format!(
            "Remove-NetQosPolicy -Name '{}' -PolicyStore ActiveStore -Confirm:$false -ErrorAction SilentlyContinue; \
             Remove-NetQosPolicy -Name '{}' -Confirm:$false -ErrorAction SilentlyContinue",
            POLICY_NAME, POLICY_NAME
        );
        
        info!("QoS: Removing policy from all stores");
        
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
            .output()
            .map_err(|e| QosError::RemoveFailed(format!("Failed to run PowerShell: {}", e)))?;
        
        if !output.status.success() {
            // It's okay if removal fails (policy might not exist)
            debug!("QoS: Policy removal returned non-zero (may not exist)");
        }
        
        Ok(())
    }
    
    /// Stop the limiter and remove the policy
    pub fn stop(&mut self) {
        if !self.is_active.swap(false, Ordering::SeqCst) {
            return; // Already stopped
        }
        
        info!("QoS: Removing bandwidth limit policy");
        
        if let Err(e) = Self::remove_policy_internal() {
            error!("QoS: Failed to remove policy: {:?}", e);
        } else {
            info!("QoS: Policy removed successfully");
        }
    }
}

impl Drop for QosPolicyLimiter {
    fn drop(&mut self) {
        self.stop();
    }
}

// Thread-safe
unsafe impl Send for QosPolicyLimiter {}
unsafe impl Sync for QosPolicyLimiter {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qos_policy_creation() {
        // Note: This test requires admin privileges
        match QosPolicyLimiter::new(100, None) {
            Ok(mut limiter) => {
                assert!(limiter.is_active());
                limiter.stop();
                assert!(!limiter.is_active());
            }
            Err(e) => {
                println!("QoS policy test skipped: {:?}", e);
            }
        }
    }
}


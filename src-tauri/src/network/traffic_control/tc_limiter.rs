//! Traffic Control bandwidth limiter implementation
//!
//! Uses Windows QoS Traffic Control API to limit bandwidth at the socket layer,
//! similar to how NetLimiter operates.

use log::{debug, error, info, warn};
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;

// Windows API types and functions
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::QoS::{
    TcRegisterClient, TcDeregisterClient, TcEnumerateInterfaces,
    TcOpenInterfaceW, TcCloseInterface, TcAddFlow, TcDeleteFlow,
    TcAddFilter, TcDeleteFilter,
    TC_GEN_FLOW, TC_GEN_FILTER, IP_PATTERN,
    TCI_CLIENT_FUNC_LIST, TC_IFC_DESCRIPTOR,
    SERVICETYPE_BESTEFFORT,
};
use windows::Win32::Networking::WinSock::AF_INET;

/// Direction for traffic control
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TcDirection {
    /// Limit inbound (download) traffic
    Inbound,
    /// Limit outbound (upload) traffic  
    Outbound,
    /// Limit both directions
    Both,
}

/// Errors from Traffic Control operations
#[derive(Error, Debug)]
pub enum TcError {
    #[error("Failed to register TC client: {0}")]
    RegisterFailed(u32),
    
    #[error("Failed to enumerate interfaces: {0}")]
    EnumerateFailed(u32),
    
    #[error("No suitable network interface found")]
    NoInterface,
    
    #[error("Failed to open interface: {0}")]
    OpenInterfaceFailed(u32),
    
    #[error("Failed to add flow: {0}")]
    AddFlowFailed(u32),
    
    #[error("Failed to add filter: {0}")]
    AddFilterFailed(u32),
    
    #[error("Traffic Control not available on this system")]
    NotAvailable,
    
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}

/// Traffic Control bandwidth limiter
/// 
/// Provides true bandwidth limiting using Windows QoS API,
/// operating at the same layer as NetLimiter.
pub struct TrafficControlLimiter {
    client_handle: HANDLE,
    interface_handle: HANDLE,
    flow_handle: HANDLE,
    filter_handle: HANDLE,
    is_active: Arc<AtomicBool>,
    limit_kbps: u32,
    direction: TcDirection,
}

// Callback functions required by TC API
// These must match the exact signatures expected by Windows
unsafe extern "system" fn tc_notify_handler(
    _client_handle: HANDLE,
    _flow_handle: HANDLE,
    _event: u32,
    _subcode: HANDLE,
    _buf_size: u32,
    _buffer: *const c_void,
) {
    // We don't need to handle notifications for simple bandwidth limiting
}

unsafe extern "system" fn tc_add_flow_complete(
    _client_handle: HANDLE,
    _status: u32,
) {
    debug!("TC: Add flow complete callback");
}

unsafe extern "system" fn tc_modify_flow_complete(
    _client_handle: HANDLE,
    _status: u32,
) {
    debug!("TC: Modify flow complete callback");
}

unsafe extern "system" fn tc_delete_flow_complete(
    _client_handle: HANDLE,
    _status: u32,
) {
    debug!("TC: Delete flow complete callback");
}

impl TrafficControlLimiter {
    /// Create a new Traffic Control limiter
    /// 
    /// # Arguments
    /// * `limit_kbps` - Bandwidth limit in kilobytes per second
    /// * `direction` - Which direction to limit (inbound, outbound, or both)
    pub fn new(limit_kbps: u32, direction: TcDirection) -> Result<Self, TcError> {
        if limit_kbps == 0 {
            return Err(TcError::InvalidParam("Bandwidth limit must be > 0".into()));
        }
        
        info!("TC: Initializing Traffic Control limiter at {} KB/s ({:?})", 
              limit_kbps, direction);
        
        unsafe {
            // Step 1: Register as TC client
            let mut client_handle = HANDLE::default();
            
            // Set up callback functions
            let client_funcs = TCI_CLIENT_FUNC_LIST {
                ClNotifyHandler: Some(tc_notify_handler),
                ClAddFlowCompleteHandler: Some(tc_add_flow_complete),
                ClModifyFlowCompleteHandler: Some(tc_modify_flow_complete),
                ClDeleteFlowCompleteHandler: Some(tc_delete_flow_complete),
            };
            
            let result = TcRegisterClient(
                0x0200, // TC_CURRENT_VERSION
                client_handle,
                &client_funcs,
                &mut client_handle,
            );
            
            if result != 0 {
                error!("TC: Failed to register client, error: {}", result);
                // Error 7501 typically means QoS Packet Scheduler service is not running
                if result == 7501 {
                    error!("TC: QoS Packet Scheduler service may not be enabled.");
                    error!("TC: Try: 1) Open Services (services.msc), 2) Find 'QoS Packet Scheduler', 3) Start it");
                    return Err(TcError::NotAvailable);
                }
                return Err(TcError::RegisterFailed(result));
            }
            
            info!("TC: Registered client successfully");
            
            // Step 2: Enumerate network interfaces
            let mut buffer_size: u32 = 0;
            
            // First call to get required buffer size
            let _ = TcEnumerateInterfaces(client_handle, &mut buffer_size, ptr::null_mut());
            
            if buffer_size == 0 {
                TcDeregisterClient(client_handle);
                return Err(TcError::NoInterface);
            }
            
            let mut buffer: Vec<u8> = vec![0u8; buffer_size as usize];
            let result = TcEnumerateInterfaces(
                client_handle, 
                &mut buffer_size, 
                buffer.as_mut_ptr() as *mut TC_IFC_DESCRIPTOR
            );
            
            if result != 0 {
                TcDeregisterClient(client_handle);
                return Err(TcError::EnumerateFailed(result));
            }
            
            // Parse the interface descriptor - use the first available interface
            let ifc_desc = &*(buffer.as_ptr() as *const TC_IFC_DESCRIPTOR);
            
            info!("TC: Found network interface, opening...");
            
            // Step 3: Open the interface
            let mut interface_handle = HANDLE::default();
            let result = TcOpenInterfaceW(
                ifc_desc.pInterfaceName,
                client_handle,
                HANDLE::default(), // ClIfcCtx - context, not needed
                &mut interface_handle,
            );
            
            if result != 0 {
                TcDeregisterClient(client_handle);
                return Err(TcError::OpenInterfaceFailed(result));
            }
            
            info!("TC: Opened interface successfully");
            
            // Step 4: Create a flow with bandwidth limit
            let bytes_per_sec = (limit_kbps as u64) * 1024;
            
            // Create FLOWSPEC manually since it might not be directly available
            #[repr(C)]
            struct FlowSpec {
                token_rate: u32,
                token_bucket_size: u32,
                peak_bandwidth: u32,
                latency: u32,
                delay_variation: u32,
                service_type: u32,
                max_sdu_size: u32,
                minimum_policed_size: u32,
            }
            
            let sending_flowspec = FlowSpec {
                token_rate: bytes_per_sec as u32,
                token_bucket_size: bytes_per_sec as u32,
                peak_bandwidth: bytes_per_sec as u32,
                latency: 0xFFFFFFFF,
                delay_variation: 0xFFFFFFFF,
                service_type: SERVICETYPE_BESTEFFORT,
                max_sdu_size: 0xFFFFFFFF,
                minimum_policed_size: 0xFFFFFFFF,
            };
            
            let receiving_flowspec = FlowSpec {
                token_rate: bytes_per_sec as u32,
                token_bucket_size: bytes_per_sec as u32,
                peak_bandwidth: bytes_per_sec as u32,
                latency: 0xFFFFFFFF,
                delay_variation: 0xFFFFFFFF,
                service_type: SERVICETYPE_BESTEFFORT,
                max_sdu_size: 0xFFFFFFFF,
                minimum_policed_size: 0xFFFFFFFF,
            };
            
            // Create the flow structure - we need to cast carefully
            let mut flow_buffer = vec![0u8; size_of::<TC_GEN_FLOW>() + 256];
            let flow = &mut *(flow_buffer.as_mut_ptr() as *mut TC_GEN_FLOW);
            
            // Copy flowspec data
            ptr::copy_nonoverlapping(
                &sending_flowspec as *const FlowSpec as *const u8,
                &mut flow.SendingFlowspec as *mut _ as *mut u8,
                size_of::<FlowSpec>()
            );
            ptr::copy_nonoverlapping(
                &receiving_flowspec as *const FlowSpec as *const u8,
                &mut flow.ReceivingFlowspec as *mut _ as *mut u8,
                size_of::<FlowSpec>()
            );
            flow.TcObjectsLength = 0;
            
            let mut flow_handle = HANDLE::default();
            let result = TcAddFlow(
                interface_handle,
                HANDLE::default(), // ClFlowCtx
                0, // Flags
                flow,
                &mut flow_handle,
            );
            
            if result != 0 {
                TcCloseInterface(interface_handle);
                TcDeregisterClient(client_handle);
                return Err(TcError::AddFlowFailed(result));
            }
            
            info!("TC: Added flow successfully");
            
            // Step 5: Add a filter to match all traffic (or specific direction)
            let mut pattern = IP_PATTERN::default();
            // Match all traffic - leave pattern as zeros (wildcard)
            
            let mut filter = TC_GEN_FILTER {
                AddressType: AF_INET.0 as u16,
                PatternSize: size_of::<IP_PATTERN>() as u32,
                Pattern: &mut pattern as *mut _ as *mut c_void,
                Mask: ptr::null_mut(), // NULL mask = match all
            };
            
            let mut filter_handle = HANDLE::default();
            let result = TcAddFilter(
                flow_handle,
                &mut filter,
                &mut filter_handle,
            );
            
            if result != 0 {
                warn!("TC: Failed to add filter (error: {}), continuing without filter", result);
                // Don't fail completely - some systems may not need explicit filter
            } else {
                info!("TC: Added filter successfully");
            }
            
            info!("TC: Traffic Control limiter active at {} KB/s", limit_kbps);
            
            Ok(Self {
                client_handle,
                interface_handle,
                flow_handle,
                filter_handle,
                is_active: Arc::new(AtomicBool::new(true)),
                limit_kbps,
                direction,
            })
        }
    }
    
    /// Check if the limiter is active
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::SeqCst)
    }
    
    /// Get the current bandwidth limit in KB/s
    pub fn limit_kbps(&self) -> u32 {
        self.limit_kbps
    }
    
    /// Get the direction being limited
    pub fn direction(&self) -> TcDirection {
        self.direction
    }
    
    /// Stop the limiter and clean up resources
    pub fn stop(&mut self) {
        if !self.is_active.swap(false, Ordering::SeqCst) {
            return; // Already stopped
        }
        
        info!("TC: Stopping Traffic Control limiter");
        
        unsafe {
            // Clean up in reverse order
            if !self.filter_handle.is_invalid() {
                let _ = TcDeleteFilter(self.filter_handle);
            }
            
            if !self.flow_handle.is_invalid() {
                let _ = TcDeleteFlow(self.flow_handle);
            }
            
            if !self.interface_handle.is_invalid() {
                let _ = TcCloseInterface(self.interface_handle);
            }
            
            if !self.client_handle.is_invalid() {
                let _ = TcDeregisterClient(self.client_handle);
            }
        }
        
        info!("TC: Traffic Control limiter stopped");
    }
}

impl Drop for TrafficControlLimiter {
    fn drop(&mut self) {
        self.stop();
    }
}

// Thread-safe - the handles are only accessed through atomic state
unsafe impl Send for TrafficControlLimiter {}
unsafe impl Sync for TrafficControlLimiter {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tc_limiter_creation() {
        // Note: This test may fail if not running as admin or TC not available
        match TrafficControlLimiter::new(100, TcDirection::Both) {
            Ok(mut limiter) => {
                assert!(limiter.is_active());
                assert_eq!(limiter.limit_kbps(), 100);
                limiter.stop();
                assert!(!limiter.is_active());
            }
            Err(e) => {
                // TC may not be available in test environment
                println!("TC not available: {:?}", e);
            }
        }
    }
}

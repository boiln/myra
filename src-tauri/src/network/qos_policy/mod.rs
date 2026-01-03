//! Windows QoS Policy for bandwidth limiting
//! 
//! Uses PowerShell New-NetQosPolicy to create OS-level bandwidth limits,
//! similar to how NetLimiter operates.

mod policy_limiter;

pub use policy_limiter::{QosPolicyLimiter, QosError};



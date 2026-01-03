//! WFP-based inbound throttling
//! 
//! Uses Windows Filtering Platform to intercept and throttle inbound traffic.
//! This is a user-mode approach that may have limitations compared to kernel drivers.

mod throttle;

pub use throttle::{WfpThrottle, WfpError};



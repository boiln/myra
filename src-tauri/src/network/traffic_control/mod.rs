//! Windows Traffic Control API for true bandwidth limiting
//! 
//! This module provides NetLimiter-style bandwidth limiting using
//! the Windows QoS Traffic Control API. Unlike WinDivert which
//! intercepts at the packet level, TC operates at the socket layer,
//! allowing proper TCP/UDP stack operation while limiting bandwidth.

mod tc_limiter;

pub use tc_limiter::{TrafficControlLimiter, TcError, TcDirection};



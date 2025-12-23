//! # Myra - A network condition simulation tool
//!
//! Myra is a tool for simulating poor network conditions by manipulating
//! network packets using WinDivert on Windows systems.
//!
//! ## Features
//!
//! * Packet dropping - Randomly drop packets to simulate packet loss
//! * Packet delay - Add latency to packets
//! * Network throttling - Slow down connections temporarily
//! * Packet reordering - Change the order packets arrive in
//! * Packet tampering - Corrupt packet data
//! * Packet duplication - Send multiple copies of packets
//! * Bandwidth limiting - Restrict connection speeds
//!
//! ## Architecture
//!
//! Myra is built using a Tauri 2.0 app structure with:
//! * Rust backend: Handles packet interception and manipulation
//! * TypeScript frontend: Provides a user-friendly interface
//!
//! The core functionality relies on WinDivert to intercept and inject
//! network packets on Windows systems.

/// Commands exposed to the Tauri frontend
pub mod commands;
/// Network packet manipulation functionality
pub mod network;
/// Configuration settings for packet manipulation
pub mod settings;
/// Shared utility functions
pub mod utils;

pub mod bandwidth;
pub mod burst;
pub mod lag;
pub mod drop;
pub mod duplicate;
pub mod reorder;
pub mod stats;
pub mod tamper;
pub mod throttle;
pub mod traits;

// Re-export module structs for convenience
pub use bandwidth::BandwidthModule;
pub use burst::BurstModule;
pub use lag::LagModule;
pub use drop::DropModule;
pub use duplicate::DuplicateModule;
pub use reorder::ReorderModule;
pub use tamper::TamperModule;
pub use throttle::ThrottleModule;
pub use traits::{ModuleContext, ModuleOptions, PacketModule};

use serde::Serialize;

/// Statistics for the burst module
#[derive(Debug, Serialize, Clone)]
pub struct BurstStats {
    /// Total packets buffered since last report
    pub buffered: usize,
    /// Total packets released in last burst
    pub released: usize,
    /// Current buffer size
    pub buffered_count: usize,
}

impl BurstStats {
    pub fn new(_ema_factor: f64) -> Self {
        Self {
            buffered: 0,
            released: 0,
            buffered_count: 0,
        }
    }

    pub fn record_buffer(&mut self, count: usize) {
        self.buffered += count;
    }

    pub fn record_release(&mut self, count: usize) {
        self.released = count;
    }

    pub fn set_buffered_count(&mut self, count: usize) {
        self.buffered_count = count;
    }

    pub fn reset_periodic(&mut self) {
        self.buffered = 0;
        self.released = 0;
    }
}

impl Default for BurstStats {
    fn default() -> Self {
        Self::new(0.05)
    }
}

//! Classic mode state management.
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::settings::classic::ClassicSettings;

/// Global state for Classic mode packet processing.
pub struct ClassicProcessingState {

    /// Flag indicating whether Classic mode processing is active
    pub running: Arc<AtomicBool>,
    /// Current Classic mode settings
    pub settings: Arc<Mutex<ClassicSettings>>,

}

impl Default for ClassicProcessingState {

    fn default() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            settings: Arc::new(Mutex::new(ClassicSettings::default())),
        }
    }

}

impl ClassicProcessingState {

    pub fn new() -> Self {
        Self::default()
    }

}

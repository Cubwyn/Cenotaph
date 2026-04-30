// src/engine/debug.rs
// Runtime debug logging and error reporting.
#![allow(dead_code)]

pub struct DebugManager {
    pub enabled: bool,
}

impl DebugManager {
    pub fn new() -> Self {
        Self { enabled: true }
    }

    pub fn log(&self, message: &str) {
        if self.enabled {
            println!("[DEBUG] {}", message);
        }
    }

    pub fn error(&self, message: &str) {
        if self.enabled {
            eprintln!("[ERROR] {}", message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_manager_enabled() {
        let debug = DebugManager::new();
        assert!(debug.enabled);
        debug.log("Test log message");
        debug.error("Test error message");
    }

    #[test]
    fn test_debug_manager_disabled() {
        let debug = DebugManager { enabled: false };
        // Should not panic even when disabled
        debug.log("Silent log");
        debug.error("Silent error");
    }
}

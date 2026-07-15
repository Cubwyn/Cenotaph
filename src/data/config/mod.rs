// src/config/mod.rs
// Configuration loading and key-binding resolution.

pub mod gameplay;
pub mod ui;
pub mod visuals;

// Note: GameConfig is no longer re-exported here as it's now used directly
// from the gameplay module where it belongs

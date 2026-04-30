// src/audio/mod.rs
// Audio system for atmospheric sound design and combat feedback.

pub mod manager;
pub mod strata;
pub mod combat;

pub use manager::AudioManager;
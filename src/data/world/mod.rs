// src/world/mod.rs
// The world module owns all level data, prop definitions, and level logic.
// Think of this as the "map" of Cenotaph — the dead kingdom's geometry and memory.

pub mod level;

// Re-export the types most commonly needed by other modules.
#[allow(unused_imports)]
pub use level::{ColliderType, LevelData, PropData};

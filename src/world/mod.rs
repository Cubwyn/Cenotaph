// src/world/mod.rs
// The world module owns all level data, prop definitions, strata, and spawn logic.
// Think of this as the "map" of Cenotaph — the dead kingdom's geometry and memory.

pub mod level;
pub mod spawn;
pub mod strata;

// Note: Removed module declarations for bells, shortcuts, and hazards
// as these files were removed during cleanup

// Re-export the types most commonly needed by other modules.
#[allow(unused_imports)]
pub use level::{ColliderType, LevelData, PropData};

//! Gameplay-owned state and rules.
//!
//! Engine modules orchestrate the frame; this module owns reusable gameplay
//! foundations such as combat math, enemy rules, health, stamina, death, and
//! respawn state.

pub mod combat;
pub mod enemy;
pub mod player;
pub mod progression;

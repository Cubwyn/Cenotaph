//! Gameplay-owned state and rules.
//!
//! Engine modules orchestrate the frame; this module owns reusable gameplay
//! foundations such as combat math, enemy rules, health, stamina, death, and
//! respawn state.

pub mod combat;
pub mod cycle;
pub mod enemy;
pub mod feedback;
pub mod mountain;
pub mod player;
pub mod progression;
pub mod relic;
pub mod save;

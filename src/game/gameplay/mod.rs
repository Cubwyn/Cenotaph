// src/gameplay/mod.rs
// The gameplay module owns all game systems: player state, combat, and the
// Resonance system (the Bell mechanic that unlocks forgotten areas).

pub mod stamina;
pub mod player;
pub mod movement;

// Note: Weapon, StaminaSystem and MovementManager were originally exported here

// Note: Removed unused exports: PlayerStats, ResonanceArt, ArtType, 
// GameFactory, WeaponBuilder, PlayerActions, WeaponActions, WeaponCondition

// src/world/spawn.rs
// Spawn points and level-transition triggers.
// Kept separate from LevelData so the world module stays composable.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// A named position in the world where the player (or an entity) can be placed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SpawnPoint {
    pub id: String,
    pub position: [f32; 3],
    /// Optional: which stratum this spawn belongs to.
    pub stratum_id: Option<String>,
}

impl SpawnPoint {
    pub fn new(id: impl Into<String>, position: [f32; 3]) -> Self {
        Self {
            id: id.into(),
            position,
            stratum_id: None,
        }
    }

    pub fn in_stratum(mut self, stratum_id: impl Into<String>) -> Self {
        self.stratum_id = Some(stratum_id.into());
        self
    }
}

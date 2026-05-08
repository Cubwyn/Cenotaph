// src/world/level.rs
// LevelData is the serialisable description of a single playable area.
// PropData describes every object placed inside that area.
// ColliderType drives the physics engine's collider selection.
//
// This module defines the data structures used to represent levels and their contents.
// Levels are serialized to JSON for persistence and can be loaded dynamically at runtime.
// The system supports both static level geometry and dynamic props with various gameplay properties.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// ── Collider shape tag ────────────────────────────────────────────────────────

/// Defines the collision shape type for props and objects in the level.
/// 
/// This enum determines how the physics engine creates collision shapes for objects.
/// Different shapes have different performance characteristics and use cases.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum ColliderType {
    /// No collision shape - object is purely decorative
    None,
    /// Box collider - fast collision detection, good for rectangular objects
    Box,
    /// Sphere collider - fast collision detection, good for round objects
    Sphere,
    /// Mesh collider - precise collision matching the object's geometry, slower performance
    Mesh,
}

// -- Prop -------------------------------------------------------------------

/// A single object placed inside a level — geometry, physics, gameplay metadata.
/// 
/// Props represent all interactive and decorative objects in a level, including
/// enemies, items, environmental objects, and gameplay triggers. Each prop has
/// associated geometry, collision properties, and gameplay metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropData {
    /// Filename of the mesh asset (relative to `assets/`).
    /// Example: "Cube.obj" or "models/enemy.glb"
    pub asset_id: String,
    /// World position of the prop [x, y, z]
    pub position: [f32; 3],
    /// Rotation in degrees [x, y, z] around each axis
    pub rotation: [f32; 3],
    /// Scale factor [x, y, z] for each axis
    pub scale: [f32; 3],
    /// Collision shape type for physics interactions
    pub collider_type: ColliderType,
    /// Whether the player can climb on this object
    pub is_climbable: bool,
    /// Whether this prop acts as a hurtbox (damage source)
    pub is_hurtbox: bool,
    /// Item ID if this prop contains an item to be collected
    pub item_id: Option<String>,
    /// Enemy type if this prop represents an enemy
    pub enemy_type: Option<String>,
    /// Health points for enemies (ignored for non-enemies)
    pub enemy_health: f32,
    /// Light color as RGB values [r, g, b] if this prop emits light
    pub light_color: Option<[f32; 3]>,
    /// Light intensity/brightness value
    pub light_intensity: f32,
    /// Ambient sound ID to play near this prop
    pub ambient_sound_id: Option<String>,
    /// If set, touching this prop triggers a level transition to the specified level
    pub trigger_level_id: Option<String>,
}

//-- Level ------------------------------------------------------------

/// The full data description of a playable level / stratum.
/// 
/// A LevelData contains all the information needed to load and render a complete
/// game level, including the base geometry, props, and atmospheric settings.
/// Levels are serialized to JSON for persistence and can be loaded dynamically.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelData {
    /// Human-readable name of the level for display purposes
    pub name: String,
    /// Path to the base map mesh (e.g. `"assets/map_001.glb"`).
    /// This defines the main geometry of the level that the player walks on.
    pub base_map: String,
    /// Default spawn position for the player [x, y, z]
    pub player_spawn: [f32; 3],
    /// List of all props placed in this level
    pub props: Vec<PropData>,
}

impl LevelData {
    /// Returns the default level used when no save file exists.
    /// 
    /// This provides a fallback level with basic settings that allows the game
    /// to start even if no level files are present. It's also useful for testing
    /// and development purposes.
    pub fn default_level() -> Self {
        Self {
            name: "map_001".to_string(),
            base_map: "assets/map_001.glb".to_string(),
            player_spawn: [0.0, 10.0, 0.0],
            props: Vec::new(),
        }
    }

    /// Loads a level from a JSON file, falling back to the default if missing.
    /// 
    /// # Parameters
    /// - `file_path`: Path to the JSON file containing level data
    /// 
    /// # Returns
    /// A LevelData instance loaded from file, or the default level if loading fails
    /// 
    /// # Error Handling
    /// This function handles file not found, JSON parsing errors, and file read errors
    /// gracefully by falling back to the default level and printing warning messages.
    /// This ensures the game can continue running even with corrupted or missing level files.
    pub fn load(file_path: &str) -> Self {
        if Path::new(file_path).exists() {
            match fs::read_to_string(file_path) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(level_data) => {
                        println!("[DEBUG] Successfully loaded level: {}", file_path);
                        level_data
                    },
                    Err(e) => {
                        eprintln!("Error: Failed to parse level JSON at {}: {}, falling back to default", file_path, e);
                        Self::default_level()
                    }
                },
                Err(e) => {
                    eprintln!("Error: Failed to read level file at {}: {}, falling back to default", file_path, e);
                    Self::default_level()
                }
            }
        } else {
            eprintln!("Error: Level file not found at {}", file_path);
            Self::default_level()
        }
    }

    /// Serialises the level to a pretty-printed JSON file.
    /// 
    /// # Parameters
    /// - `file_path`: Path where the JSON file should be saved
    /// 
    /// # Returns
    /// - `Ok(())` on successful save
    /// - `Err(String)` with error message if serialization or file writing fails
    /// 
    /// # Usage
    /// This is used for saving level edits, creating backup files, or exporting
    /// levels for sharing or version control.
    pub fn save(&self, file_path: &str) -> Result<(), String> {
        let data = match serde_json::to_string_pretty(self) {
            Ok(data) => data,
            Err(e) => return Err(format!("Failed to serialize level: {}", e)),
        };
        match fs::write(file_path, data) {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("Failed to write level file: {}", e)),
        }
    }
}

// --- Tests ------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_level_has_correct_name() {
        let level = LevelData::default_level();
        assert_eq!(level.name, "map_001");
    }

    #[test]
    fn default_level_starts_empty() {
        let level = LevelData::default_level();
        assert!(level.props.is_empty());
    }
}
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
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum ColliderType {
    /// No collision shape - object is purely decorative
    #[default]
    None,
    /// Box collider - fast collision detection, good for rectangular objects
    Box,
    /// Sphere collider - fast collision detection, good for round objects
    Sphere,
    /// Mesh collider - precise collision matching the object's geometry, slower performance
    Mesh,
}

fn zero_vec3() -> [f32; 3] {
    [0.0, 0.0, 0.0]
}

fn unit_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
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
    #[serde(default = "zero_vec3")]
    pub position: [f32; 3],
    /// Rotation in degrees [x, y, z] around each axis
    #[serde(default = "zero_vec3")]
    pub rotation: [f32; 3],
    /// Scale factor [x, y, z] for each axis
    #[serde(default = "unit_scale")]
    pub scale: [f32; 3],
    /// Collision shape type for physics interactions
    #[serde(default)]
    pub collider_type: ColliderType,
    /// Whether the player can climb on this object
    #[serde(default)]
    pub is_climbable: bool,
    /// Whether this prop acts as a hurtbox (damage source)
    #[serde(default)]
    pub is_hurtbox: bool,
    /// Item ID if this prop contains an item to be collected
    #[serde(default)]
    pub item_id: Option<String>,
    /// Amount of unsecured resource granted when collected.
    #[serde(default)]
    pub resource_value: u32,
    /// Anchor ID if this prop banks resources and sets a local respawn point.
    #[serde(default)]
    pub anchor_id: Option<String>,
    /// Enemy type if this prop represents an enemy
    #[serde(default)]
    pub enemy_type: Option<String>,
    /// Health points for enemies (ignored for non-enemies)
    #[serde(default)]
    pub enemy_health: f32,
    /// Light color as RGB values [r, g, b] if this prop emits light
    #[serde(default)]
    pub light_color: Option<[f32; 3]>,
    /// Light intensity/brightness value
    #[serde(default)]
    pub light_intensity: f32,
    /// Ambient sound ID to play near this prop
    #[serde(default)]
    pub ambient_sound_id: Option<String>,
    /// If set, touching this prop triggers a level transition to the specified level
    #[serde(default)]
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
        match Self::try_load(file_path) {
            Ok(level_data) => {
                println!("[DEBUG] Successfully loaded level: {}", file_path);
                level_data
            }
            Err(e) => {
                eprintln!("Error: {}, falling back to default", e);
                Self::default_level()
            }
        }
    }

    /// Loads a level from disk without falling back.
    ///
    /// Use this for validation and tooling, where broken content should fail
    /// loudly instead of being replaced by a default level.
    pub fn try_load(file_path: &str) -> Result<Self, String> {
        if !Path::new(file_path).exists() {
            return Err(format!("level file not found at {}", file_path));
        }

        let data = fs::read_to_string(file_path)
            .map_err(|e| format!("failed to read level file at {}: {}", file_path, e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("failed to parse level JSON at {}: {}", file_path, e))
    }

    /// Returns content-authoring errors that can break runtime assumptions.
    ///
    /// This does not try to prove the level is fun. It only checks the stable
    /// foundation contract: finite transforms, usable scales, sensible enemy
    /// values, and references that can be resolved from disk.
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("level name must not be empty".to_string());
        }
        if self.base_map.trim().is_empty() {
            errors.push("base_map must not be empty".to_string());
        } else if !Path::new(&self.base_map).exists() {
            errors.push(format!("base_map '{}' does not exist", self.base_map));
        }
        if !self.player_spawn.iter().all(|v| v.is_finite()) {
            errors.push("player_spawn must contain finite numbers".to_string());
        }

        for (index, prop) in self.props.iter().enumerate() {
            errors.extend(prop.validation_errors(index));
        }

        errors
    }

    pub fn validate(&self) -> Result<(), Vec<String>> {
        let errors = self.validation_errors();
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
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
    #[allow(dead_code)]
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

impl PropData {
    pub fn rotation_radians(&self) -> [f32; 3] {
        [
            self.rotation[0].to_radians(),
            self.rotation[1].to_radians(),
            self.rotation[2].to_radians(),
        ]
    }

    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("prop {} ('{}')", index, self.asset_id);

        if self.asset_id.trim().is_empty() {
            errors.push(format!("{} asset_id must not be empty", label));
        } else {
            let asset_path = format!("assets/{}", self.asset_id);
            if !Path::new(&asset_path).exists() {
                errors.push(format!(
                    "{} references missing asset '{}'",
                    label, asset_path
                ));
            }
        }

        if !self.position.iter().all(|v| v.is_finite()) {
            errors.push(format!("{} position must contain finite numbers", label));
        }
        if !self.rotation.iter().all(|v| v.is_finite()) {
            errors.push(format!("{} rotation must contain finite numbers", label));
        }
        if !self.scale.iter().all(|v| v.is_finite()) {
            errors.push(format!("{} scale must contain finite numbers", label));
        }
        if self.scale.iter().any(|v| v.abs() <= f32::EPSILON) {
            errors.push(format!("{} scale must not contain zero values", label));
        }
        if self
            .enemy_type
            .as_ref()
            .is_some_and(|enemy_type| enemy_type.trim().is_empty())
        {
            errors.push(format!("{} enemy_type must not be empty", label));
        }
        if self
            .anchor_id
            .as_ref()
            .is_some_and(|anchor_id| anchor_id.trim().is_empty())
        {
            errors.push(format!("{} anchor_id must not be empty", label));
        }
        if self.resource_value > 0 && self.enemy_type.is_some() {
            errors.push(format!(
                "{} cannot be both a resource pickup and an enemy",
                label
            ));
        }
        if self.enemy_type.is_some() && self.enemy_health < 0.0 {
            errors.push(format!("{} enemy_health must not be negative", label));
        }
        if let Some(target) = self.trigger_level_id.as_ref() {
            let target_path = format!("levels/{}.json", target);
            if !Path::new(&target_path).exists() {
                errors.push(format!(
                    "{} trigger_level_id references missing level '{}'",
                    label, target_path
                ));
            }
        }
        if self.light_color.is_none() && self.light_intensity > 0.0 {
            errors.push(format!(
                "{} light_intensity is set without a light_color",
                label
            ));
        }

        errors
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

    #[test]
    fn minimal_prop_json_uses_foundation_defaults() {
        let prop: PropData = serde_json::from_str(r#"{ "asset_id": "Cube.obj" }"#).unwrap();
        assert_eq!(prop.position, [0.0, 0.0, 0.0]);
        assert_eq!(prop.scale, [1.0, 1.0, 1.0]);
        assert_eq!(prop.collider_type, ColliderType::None);
        assert!(!prop.is_hurtbox);
        assert_eq!(prop.resource_value, 0);
        assert!(prop.anchor_id.is_none());
    }

    #[test]
    fn prop_rotation_contract_is_degrees() {
        let prop: PropData =
            serde_json::from_str(r#"{ "asset_id": "Cube.obj", "rotation": [0.0, 90.0, 180.0] }"#)
                .unwrap();
        let radians = prop.rotation_radians();
        assert_eq!(radians[0], 0.0);
        assert!((radians[1] - std::f32::consts::FRAC_PI_2).abs() < f32::EPSILON);
        assert!((radians[2] - std::f32::consts::PI).abs() < f32::EPSILON);
    }

    #[test]
    fn validation_allows_enemy_health_to_come_from_definition() {
        let prop: PropData =
            serde_json::from_str(r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened" }"#)
                .unwrap();

        let errors = prop.validation_errors(0);
        assert!(errors.is_empty(), "{:?}", errors);
    }

    #[test]
    fn validation_rejects_empty_anchor_id() {
        let prop: PropData =
            serde_json::from_str(r#"{ "asset_id": "Cube.obj", "anchor_id": "" }"#).unwrap();

        let errors = prop.validation_errors(0);
        assert!(errors.iter().any(|error| error.contains("anchor_id")));
    }

    #[test]
    fn validation_rejects_enemy_resource_combo() {
        let prop: PropData = serde_json::from_str(
            r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened", "resource_value": 5 }"#,
        )
        .unwrap();

        let errors = prop.validation_errors(0);
        assert!(errors.iter().any(|error| error.contains("resource pickup")));
    }

    #[test]
    fn save_writes_level_json() {
        let level = LevelData::default_level();
        let path = std::env::temp_dir().join("cenotaph_level_save_test.json");
        level.save(path.to_str().unwrap()).unwrap();
        assert!(path.exists());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn foundation_test_level_validates() {
        let level = LevelData::load("levels/foundation_test.json");
        assert_eq!(level.props.len(), 7);
        assert_eq!(level.validate(), Ok(()));
    }
}

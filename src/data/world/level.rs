// src/data/world/level.rs
// LevelData is the serializable description of a single playable area.
// PropData describes every object placed inside that area.
// ColliderType drives the physics engine's collider selection.
//
// This module defines the data structures used to represent levels and their contents.
// Levels are serialized to JSON and can be loaded dynamically at runtime.
// The system supports both static level geometry and dynamic props with various gameplay properties.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::persistence::{recover_interrupted_write, write_file_staged};

pub fn validate_level_id(level_id: &str) -> Result<(), String> {
    if level_id.is_empty() {
        return Err("level id must not be empty".to_string());
    }
    if level_id.len() > 64 {
        return Err("level id must not exceed 64 characters".to_string());
    }
    if !level_id
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(format!(
            "invalid level id '{}': use only letters, numbers, '-' and '_'",
            level_id
        ));
    }
    Ok(())
}

// ── Collider shape tag ────────────────────────────────────────────────────────

/// Defines the collision shape type for props and objects in the level.
///
/// This enum determines how the physics engine creates collision shapes for objects.
/// Different shapes have different performance characteristics and use cases.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
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

fn default_one_u32() -> u32 {
    1
}

fn default_one_f32() -> f32 {
    1.0
}

fn default_trigger_radius() -> f32 {
    2.5
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelPathKind {
    #[default]
    Enemy,
    Npc,
    Platform,
    Cinematic,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelEventTriggerKind {
    #[default]
    Proximity,
    OnEnter,
    Interact,
    Manual,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LevelEventActionKind {
    #[default]
    SetFlag,
    LoadLevel,
    GrantResource,
    SpawnLoot,
    StartDialogue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AssetImportData {
    pub id: String,
    pub asset_id: String,
    #[serde(default)]
    pub source_path: Option<String>,
    #[serde(default = "unit_scale")]
    pub default_scale: [f32; 3],
    #[serde(default)]
    pub default_collider_type: ColliderType,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LootTableData {
    pub id: String,
    #[serde(default = "default_one_u32")]
    pub rolls: u32,
    #[serde(default)]
    pub entries: Vec<LootEntryData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LootEntryData {
    #[serde(default = "default_one_u32")]
    pub weight: u32,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub resource_value: u32,
    #[serde(default = "default_one_u32")]
    pub quantity: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelPathData {
    pub id: String,
    #[serde(default)]
    pub kind: LevelPathKind,
    #[serde(default)]
    pub looped: bool,
    #[serde(default = "default_one_f32")]
    pub speed_multiplier: f32,
    #[serde(default)]
    pub waypoints: Vec<[f32; 3]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelEventTriggerData {
    #[serde(default)]
    pub kind: LevelEventTriggerKind,
    #[serde(default)]
    pub position: [f32; 3],
    #[serde(default = "default_trigger_radius")]
    pub radius: f32,
    #[serde(default)]
    pub prop_id: Option<String>,
    #[serde(default)]
    pub flag_id: Option<String>,
}

impl Default for LevelEventTriggerData {
    fn default() -> Self {
        Self {
            kind: LevelEventTriggerKind::Proximity,
            position: [0.0, 0.0, 0.0],
            radius: default_trigger_radius(),
            prop_id: None,
            flag_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelEventActionData {
    #[serde(default)]
    pub kind: LevelEventActionKind,
    #[serde(default)]
    pub target_level_id: Option<String>,
    #[serde(default)]
    pub loot_table_id: Option<String>,
    #[serde(default)]
    pub dialogue_id: Option<String>,
    #[serde(default)]
    pub flag_id: Option<String>,
    #[serde(default)]
    pub resource_value: u32,
    #[serde(default)]
    pub spawn_position: Option<[f32; 3]>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelEventData {
    pub id: String,
    #[serde(default = "default_true")]
    pub once: bool,
    #[serde(default)]
    pub trigger: LevelEventTriggerData,
    #[serde(default)]
    pub actions: Vec<LevelEventActionData>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DialogueData {
    pub id: String,
    pub speaker: String,
    #[serde(default)]
    pub lines: Vec<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TerrainBrushData {
    #[serde(default = "default_terrain_resolution")]
    pub columns: u32,
    #[serde(default = "default_terrain_resolution")]
    pub rows: u32,
    #[serde(default)]
    pub seed: u32,
    #[serde(default = "default_terrain_relief")]
    pub relief: f32,
    #[serde(default = "default_terrain_base_thickness")]
    pub base_thickness: f32,
    #[serde(default = "default_terrain_sculpt_strength")]
    pub sculpt_strength: f32,
}

fn default_terrain_resolution() -> u32 {
    8
}

fn default_terrain_relief() -> f32 {
    3.0
}

fn default_terrain_base_thickness() -> f32 {
    0.5
}

fn default_terrain_sculpt_strength() -> f32 {
    0.5
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BrushGeometryData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terrain: Option<TerrainBrushData>,
    #[serde(default)]
    pub vertices: Vec<[f32; 3]>,
    #[serde(default)]
    pub faces: Vec<[u32; 3]>,
}

// -- Prop -------------------------------------------------------------------

/// A single object placed inside a level — geometry, physics, gameplay metadata.
///
/// Props represent all interactive and decorative objects in a level, including
/// enemies, items, environmental objects, and gameplay triggers. Each prop has
/// associated geometry, collision properties, and gameplay metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropData {
    /// Optional stable authoring ID used by editor tools and level events.
    #[serde(default)]
    pub id: Option<String>,
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
    /// Optional editor-authored local mesh used for brush/slope geometry.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub brush_geometry: Option<BrushGeometryData>,
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
    /// Optional loot table used by enemies, containers, or scripted events.
    #[serde(default)]
    pub loot_table_id: Option<String>,
    /// Optional authored patrol/platform/cinematic path.
    #[serde(default)]
    pub path_id: Option<String>,
    /// Optional dialogue that can be started when interacting with this prop.
    #[serde(default)]
    pub dialogue_id: Option<String>,
    /// Optional level event invoked by this prop in future editor workflows.
    #[serde(default)]
    pub event_id: Option<String>,
}

//-- Level ------------------------------------------------------------

/// The full data description of a playable level / stratum.
///
/// A LevelData contains all the information needed to load and render a complete
/// game level, including the base geometry, props, and atmospheric settings.
/// Levels are serialized to JSON and can be loaded dynamically.
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
    #[serde(default)]
    pub props: Vec<PropData>,
    /// Imported/curated model assets exposed to the editor for this level.
    #[serde(default)]
    pub asset_imports: Vec<AssetImportData>,
    /// Level-local loot tables used by enemies, events, and containers.
    #[serde(default)]
    pub loot_tables: Vec<LootTableData>,
    /// Authored movement paths for enemies, NPCs, platforms, and cinematics.
    #[serde(default)]
    pub paths: Vec<LevelPathData>,
    /// Scripted event triggers and actions.
    #[serde(default)]
    pub events: Vec<LevelEventData>,
    /// Level-local dialogue blocks.
    #[serde(default)]
    pub dialogues: Vec<DialogueData>,
}

impl LevelData {
    /// Returns a minimal level fixture for tests and authoring utilities.
    /// Runtime loading is strict and never substitutes this for broken content.
    #[cfg(test)]
    pub fn default_level() -> Self {
        Self {
            name: "map_001".to_string(),
            base_map: "assets/map_001.glb".to_string(),
            player_spawn: [0.0, 10.0, 0.0],
            props: Vec::new(),
            asset_imports: Vec::new(),
            loot_tables: Vec::new(),
            paths: Vec::new(),
            events: Vec::new(),
            dialogues: Vec::new(),
        }
    }

    /// Loads a level from disk without falling back.
    ///
    /// Use this for validation and tooling, where broken content should fail
    /// loudly instead of being replaced by a default level.
    pub fn try_load(file_path: &str) -> Result<Self, String> {
        recover_interrupted_write(file_path)?;
        if !Path::new(file_path).exists() {
            return Err(format!("level file not found at {}", file_path));
        }

        let data = fs::read_to_string(file_path)
            .map_err(|e| format!("failed to read level file at {}: {}", file_path, e))?;
        serde_json::from_str(&data)
            .map_err(|e| format!("failed to parse level JSON at {}: {}", file_path, e))
    }

    /// Saves the level as pretty JSON after validating the authoring contract.
    pub fn save_to_path(&self, file_path: impl AsRef<Path>) -> Result<(), String> {
        self.validate()
            .map_err(|errors| format!("level validation failed: {}", errors.join("; ")))?;

        let file_path = file_path.as_ref();
        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize level JSON: {}", e))?;
        write_file_staged(file_path, format!("{}\n", data).as_bytes())
            .map_err(|e| format!("failed to write level JSON: {}", e))
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

        let mut prop_ids = std::collections::HashSet::new();
        for (index, prop) in self.props.iter().enumerate() {
            errors.extend(prop.validation_errors(index));
            if let Some(id) = prop.id.as_deref() {
                collect_unique_id("prop", index, id, &mut prop_ids, &mut errors);
            }
        }

        let _asset_import_ids = collect_ids(
            "asset import",
            self.asset_imports
                .iter()
                .enumerate()
                .map(|(index, asset)| (index, asset.id.as_str())),
            &mut errors,
        );
        for (index, asset) in self.asset_imports.iter().enumerate() {
            errors.extend(asset.validation_errors(index));
        }

        let loot_table_ids = collect_ids(
            "loot table",
            self.loot_tables
                .iter()
                .enumerate()
                .map(|(index, table)| (index, table.id.as_str())),
            &mut errors,
        );
        for (index, table) in self.loot_tables.iter().enumerate() {
            errors.extend(table.validation_errors(index));
        }

        let path_ids = collect_ids(
            "path",
            self.paths
                .iter()
                .enumerate()
                .map(|(index, path)| (index, path.id.as_str())),
            &mut errors,
        );
        for (index, path) in self.paths.iter().enumerate() {
            errors.extend(path.validation_errors(index));
        }

        let event_ids = collect_ids(
            "event",
            self.events
                .iter()
                .enumerate()
                .map(|(index, event)| (index, event.id.as_str())),
            &mut errors,
        );
        let dialogue_ids = collect_ids(
            "dialogue",
            self.dialogues
                .iter()
                .enumerate()
                .map(|(index, dialogue)| (index, dialogue.id.as_str())),
            &mut errors,
        );
        for (index, event) in self.events.iter().enumerate() {
            errors.extend(event.validation_errors(
                index,
                &prop_ids,
                &loot_table_ids,
                &path_ids,
                &event_ids,
                &dialogue_ids,
            ));
        }

        for (index, dialogue) in self.dialogues.iter().enumerate() {
            errors.extend(dialogue.validation_errors(index));
        }

        for (index, prop) in self.props.iter().enumerate() {
            let label = format!("prop {} ('{}')", index, prop.asset_id);
            validate_reference(
                &label,
                "loot_table_id",
                prop.loot_table_id.as_deref(),
                &loot_table_ids,
                &mut errors,
            );
            validate_reference(
                &label,
                "path_id",
                prop.path_id.as_deref(),
                &path_ids,
                &mut errors,
            );
            validate_reference(
                &label,
                "dialogue_id",
                prop.dialogue_id.as_deref(),
                &dialogue_ids,
                &mut errors,
            );
            validate_reference(
                &label,
                "event_id",
                prop.event_id.as_deref(),
                &event_ids,
                &mut errors,
            );
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
}

impl BrushGeometryData {
    pub fn validation_errors(&self, label: &str) -> Vec<String> {
        let mut errors = Vec::new();

        if self.kind.as_deref() == Some("terrain") && self.terrain.is_none() {
            errors.push(format!(
                "{} terrain brush_geometry must include terrain metadata",
                label
            ));
        }
        if let Some(terrain) = self.terrain.as_ref() {
            if self.kind.as_deref() != Some("terrain") {
                errors.push(format!(
                    "{} brush_geometry terrain metadata requires kind 'terrain'",
                    label
                ));
            }
            if !(2..=24).contains(&terrain.columns) || !(2..=24).contains(&terrain.rows) {
                errors.push(format!(
                    "{} terrain grid must use between 2 and 24 rows and columns",
                    label
                ));
            } else {
                let expected_vertices = ((terrain.columns + 1) * (terrain.rows + 1) * 2) as usize;
                if self.vertices.len() != expected_vertices {
                    errors.push(format!(
                        "{} terrain grid metadata expects {} vertices, found {}",
                        label,
                        expected_vertices,
                        self.vertices.len()
                    ));
                }
            }
            if !terrain.relief.is_finite() || terrain.relief < 0.0 {
                errors.push(format!(
                    "{} terrain relief must be finite and non-negative",
                    label
                ));
            }
            if !terrain.base_thickness.is_finite() || terrain.base_thickness <= 0.0 {
                errors.push(format!(
                    "{} terrain base_thickness must be finite and greater than zero",
                    label
                ));
            }
            if !terrain.sculpt_strength.is_finite() || terrain.sculpt_strength <= 0.0 {
                errors.push(format!(
                    "{} terrain sculpt_strength must be finite and greater than zero",
                    label
                ));
            }
        }

        if self.vertices.len() < 3 {
            errors.push(format!(
                "{} brush_geometry must contain at least 3 vertices",
                label
            ));
        }
        if self.vertices.len() > 4096 {
            errors.push(format!(
                "{} brush_geometry must not contain more than 4096 vertices",
                label
            ));
        }
        if self.faces.is_empty() {
            errors.push(format!(
                "{} brush_geometry must contain at least 1 triangle face",
                label
            ));
        }
        if self.faces.len() > 8192 {
            errors.push(format!(
                "{} brush_geometry must not contain more than 8192 triangle faces",
                label
            ));
        }

        for (vertex_index, vertex) in self.vertices.iter().enumerate() {
            if !vertex.iter().all(|value| value.is_finite()) {
                errors.push(format!(
                    "{} brush_geometry vertex {} must contain finite numbers",
                    label, vertex_index
                ));
            }
        }

        for (face_index, face) in self.faces.iter().enumerate() {
            let vertex_count = self.vertices.len() as u32;
            if face.iter().any(|index| *index >= vertex_count) {
                errors.push(format!(
                    "{} brush_geometry face {} references a missing vertex",
                    label, face_index
                ));
                continue;
            }
            if face[0] == face[1] || face[1] == face[2] || face[0] == face[2] {
                errors.push(format!(
                    "{} brush_geometry face {} must reference three unique vertices",
                    label, face_index
                ));
                continue;
            }
            if self.vertices.len() >= 3 {
                let a = self.vertices[face[0] as usize];
                let b = self.vertices[face[1] as usize];
                let c = self.vertices[face[2] as usize];
                let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
                let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
                let cross = [
                    ab[1] * ac[2] - ab[2] * ac[1],
                    ab[2] * ac[0] - ab[0] * ac[2],
                    ab[0] * ac[1] - ab[1] * ac[0],
                ];
                let area_squared = cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2];
                if area_squared <= 0.000001 {
                    errors.push(format!(
                        "{} brush_geometry face {} must not be degenerate",
                        label, face_index
                    ));
                }
            }
        }

        errors
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
        } else if self.brush_geometry.is_none() {
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
        validate_optional_authoring_id(&label, "id", self.id.as_deref(), &mut errors);
        validate_optional_authoring_id(&label, "path_id", self.path_id.as_deref(), &mut errors);
        validate_optional_authoring_id(
            &label,
            "loot_table_id",
            self.loot_table_id.as_deref(),
            &mut errors,
        );
        validate_optional_authoring_id(
            &label,
            "dialogue_id",
            self.dialogue_id.as_deref(),
            &mut errors,
        );
        validate_optional_authoring_id(&label, "event_id", self.event_id.as_deref(), &mut errors);
        if self
            .item_id
            .as_ref()
            .is_some_and(|item_id| item_id.trim().is_empty())
        {
            errors.push(format!("{} item_id must not be empty", label));
        }
        if self.resource_value > 0 && self.enemy_type.is_some() {
            errors.push(format!(
                "{} cannot be both a resource pickup and an enemy",
                label
            ));
        }
        if self.resource_value > 0 && self.item_id.is_some() {
            errors.push(format!(
                "{} cannot be both a resource pickup and an item pickup",
                label
            ));
        }
        if self.enemy_type.is_some() && self.item_id.is_some() {
            errors.push(format!(
                "{} cannot be both an item pickup and an enemy",
                label
            ));
        }
        if self.enemy_type.is_some() && self.enemy_health < 0.0 {
            errors.push(format!("{} enemy_health must not be negative", label));
        }
        if let Some(target) = self.trigger_level_id.as_ref() {
            if let Err(error) = validate_level_id(target) {
                errors.push(format!("{} trigger_level_id is invalid: {}", label, error));
            } else {
                let target_path = format!("levels/{}.json", target);
                if !Path::new(&target_path).exists() {
                    errors.push(format!(
                        "{} trigger_level_id references missing level '{}'",
                        label, target_path
                    ));
                }
            }
        }
        if self.light_color.is_none() && self.light_intensity > 0.0 {
            errors.push(format!(
                "{} light_intensity is set without a light_color",
                label
            ));
        }
        if let Some(geometry) = self.brush_geometry.as_ref() {
            if matches!(self.collider_type, ColliderType::Box | ColliderType::Sphere) {
                errors.push(format!(
                    "{} brush_geometry should use Mesh or None collider_type",
                    label
                ));
            }
            errors.extend(geometry.validation_errors(&label));
        }

        errors
    }
}

impl AssetImportData {
    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("asset import {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        if self.asset_id.trim().is_empty() {
            errors.push(format!("{} asset_id must not be empty", label));
        } else if !authoring_asset_exists(&self.asset_id, self.source_path.as_deref()) {
            errors.push(format!(
                "{} references missing asset '{}' or source_path",
                label, self.asset_id
            ));
        }
        if !self.default_scale.iter().all(|v| v.is_finite()) {
            errors.push(format!(
                "{} default_scale must contain finite numbers",
                label
            ));
        }
        if self.default_scale.iter().any(|v| v.abs() <= f32::EPSILON) {
            errors.push(format!(
                "{} default_scale must not contain zero values",
                label
            ));
        }
        if self
            .source_path
            .as_ref()
            .is_some_and(|source_path| source_path.trim().is_empty())
        {
            errors.push(format!("{} source_path must not be empty", label));
        }
        if self
            .notes
            .as_ref()
            .is_some_and(|notes| notes.trim().is_empty())
        {
            errors.push(format!("{} notes must not be empty", label));
        }

        errors
    }
}

fn authoring_asset_exists(asset_id: &str, source_path: Option<&str>) -> bool {
    let asset_id = asset_id.trim();
    if asset_id.is_empty() {
        return false;
    }
    if Path::new(asset_id).exists() {
        return true;
    }
    if Path::new("assets").join(asset_id).exists() {
        return true;
    }
    source_path
        .map(str::trim)
        .filter(|source_path| !source_path.is_empty())
        .is_some_and(|source_path| Path::new(source_path).exists())
}

impl LootTableData {
    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("loot table {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        if self.rolls == 0 {
            errors.push(format!("{} rolls must be at least 1", label));
        }
        if self.entries.is_empty() {
            errors.push(format!("{} must contain at least one entry", label));
        }
        for (entry_index, entry) in self.entries.iter().enumerate() {
            errors.extend(entry.validation_errors(&label, entry_index));
        }

        errors
    }
}

impl LootEntryData {
    pub fn validation_errors(&self, table_label: &str, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("{} entry {}", table_label, index);
        if self.weight == 0 {
            errors.push(format!("{} weight must be at least 1", label));
        }
        if self.quantity == 0 {
            errors.push(format!("{} quantity must be at least 1", label));
        }
        if self
            .item_id
            .as_ref()
            .is_some_and(|item_id| item_id.trim().is_empty())
        {
            errors.push(format!("{} item_id must not be empty", label));
        }
        if self.item_id.is_none() && self.resource_value == 0 {
            errors.push(format!(
                "{} must grant either an item_id or resource_value",
                label
            ));
        }
        if self.item_id.is_some() && self.resource_value > 0 {
            errors.push(format!(
                "{} cannot grant both item_id and resource_value",
                label
            ));
        }

        errors
    }
}

impl LevelPathData {
    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("path {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        if !self.speed_multiplier.is_finite() || self.speed_multiplier <= 0.0 {
            errors.push(format!("{} speed_multiplier must be > 0", label));
        }
        if self.waypoints.len() < 2 {
            errors.push(format!("{} must contain at least two waypoints", label));
        }
        for (waypoint_index, waypoint) in self.waypoints.iter().enumerate() {
            if !waypoint.iter().all(|v| v.is_finite()) {
                errors.push(format!(
                    "{} waypoint {} must contain finite numbers",
                    label, waypoint_index
                ));
            }
        }

        errors
    }
}

impl LevelEventData {
    pub fn validation_errors(
        &self,
        index: usize,
        prop_ids: &std::collections::HashSet<String>,
        loot_table_ids: &std::collections::HashSet<String>,
        path_ids: &std::collections::HashSet<String>,
        _event_ids: &std::collections::HashSet<String>,
        dialogue_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("event {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        errors.extend(self.trigger.validation_errors(&label, prop_ids));
        if self.actions.is_empty() {
            errors.push(format!("{} must contain at least one action", label));
        }
        for (action_index, action) in self.actions.iter().enumerate() {
            errors.extend(action.validation_errors(
                &label,
                action_index,
                loot_table_ids,
                path_ids,
                dialogue_ids,
            ));
        }

        errors
    }
}

impl LevelEventTriggerData {
    pub fn validation_errors(
        &self,
        event_label: &str,
        prop_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("{} trigger", event_label);
        if !self.position.iter().all(|v| v.is_finite()) {
            errors.push(format!("{} position must contain finite numbers", label));
        }
        if !self.radius.is_finite() || self.radius <= 0.0 {
            errors.push(format!("{} radius must be > 0", label));
        }
        validate_optional_authoring_id(&label, "prop_id", self.prop_id.as_deref(), &mut errors);
        validate_optional_authoring_id(&label, "flag_id", self.flag_id.as_deref(), &mut errors);
        if self.kind == LevelEventTriggerKind::Interact && self.prop_id.is_none() {
            errors.push(format!("{} interact triggers require prop_id", label));
        }
        validate_reference(
            &label,
            "prop_id",
            self.prop_id.as_deref(),
            prop_ids,
            &mut errors,
        );

        errors
    }
}

impl LevelEventActionData {
    pub fn validation_errors(
        &self,
        event_label: &str,
        index: usize,
        loot_table_ids: &std::collections::HashSet<String>,
        _path_ids: &std::collections::HashSet<String>,
        dialogue_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("{} action {}", event_label, index);
        validate_optional_authoring_id(
            &label,
            "loot_table_id",
            self.loot_table_id.as_deref(),
            &mut errors,
        );
        validate_optional_authoring_id(
            &label,
            "dialogue_id",
            self.dialogue_id.as_deref(),
            &mut errors,
        );
        validate_optional_authoring_id(&label, "flag_id", self.flag_id.as_deref(), &mut errors);
        if let Some(position) = self.spawn_position {
            if !position.iter().all(|v| v.is_finite()) {
                errors.push(format!(
                    "{} spawn_position must contain finite numbers",
                    label
                ));
            }
        }

        match self.kind {
            LevelEventActionKind::LoadLevel => {
                let Some(target) = self.target_level_id.as_ref() else {
                    errors.push(format!("{} LoadLevel requires target_level_id", label));
                    return errors;
                };
                if let Err(error) = validate_level_id(target) {
                    errors.push(format!("{} target_level_id is invalid: {}", label, error));
                } else {
                    let target_path = format!("levels/{}.json", target);
                    if !Path::new(&target_path).exists() {
                        errors.push(format!(
                            "{} target_level_id references missing level '{}'",
                            label, target_path
                        ));
                    }
                }
            }
            LevelEventActionKind::GrantResource => {
                if self.resource_value == 0 {
                    errors.push(format!(
                        "{} GrantResource requires resource_value > 0",
                        label
                    ));
                }
            }
            LevelEventActionKind::SpawnLoot => {
                if self.loot_table_id.is_none() {
                    errors.push(format!("{} SpawnLoot requires loot_table_id", label));
                }
                validate_reference(
                    &label,
                    "loot_table_id",
                    self.loot_table_id.as_deref(),
                    loot_table_ids,
                    &mut errors,
                );
            }
            LevelEventActionKind::StartDialogue => {
                if self.dialogue_id.is_none() {
                    errors.push(format!("{} StartDialogue requires dialogue_id", label));
                }
                validate_reference(
                    &label,
                    "dialogue_id",
                    self.dialogue_id.as_deref(),
                    dialogue_ids,
                    &mut errors,
                );
            }
            LevelEventActionKind::SetFlag => {
                if self
                    .flag_id
                    .as_ref()
                    .is_none_or(|flag_id| flag_id.trim().is_empty())
                {
                    errors.push(format!("{} SetFlag requires flag_id", label));
                }
            }
        }
        errors
    }
}

impl DialogueData {
    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("dialogue {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        if self.speaker.trim().is_empty() {
            errors.push(format!("{} speaker must not be empty", label));
        }
        if self.lines.is_empty() {
            errors.push(format!("{} must contain at least one line", label));
        }
        for (line_index, line) in self.lines.iter().enumerate() {
            if line.trim().is_empty() {
                errors.push(format!("{} line {} must not be empty", label, line_index));
            }
        }

        errors
    }
}

fn collect_ids<'a>(
    collection: &str,
    ids: impl Iterator<Item = (usize, &'a str)>,
    errors: &mut Vec<String>,
) -> std::collections::HashSet<String> {
    let mut seen = std::collections::HashSet::new();
    for (index, id) in ids {
        collect_unique_id(collection, index, id, &mut seen, errors);
    }
    seen
}

fn collect_unique_id(
    collection: &str,
    index: usize,
    id: &str,
    seen: &mut std::collections::HashSet<String>,
    errors: &mut Vec<String>,
) {
    let label = format!("{} {} ('{}')", collection, index, id);
    validate_authoring_id(&label, id, errors);
    if !id.trim().is_empty() && !seen.insert(id.trim().to_string()) {
        errors.push(format!("duplicate {} id '{}'", collection, id.trim()));
    }
}

fn validate_reference(
    label: &str,
    field: &str,
    value: Option<&str>,
    known_ids: &std::collections::HashSet<String>,
    errors: &mut Vec<String>,
) {
    let Some(value) = value else {
        return;
    };
    if !value.trim().is_empty() && !known_ids.contains(value.trim()) {
        errors.push(format!(
            "{} {} references unknown id '{}'",
            label, field, value
        ));
    }
}

fn validate_authoring_id(label: &str, id: &str, errors: &mut Vec<String>) {
    if id.trim().is_empty() {
        errors.push(format!("{} id must not be empty", label));
    } else if !is_authoring_id(id) {
        errors.push(format!(
            "{} id '{}' must use only letters, numbers, '_' or '-'",
            label, id
        ));
    }
}

fn validate_optional_authoring_id(
    label: &str,
    field: &str,
    value: Option<&str>,
    errors: &mut Vec<String>,
) {
    let Some(value) = value else {
        return;
    };
    if value.trim().is_empty() {
        errors.push(format!("{} {} must not be empty", label, field));
    } else if !is_authoring_id(value) {
        errors.push(format!(
            "{} {} '{}' must use only letters, numbers, '_' or '-'",
            label, field, value
        ));
    }
}

fn is_authoring_id(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

// --- Tests ------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_ids_reject_path_traversal() {
        assert!(validate_level_id("movement_test").is_ok());
        assert!(validate_level_id("../movement_test").is_err());
        assert!(validate_level_id("nested/level").is_err());
    }

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
    fn terrain_brush_metadata_round_trips() {
        let geometry = BrushGeometryData {
            kind: Some("terrain".to_string()),
            terrain: Some(TerrainBrushData {
                columns: 2,
                rows: 2,
                seed: 17,
                relief: 3.0,
                base_thickness: 0.5,
                sculpt_strength: 0.75,
            }),
            vertices: vec![[0.0, 0.0, 0.0]; 18],
            faces: vec![[0, 1, 2]],
        };

        let json = serde_json::to_string(&geometry).unwrap();
        let loaded: BrushGeometryData = serde_json::from_str(&json).unwrap();
        let terrain = loaded.terrain.unwrap();

        assert_eq!(loaded.kind.as_deref(), Some("terrain"));
        assert_eq!(terrain.columns, 2);
        assert_eq!(terrain.rows, 2);
        assert_eq!(terrain.seed, 17);
        assert_eq!(terrain.sculpt_strength, 0.75);
    }

    #[test]
    fn terrain_brush_validation_rejects_invalid_grid_metadata() {
        let geometry = BrushGeometryData {
            kind: Some("terrain".to_string()),
            terrain: Some(TerrainBrushData {
                columns: 1,
                rows: 30,
                seed: 0,
                relief: 1.0,
                base_thickness: 0.5,
                sculpt_strength: 0.5,
            }),
            vertices: vec![[0.0, 0.0, 0.0]; 3],
            faces: vec![[0, 1, 2]],
        };

        assert!(geometry
            .validation_errors("prop 0")
            .iter()
            .any(|error| error.contains("terrain grid must use between 2 and 24")));
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
    fn validation_rejects_empty_item_id() {
        let prop: PropData =
            serde_json::from_str(r#"{ "asset_id": "Cube.obj", "item_id": "" }"#).unwrap();

        let errors = prop.validation_errors(0);
        assert!(errors.iter().any(|error| error.contains("item_id")));
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
    fn validation_rejects_item_resource_combo() {
        let prop: PropData = serde_json::from_str(
            r#"{ "asset_id": "Cube.obj", "item_id": "ash_splinter", "resource_value": 5 }"#,
        )
        .unwrap();

        let errors = prop.validation_errors(0);
        assert!(errors.iter().any(|error| error.contains("item pickup")));
    }

    #[test]
    fn foundation_test_level_validates() {
        let level = LevelData::try_load("levels/foundation_test.json").unwrap();
        assert_eq!(level.props.len(), 16);
        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn movement_test_level_validates() {
        let level = LevelData::try_load("levels/movement_test.json").unwrap();
        assert_eq!(level.props.len(), 18);
        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn level_save_round_trips_pretty_json() {
        let temp_path = std::env::temp_dir().join(format!(
            "cenotaph_level_save_test_{}_{}.json",
            std::process::id(),
            17
        ));
        let level = LevelData {
            name: "Save Test".to_string(),
            base_map: "assets/test_movement_arena.obj".to_string(),
            player_spawn: [0.0, 128.0, 0.0],
            props: vec![PropData {
                id: None,
                asset_id: "props/test_wall.obj".to_string(),
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::Box,
                brush_geometry: None,
                is_climbable: false,
                is_hurtbox: false,
                item_id: None,
                resource_value: 0,
                anchor_id: None,
                enemy_type: None,
                enemy_health: 0.0,
                light_color: None,
                light_intensity: 0.0,
                ambient_sound_id: None,
                trigger_level_id: None,
                loot_table_id: None,
                path_id: None,
                dialogue_id: None,
                event_id: None,
            }],
            asset_imports: Vec::new(),
            loot_tables: vec![LootTableData {
                id: "starter_loot".to_string(),
                rolls: 1,
                entries: vec![LootEntryData {
                    weight: 1,
                    item_id: Some("ash_splinter".to_string()),
                    resource_value: 0,
                    quantity: 1,
                }],
            }],
            paths: vec![LevelPathData {
                id: "enemy_patrol".to_string(),
                kind: LevelPathKind::Enemy,
                looped: true,
                speed_multiplier: 1.0,
                waypoints: vec![[0.0, 128.0, 0.0], [4.0, 128.0, 0.0]],
            }],
            events: vec![LevelEventData {
                id: "arrival_bark".to_string(),
                once: true,
                trigger: LevelEventTriggerData {
                    kind: LevelEventTriggerKind::OnEnter,
                    position: [0.0, 0.0, 0.0],
                    radius: 2.5,
                    prop_id: None,
                    flag_id: None,
                },
                actions: vec![LevelEventActionData {
                    kind: LevelEventActionKind::StartDialogue,
                    target_level_id: None,
                    loot_table_id: None,
                    dialogue_id: Some("opening".to_string()),
                    flag_id: None,
                    resource_value: 0,
                    spawn_position: None,
                }],
            }],
            dialogues: vec![DialogueData {
                id: "opening".to_string(),
                speaker: "Cenotaph".to_string(),
                lines: vec!["The editor remembers this room.".to_string()],
            }],
        };

        level.save_to_path(&temp_path).unwrap();
        let loaded = LevelData::try_load(temp_path.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(&temp_path);

        assert_eq!(loaded.name, "Save Test");
        assert_eq!(loaded.props.len(), 1);
        assert_eq!(loaded.props[0].collider_type, ColliderType::Box);
        assert_eq!(loaded.loot_tables.len(), 1);
        assert_eq!(loaded.paths.len(), 1);
        assert_eq!(loaded.events.len(), 1);
        assert_eq!(loaded.dialogues.len(), 1);
    }

    #[test]
    fn advanced_authoring_defaults_survive_minimal_level_json() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Minimal",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": []
            }
            "#,
        )
        .unwrap();

        assert!(level.asset_imports.is_empty());
        assert!(level.loot_tables.is_empty());
        assert!(level.paths.is_empty());
        assert!(level.events.is_empty());
        assert!(level.dialogues.is_empty());
    }

    #[test]
    fn asset_imports_can_track_source_files_outside_assets() {
        let import = AssetImportData {
            id: "source_texture".to_string(),
            asset_id: "textures/source_albedo.webp".to_string(),
            source_path: Some("Cargo.toml".to_string()),
            default_scale: [1.0, 1.0, 1.0],
            default_collider_type: ColliderType::None,
            tags: vec!["texture".to_string()],
            notes: Some("source-only editor import".to_string()),
        };

        assert!(import.validation_errors(0).is_empty());
    }

    #[test]
    fn validation_accepts_custom_brush_geometry_without_asset_file() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Brush Geometry Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    {
                        "asset_id": "editor/brush_geometry",
                        "collider_type": "Mesh",
                        "brush_geometry": {
                            "vertices": [
                                [-1.0, 0.0, -1.0],
                                [1.0, 0.0, -1.0],
                                [1.0, 1.0, 1.0],
                                [-1.0, 0.0, 1.0]
                            ],
                            "faces": [[0, 1, 2], [0, 2, 3]]
                        }
                    }
                ]
            }
            "#,
        )
        .unwrap();

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_broken_custom_brush_geometry() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Broken Brush Geometry Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    {
                        "asset_id": "editor/brush_geometry",
                        "collider_type": "Mesh",
                        "brush_geometry": {
                            "vertices": [
                                [0.0, 0.0, 0.0],
                                [1.0, 0.0, 0.0],
                                [2.0, 0.0, 0.0]
                            ],
                            "faces": [[0, 1, 2], [0, 1, 9]]
                        }
                    }
                ]
            }
            "#,
        )
        .unwrap();
        let errors = level.validate().unwrap_err();

        assert!(errors
            .iter()
            .any(|error| error.contains("brush_geometry face 0 must not be degenerate")));
        assert!(errors
            .iter()
            .any(|error| error.contains("brush_geometry face 1 references a missing vertex")));
    }

    #[test]
    fn validation_accepts_connected_authoring_graph() {
        let level = LevelData {
            name: "Authoring Graph".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            props: vec![PropData {
                id: Some("guard_01".to_string()),
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::Sphere,
                brush_geometry: None,
                is_climbable: false,
                is_hurtbox: false,
                item_id: None,
                resource_value: 0,
                anchor_id: None,
                enemy_type: Some("ashbound".to_string()),
                enemy_health: 10.0,
                light_color: None,
                light_intensity: 0.0,
                ambient_sound_id: None,
                trigger_level_id: None,
                loot_table_id: Some("guard_drops".to_string()),
                path_id: Some("guard_patrol".to_string()),
                dialogue_id: None,
                event_id: Some("guard_intro".to_string()),
            }],
            asset_imports: vec![AssetImportData {
                id: "cube_import".to_string(),
                asset_id: "Cube.obj".to_string(),
                source_path: None,
                default_scale: [1.0, 1.0, 1.0],
                default_collider_type: ColliderType::Box,
                tags: vec!["test".to_string()],
                notes: Some("fixture".to_string()),
            }],
            loot_tables: vec![LootTableData {
                id: "guard_drops".to_string(),
                rolls: 1,
                entries: vec![LootEntryData {
                    weight: 2,
                    item_id: None,
                    resource_value: 25,
                    quantity: 1,
                }],
            }],
            paths: vec![LevelPathData {
                id: "guard_patrol".to_string(),
                kind: LevelPathKind::Enemy,
                looped: true,
                speed_multiplier: 0.75,
                waypoints: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            }],
            events: vec![LevelEventData {
                id: "guard_intro".to_string(),
                once: true,
                trigger: LevelEventTriggerData {
                    kind: LevelEventTriggerKind::Interact,
                    position: [0.0, 0.0, 0.0],
                    radius: 2.5,
                    prop_id: Some("guard_01".to_string()),
                    flag_id: None,
                },
                actions: vec![LevelEventActionData {
                    kind: LevelEventActionKind::StartDialogue,
                    target_level_id: None,
                    loot_table_id: None,
                    dialogue_id: Some("guard_dialogue".to_string()),
                    flag_id: None,
                    resource_value: 0,
                    spawn_position: None,
                }],
            }],
            dialogues: vec![DialogueData {
                id: "guard_dialogue".to_string(),
                speaker: "Guard".to_string(),
                lines: vec!["Stay on the path.".to_string()],
            }],
        };

        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn validation_rejects_broken_authoring_references() {
        let level = LevelData {
            name: "Broken Graph".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            props: vec![PropData {
                id: Some("bad prop".to_string()),
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::None,
                brush_geometry: None,
                is_climbable: false,
                is_hurtbox: false,
                item_id: None,
                resource_value: 0,
                anchor_id: None,
                enemy_type: None,
                enemy_health: 0.0,
                light_color: None,
                light_intensity: 0.0,
                ambient_sound_id: None,
                trigger_level_id: None,
                loot_table_id: Some("missing_loot".to_string()),
                path_id: Some("missing_path".to_string()),
                dialogue_id: Some("missing_dialogue".to_string()),
                event_id: Some("missing_event".to_string()),
            }],
            asset_imports: Vec::new(),
            loot_tables: vec![LootTableData {
                id: "bad_table".to_string(),
                rolls: 0,
                entries: vec![LootEntryData {
                    weight: 0,
                    item_id: Some("ash_splinter".to_string()),
                    resource_value: 5,
                    quantity: 0,
                }],
            }],
            paths: vec![LevelPathData {
                id: "short_path".to_string(),
                kind: LevelPathKind::Enemy,
                looped: false,
                speed_multiplier: 0.0,
                waypoints: vec![[0.0, 0.0, 0.0]],
            }],
            events: vec![LevelEventData {
                id: "broken_event".to_string(),
                once: true,
                trigger: LevelEventTriggerData {
                    kind: LevelEventTriggerKind::Interact,
                    position: [0.0, 0.0, 0.0],
                    radius: 0.0,
                    prop_id: None,
                    flag_id: None,
                },
                actions: vec![LevelEventActionData {
                    kind: LevelEventActionKind::SpawnLoot,
                    target_level_id: None,
                    loot_table_id: Some("missing_loot".to_string()),
                    dialogue_id: None,
                    flag_id: None,
                    resource_value: 0,
                    spawn_position: Some([f32::NAN, 0.0, 0.0]),
                }],
            }],
            dialogues: vec![DialogueData {
                id: "empty_dialogue".to_string(),
                speaker: String::new(),
                lines: vec![String::new()],
            }],
        };

        let errors = level.validation_errors();
        assert!(errors.iter().any(|error| error.contains("bad prop")));
        assert!(errors.iter().any(|error| error.contains("missing_loot")));
        assert!(errors
            .iter()
            .any(|error| error.contains("speed_multiplier")));
        assert!(errors
            .iter()
            .any(|error| error.contains("interact triggers require prop_id")));
        assert!(errors.iter().any(|error| error.contains("spawn_position")));
        assert!(errors.iter().any(|error| error.contains("speaker")));
    }
}

//! Versioned JSON schema, migration, and validation for playable levels.
//!
//! Authored references are validated here before runtime systems materialize
//! enemies, collision, events, loot, dialogue, and atmosphere.

use std::fs;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

use crate::core::persistence::recover_interrupted_write;

pub const CURRENT_LEVEL_VERSION: u32 = 1;
/// World-space Y translation applied to authored base-map model geometry.
pub const BASE_MAP_Y_OFFSET: f32 = 124.5;
/// Reserved namespace for deterministic pickups created by runtime loot rolls.
pub const RUNTIME_LOOT_ID_PREFIX: &str = "runtime_loot_";

fn legacy_level_version() -> u32 {
    0
}

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

/// Collider geometry requested for an authored prop.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Default)]
pub enum ColliderType {
    /// No collider; the prop is visual or event-only.
    #[default]
    None,
    /// Axis-aligned box derived from the prop transform.
    Box,
    /// Sphere derived from the prop transform.
    Sphere,
    /// Triangle mesh loaded from the prop geometry.
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

fn default_mountain_reaction_duration() -> f32 {
    4.0
}

fn default_material_tint() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

fn default_material_uv_scale() -> f32 {
    8.0
}

fn default_clear_color() -> [f32; 3] {
    [0.025, 0.025, 0.035]
}

fn default_fog_color() -> [f32; 3] {
    [0.10, 0.10, 0.15]
}

fn default_fog_density() -> f32 {
    0.01
}

fn default_key_light_color() -> [f32; 3] {
    [1.0, 0.8, 0.5]
}

fn default_key_light_intensity() -> f32 {
    2.0
}

fn default_particle_count() -> u32 {
    96
}

fn default_particle_color() -> [f32; 3] {
    [0.62, 0.65, 0.68]
}

fn default_particle_opacity() -> f32 {
    0.28
}

fn default_particle_size() -> f32 {
    0.045
}

fn default_particle_radius() -> f32 {
    24.0
}

fn default_particle_height() -> f32 {
    12.0
}

fn default_particle_speed() -> f32 {
    0.35
}

fn default_wind() -> [f32; 3] {
    [0.08, 0.0, 0.03]
}

fn default_ambience_volume() -> f32 {
    0.16
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParticlePreset {
    None,
    Ashfall,
    Embers,
    #[default]
    Dust,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AmbiencePreset {
    Silent,
    #[default]
    Omission,
    AshWind,
    EmberVault,
}

/// Lightweight surface authoring shared by base maps and props.
/// Texture paths are relative to `textures/` so projects remain portable.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct SurfaceMaterialData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub texture: Option<String>,
    #[serde(default = "default_material_tint")]
    pub tint: [f32; 3],
    #[serde(default = "default_material_uv_scale")]
    pub uv_scale: f32,
    #[serde(default)]
    pub emissive: f32,
}

impl Default for SurfaceMaterialData {
    fn default() -> Self {
        Self {
            texture: None,
            tint: default_material_tint(),
            uv_scale: default_material_uv_scale(),
            emissive: 0.0,
        }
    }
}

/// Per-level mood settings. The particle budget is intentionally capped so a
/// dramatic atmosphere remains cheap enough to leave enabled during authoring.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(default)]
pub struct AtmosphereData {
    #[serde(default = "default_clear_color")]
    pub clear_color: [f32; 3],
    #[serde(default = "default_fog_color")]
    pub fog_color: [f32; 3],
    #[serde(default = "default_fog_density")]
    pub fog_density: f32,
    #[serde(default = "default_key_light_color")]
    pub key_light_color: [f32; 3],
    #[serde(default = "default_key_light_intensity")]
    pub key_light_intensity: f32,
    #[serde(default)]
    pub particle_preset: ParticlePreset,
    #[serde(default = "default_particle_count")]
    pub particle_count: u32,
    #[serde(default = "default_particle_color")]
    pub particle_color: [f32; 3],
    #[serde(default = "default_particle_opacity")]
    pub particle_opacity: f32,
    #[serde(default = "default_particle_size")]
    pub particle_size: f32,
    #[serde(default = "default_particle_radius")]
    pub particle_radius: f32,
    #[serde(default = "default_particle_height")]
    pub particle_height: f32,
    #[serde(default = "default_particle_speed")]
    pub particle_speed: f32,
    #[serde(default = "default_wind")]
    pub wind: [f32; 3],
    #[serde(default)]
    pub ambience_preset: AmbiencePreset,
    #[serde(default = "default_ambience_volume")]
    pub ambience_volume: f32,
}

impl Default for AtmosphereData {
    fn default() -> Self {
        Self {
            clear_color: default_clear_color(),
            fog_color: default_fog_color(),
            fog_density: default_fog_density(),
            key_light_color: default_key_light_color(),
            key_light_intensity: default_key_light_intensity(),
            particle_preset: ParticlePreset::default(),
            particle_count: default_particle_count(),
            particle_color: default_particle_color(),
            particle_opacity: default_particle_opacity(),
            particle_size: default_particle_size(),
            particle_radius: default_particle_radius(),
            particle_height: default_particle_height(),
            particle_speed: default_particle_speed(),
            wind: default_wind(),
            ambience_preset: AmbiencePreset::default(),
            ambience_volume: default_ambience_volume(),
        }
    }
}

/// Reusable authored atmosphere changes triggered by level events.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MountainReactionData {
    pub id: String,
    #[serde(default = "default_mountain_reaction_duration")]
    pub duration: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_color: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fog_color: Option<[f32; 3]>,
    #[serde(default = "default_one_f32")]
    pub fog_density_multiplier: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_light_color: Option<[f32; 3]>,
    #[serde(default = "default_one_f32")]
    pub key_light_intensity_multiplier: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle_color: Option<[f32; 3]>,
    #[serde(default = "default_one_f32")]
    pub particle_speed_multiplier: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wind: Option<[f32; 3]>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ambience_preset: Option<AmbiencePreset>,
    #[serde(default = "default_one_f32")]
    pub ambience_volume_multiplier: f32,
}

impl Default for MountainReactionData {
    fn default() -> Self {
        Self {
            id: String::new(),
            duration: default_mountain_reaction_duration(),
            clear_color: None,
            fog_color: None,
            fog_density_multiplier: default_one_f32(),
            key_light_color: None,
            key_light_intensity_multiplier: default_one_f32(),
            particle_color: None,
            particle_speed_multiplier: default_one_f32(),
            wind: None,
            ambience_preset: None,
            ambience_volume_multiplier: default_one_f32(),
        }
    }
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
    ReactMountain,
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
    pub reaction_id: Option<String>,
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
    /// Optional stable authoring ID used by tooling and level events.
    #[serde(default)]
    pub id: Option<String>,
    /// Optional authored name used by ritual and named-encounter HUD surfaces.
    #[serde(default)]
    pub display_name: Option<String>,
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
    /// Optional texture/tint overrides for this prop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surface_material: Option<SurfaceMaterialData>,
    /// Optional authored local mesh used for brush or slope geometry.
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
    /// Optional manual event fired when this enemy dies or this Anchor is first bound.
    #[serde(default)]
    pub event_id: Option<String>,
}

impl PropData {
    pub fn spawn_enemy(
        enemy: &crate::data::enemy::EnemyDefinition,
        position: [f32; 3],
        scale: [f32; 3],
    ) -> Self {
        Self {
            id: None,
            display_name: None,
            asset_id: enemy.model_asset.clone(),
            position,
            rotation: [0.0, 0.0, 0.0],
            scale,
            collider_type: enemy.collider_type,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            resource_value: 0,
            anchor_id: None,
            enemy_type: Some(enemy.id.clone()),
            enemy_health: enemy.health,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }

    pub fn loot(
        item_id: Option<String>,
        resource_value: u32,
        asset_id: String,
        position: [f32; 3],
        runtime_id: String,
    ) -> Self {
        Self {
            id: Some(runtime_id),
            display_name: None,
            asset_id,
            position,
            rotation: [0.0, 0.0, 0.0],
            scale: [0.35, 0.35, 0.35],
            collider_type: ColliderType::None,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id,
            resource_value,
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
        }
    }
}

//-- Level ------------------------------------------------------------

/// The full data description of a playable level / stratum.
///
/// A LevelData contains all the information needed to load and render a complete
/// game level, including the base geometry, props, and atmospheric settings.
/// Levels are serialized to JSON and can be loaded dynamically.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelData {
    /// Version of the authored level contract. Version 0 represents legacy
    /// files created before explicit schema versioning was introduced.
    #[serde(default = "legacy_level_version")]
    pub version: u32,
    /// Human-readable name of the level for display purposes
    pub name: String,
    /// Path to the base map mesh (e.g. `"assets/map_001.glb"`).
    /// This defines the main geometry of the level that the player walks on.
    pub base_map: String,
    /// Default spawn position for the player [x, y, z]
    pub player_spawn: [f32; 3],
    /// Fog, particles, key light, wind, and procedural ambience for this level.
    #[serde(default)]
    pub atmosphere: AtmosphereData,
    /// Texture and shading parameters applied to the base-map geometry.
    #[serde(default)]
    pub base_material: SurfaceMaterialData,
    /// Reusable atmosphere changes that events can ask the mountain to perform.
    #[serde(default)]
    pub mountain_reactions: Vec<MountainReactionData>,
    /// List of all props placed in this level
    #[serde(default)]
    pub props: Vec<PropData>,
    /// Imported or curated model assets associated with this level.
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
            version: CURRENT_LEVEL_VERSION,
            name: "map_001".to_string(),
            base_map: "assets/map_001.glb".to_string(),
            player_spawn: [0.0, 10.0, 0.0],
            atmosphere: AtmosphereData::default(),
            base_material: SurfaceMaterialData::default(),
            mountain_reactions: Vec::new(),
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
        Self::from_json_str(&data)
            .map_err(|error| format!("failed to load level JSON at {}: {}", file_path, error))
    }

    /// Parses and migrates a level document into the current authored schema.
    /// All level JSON passes through this function so every loading path uses
    /// the same compatibility rules.
    pub fn from_json_str(data: &str) -> Result<Self, String> {
        let level: Self = serde_json::from_str(data)
            .map_err(|error| format!("failed to parse level JSON: {}", error))?;
        Self::migrate_to_current(level)
    }

    fn migrate_to_current(mut level: Self) -> Result<Self, String> {
        if level.version > CURRENT_LEVEL_VERSION {
            return Err(format!(
                "level version {} is newer than supported version {}",
                level.version, CURRENT_LEVEL_VERSION
            ));
        }

        while level.version < CURRENT_LEVEL_VERSION {
            match level.version {
                0 => level.version = 1,
                version => {
                    return Err(format!(
                        "no migration exists from level version {} to {}",
                        version, CURRENT_LEVEL_VERSION
                    ));
                }
            }
        }

        Ok(level)
    }

    /// Returns content-authoring errors that can break runtime assumptions.
    ///
    /// This does not try to prove the level is fun. It only checks the stable
    /// foundation contract: finite transforms, usable scales, sensible enemy
    /// values, and references that can be resolved from disk.
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.version != CURRENT_LEVEL_VERSION {
            errors.push(format!(
                "level version must be {}, found {}",
                CURRENT_LEVEL_VERSION, self.version
            ));
        }

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
        errors.extend(self.atmosphere.validation_errors());
        errors.extend(self.base_material.validation_errors("base_material"));

        let mountain_reaction_ids = collect_ids(
            "mountain reaction",
            self.mountain_reactions
                .iter()
                .enumerate()
                .map(|(index, reaction)| (index, reaction.id.as_str())),
            &mut errors,
        );
        for (index, reaction) in self.mountain_reactions.iter().enumerate() {
            errors.extend(reaction.validation_errors(index));
        }

        let mut prop_ids = std::collections::HashSet::new();
        let mut anchor_ids = std::collections::HashSet::new();
        for (index, prop) in self.props.iter().enumerate() {
            errors.extend(prop.validation_errors(index));
            if let Some(id) = prop.id.as_deref() {
                collect_unique_id("prop", index, id, &mut prop_ids, &mut errors);
            }
            if let Some(anchor_id) = prop.anchor_id.as_deref() {
                collect_unique_id("anchor", index, anchor_id, &mut anchor_ids, &mut errors);
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
                &dialogue_ids,
                &mountain_reaction_ids,
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
            if let Some(event_id) = prop.event_id.as_deref() {
                if prop.enemy_type.is_none() && prop.anchor_id.is_none() {
                    errors.push(format!(
                        "{} event_id is only supported for enemy defeat or Anchor binding",
                        label
                    ));
                }
                if let Some(event) = self.events.iter().find(|event| event.id == event_id) {
                    if event.trigger.kind != LevelEventTriggerKind::Manual {
                        errors.push(format!(
                            "{} event_id '{}' must reference a Manual event",
                            label, event_id
                        ));
                    }
                }
            }
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
        if self
            .display_name
            .as_ref()
            .is_some_and(|display_name| display_name.trim().is_empty())
        {
            errors.push(format!("{} display_name must not be empty", label));
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
        if let Some(material) = self.surface_material.as_ref() {
            errors.extend(material.validation_errors(&format!("{} surface_material", label)));
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
        validate_optional_authoring_id(&label, "anchor_id", self.anchor_id.as_deref(), &mut errors);
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
            .id
            .as_deref()
            .is_some_and(|id| id.starts_with(RUNTIME_LOOT_ID_PREFIX))
        {
            errors.push(format!(
                "{} id must not use the reserved '{}' runtime namespace",
                label, RUNTIME_LOOT_ID_PREFIX
            ));
        }
        if self.loot_table_id.is_some() && self.id.is_none() {
            errors.push(format!(
                "{} requires a stable id when loot_table_id is set",
                label
            ));
        }
        if self.event_id.is_some() && self.id.is_none() {
            errors.push(format!(
                "{} requires a stable id when event_id is set",
                label
            ));
        }
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
        if self.anchor_id.is_some()
            && (self.enemy_type.is_some() || self.item_id.is_some() || self.resource_value > 0)
        {
            errors.push(format!(
                "{} cannot combine an Anchor with an enemy or pickup role",
                label
            ));
        }
        if self.enemy_type.is_some() && (!self.enemy_health.is_finite() || self.enemy_health < 0.0)
        {
            errors.push(format!(
                "{} enemy_health must be finite and non-negative",
                label
            ));
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

impl SurfaceMaterialData {
    pub fn validation_errors(&self, label: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if !finite_color_in_range(self.tint, 0.0, 4.0) {
            errors.push(format!(
                "{} tint must contain finite values between 0 and 4",
                label
            ));
        }
        if !self.uv_scale.is_finite() || !(0.05..=64.0).contains(&self.uv_scale) {
            errors.push(format!("{} uv_scale must be between 0.05 and 64", label));
        }
        if !self.emissive.is_finite() || !(0.0..=4.0).contains(&self.emissive) {
            errors.push(format!("{} emissive must be between 0 and 4", label));
        }
        if let Some(texture) = self.texture.as_deref() {
            let raw_texture = texture;
            let texture = raw_texture.trim();
            let path = Path::new(texture);
            let safe = raw_texture == texture
                && !texture.is_empty()
                && !path.is_absolute()
                && path
                    .components()
                    .all(|component| matches!(component, Component::Normal(_)));
            if !safe {
                errors.push(format!(
                    "{} texture must be a safe path relative to textures/",
                    label
                ));
            } else {
                let extension_supported = path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .is_some_and(|extension| {
                        matches!(
                            extension.to_ascii_lowercase().as_str(),
                            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tga"
                        )
                    });
                if !extension_supported {
                    errors.push(format!("{} texture uses an unsupported format", label));
                } else if !Path::new("textures").join(path).is_file() {
                    errors.push(format!(
                        "{} references missing texture 'textures/{}'",
                        label, texture
                    ));
                }
            }
        }
        errors
    }
}

impl AtmosphereData {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if !finite_color_in_range(self.clear_color, 0.0, 1.0) {
            errors.push("atmosphere clear_color must contain values between 0 and 1".to_string());
        }
        if !finite_color_in_range(self.fog_color, 0.0, 1.0) {
            errors.push("atmosphere fog_color must contain values between 0 and 1".to_string());
        }
        if !finite_color_in_range(self.key_light_color, 0.0, 2.0) {
            errors
                .push("atmosphere key_light_color must contain values between 0 and 2".to_string());
        }
        if !self.fog_density.is_finite() || !(0.0..=0.2).contains(&self.fog_density) {
            errors.push("atmosphere fog_density must be between 0 and 0.2".to_string());
        }
        if !self.key_light_intensity.is_finite()
            || !(0.0..=12.0).contains(&self.key_light_intensity)
        {
            errors.push("atmosphere key_light_intensity must be between 0 and 12".to_string());
        }
        if self.particle_count > 512 {
            errors.push("atmosphere particle_count must not exceed 512".to_string());
        }
        if !finite_color_in_range(self.particle_color, 0.0, 2.0) {
            errors
                .push("atmosphere particle_color must contain values between 0 and 2".to_string());
        }
        validate_atmosphere_range(
            &mut errors,
            "particle_opacity",
            self.particle_opacity,
            0.0,
            1.0,
        );
        validate_atmosphere_range(&mut errors, "particle_size", self.particle_size, 0.01, 2.0);
        validate_atmosphere_range(
            &mut errors,
            "particle_radius",
            self.particle_radius,
            2.0,
            100.0,
        );
        validate_atmosphere_range(
            &mut errors,
            "particle_height",
            self.particle_height,
            2.0,
            100.0,
        );
        validate_atmosphere_range(
            &mut errors,
            "particle_speed",
            self.particle_speed,
            0.0,
            20.0,
        );
        if !self
            .wind
            .iter()
            .all(|value| value.is_finite() && value.abs() <= 20.0)
        {
            errors.push("atmosphere wind values must be finite and between -20 and 20".to_string());
        }
        validate_atmosphere_range(
            &mut errors,
            "ambience_volume",
            self.ambience_volume,
            0.0,
            1.0,
        );
        errors
    }
}

impl MountainReactionData {
    pub fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("mountain reaction {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);

        if !self.duration.is_finite() || self.duration <= 0.0 {
            errors.push(format!(
                "{} duration must be finite and greater than zero",
                label
            ));
        }
        validate_optional_reaction_color(&mut errors, &label, "clear_color", self.clear_color, 1.0);
        validate_optional_reaction_color(&mut errors, &label, "fog_color", self.fog_color, 1.0);
        validate_nonnegative_finite_multiplier(
            &mut errors,
            &label,
            "fog_density_multiplier",
            self.fog_density_multiplier,
        );
        validate_optional_reaction_color(
            &mut errors,
            &label,
            "key_light_color",
            self.key_light_color,
            2.0,
        );
        validate_nonnegative_finite_multiplier(
            &mut errors,
            &label,
            "key_light_intensity_multiplier",
            self.key_light_intensity_multiplier,
        );
        validate_optional_reaction_color(
            &mut errors,
            &label,
            "particle_color",
            self.particle_color,
            2.0,
        );
        if !self.particle_speed_multiplier.is_finite() {
            errors.push(format!(
                "{} particle_speed_multiplier must be finite",
                label
            ));
        }
        if self
            .wind
            .is_some_and(|wind| !wind.iter().all(|value| value.is_finite()))
        {
            errors.push(format!("{} wind must contain finite numbers", label));
        }
        validate_nonnegative_finite_multiplier(
            &mut errors,
            &label,
            "ambience_volume_multiplier",
            self.ambience_volume_multiplier,
        );

        errors
    }
}

fn validate_optional_reaction_color(
    errors: &mut Vec<String>,
    label: &str,
    field: &str,
    color: Option<[f32; 3]>,
    max: f32,
) {
    if color.is_some_and(|color| !finite_color_in_range(color, 0.0, max)) {
        errors.push(format!(
            "{} {} must contain finite values between 0 and {}",
            label, field, max
        ));
    }
}

fn validate_nonnegative_finite_multiplier(
    errors: &mut Vec<String>,
    label: &str,
    field: &str,
    value: f32,
) {
    if !value.is_finite() || value < 0.0 {
        errors.push(format!(
            "{} {} must be finite and non-negative",
            label, field
        ));
    }
}

fn finite_color_in_range(color: [f32; 3], min: f32, max: f32) -> bool {
    color
        .iter()
        .all(|value| value.is_finite() && (min..=max).contains(value))
}

fn validate_atmosphere_range(
    errors: &mut Vec<String>,
    field: &str,
    value: f32,
    min: f32,
    max: f32,
) {
    if !value.is_finite() || !(min..=max).contains(&value) {
        errors.push(format!(
            "atmosphere {} must be between {} and {}",
            field, min, max
        ));
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
        dialogue_ids: &std::collections::HashSet<String>,
        mountain_reaction_ids: &std::collections::HashSet<String>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("event {} ('{}')", index, self.id);
        validate_authoring_id(&label, &self.id, &mut errors);
        errors.extend(self.trigger.validation_errors(&label, prop_ids));
        if !self.once
            && matches!(
                self.trigger.kind,
                LevelEventTriggerKind::OnEnter | LevelEventTriggerKind::Proximity
            )
        {
            errors.push(format!(
                "{} repeatable automatic triggers are unsupported; use Interact or Manual",
                label
            ));
        }
        if self.actions.is_empty() {
            errors.push(format!("{} must contain at least one action", label));
        }
        for (action_index, action) in self.actions.iter().enumerate() {
            errors.extend(action.validation_errors(
                &label,
                action_index,
                loot_table_ids,
                dialogue_ids,
                mountain_reaction_ids,
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
        dialogue_ids: &std::collections::HashSet<String>,
        mountain_reaction_ids: &std::collections::HashSet<String>,
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
        validate_optional_authoring_id(
            &label,
            "reaction_id",
            self.reaction_id.as_deref(),
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
            LevelEventActionKind::ReactMountain => {
                if self.reaction_id.is_none() {
                    errors.push(format!("{} ReactMountain requires reaction_id", label));
                }
                validate_reference(
                    &label,
                    "reaction_id",
                    self.reaction_id.as_deref(),
                    mountain_reaction_ids,
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
        assert_eq!(level.version, CURRENT_LEVEL_VERSION);
    }

    #[test]
    fn legacy_level_json_migrates_to_current_version() {
        let level = LevelData::from_json_str(
            r#"{
                "name": "Legacy",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": []
            }"#,
        )
        .unwrap();

        assert_eq!(level.version, CURRENT_LEVEL_VERSION);
        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn future_level_version_is_rejected_before_runtime() {
        let error = LevelData::from_json_str(
            r#"{
                "version": 2,
                "name": "Future",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": []
            }"#,
        )
        .unwrap_err();

        assert!(error.contains("newer than supported version"));
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
    fn validation_requires_unique_authoring_safe_anchor_ids() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Anchor Identity Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    { "asset_id": "Cube.obj", "anchor_id": "same_anchor" },
                    { "asset_id": "Cube.obj", "anchor_id": "same_anchor" },
                    { "asset_id": "Cube.obj", "anchor_id": "unsafe anchor" }
                ]
            }
            "#,
        )
        .unwrap();

        let errors = level.validation_errors();
        assert!(errors
            .iter()
            .any(|error| error.contains("duplicate anchor id 'same_anchor'")));
        assert!(errors
            .iter()
            .any(|error| error.contains("anchor_id 'unsafe anchor' must use only")));
    }

    #[test]
    fn prop_events_are_reserved_for_manual_enemy_or_anchor_consequences() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Prop Event Contract Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    { "id": "stone", "asset_id": "Cube.obj", "event_id": "stone_event" }
                ],
                "events": [
                    {
                        "id": "stone_event",
                        "once": true,
                        "trigger": { "kind": "Proximity" },
                        "actions": [{ "kind": "GrantResource", "resource_value": 1 }]
                    }
                ]
            }
            "#,
        )
        .unwrap();

        let errors = level.validation_errors();
        assert!(errors
            .iter()
            .any(|error| error.contains("only supported for enemy defeat or Anchor binding")));
        assert!(errors
            .iter()
            .any(|error| error.contains("must reference a Manual event")));
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
    fn authored_loot_sources_require_stable_prop_ids() {
        let prop: PropData = serde_json::from_str(
            r#"{ "asset_id": "Cube.obj", "enemy_type": "Burdened", "loot_table_id": "drop" }"#,
        )
        .unwrap();

        let errors = prop.validation_errors(0);

        assert!(errors
            .iter()
            .any(|error| error.contains("stable id when loot_table_id is set")));
    }

    #[test]
    fn authored_props_cannot_claim_the_runtime_loot_namespace() {
        let prop: PropData =
            serde_json::from_str(r#"{ "id": "runtime_loot_authored", "asset_id": "Cube.obj" }"#)
                .unwrap();

        assert!(prop
            .validation_errors(0)
            .iter()
            .any(|error| error.contains("reserved 'runtime_loot_'")));
    }

    #[test]
    fn event_linked_props_require_stable_ids() {
        let prop: PropData =
            serde_json::from_str(r#"{ "asset_id": "Cube.obj", "event_id": "keeper_fall" }"#)
                .unwrap();

        assert!(prop
            .validation_errors(0)
            .iter()
            .any(|error| error.contains("stable id when event_id is set")));
    }

    #[test]
    fn repeatable_automatic_events_are_rejected_before_they_can_fire_every_frame() {
        let mut level = LevelData::default_level();
        level.events = vec![LevelEventData {
            id: "repeat_proximity".to_string(),
            once: false,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::Proximity,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: None,
            },
            actions: vec![LevelEventActionData {
                kind: LevelEventActionKind::GrantResource,
                target_level_id: None,
                loot_table_id: None,
                dialogue_id: None,
                reaction_id: None,
                flag_id: None,
                resource_value: 1,
                spawn_position: None,
            }],
        }];

        assert!(level
            .validation_errors()
            .iter()
            .any(|error| error.contains("repeatable automatic triggers are unsupported")));
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
    fn ashwalk_first_ascent_keeps_its_playable_loop_contract() {
        let level = LevelData::try_load("levels/ashwalk_01.json").unwrap();

        assert_eq!(level.version, CURRENT_LEVEL_VERSION);
        assert!(level.props.len() <= 14);
        assert_eq!(level.validate(), Ok(()));
        assert_eq!(level.atmosphere.particle_preset, ParticlePreset::Ashfall);
        assert_eq!(level.atmosphere.ambience_preset, AmbiencePreset::AshWind);
        assert_eq!(
            level.base_material.texture.as_deref(),
            Some("cenotaph/ash_stone.png")
        );
        assert_eq!(
            level
                .props
                .iter()
                .filter(|prop| prop.anchor_id.is_some())
                .count(),
            1
        );
        assert_eq!(level.props.iter().filter(|prop| prop.is_hurtbox).count(), 1);
        assert!(
            level
                .props
                .iter()
                .map(|prop| prop.resource_value)
                .sum::<u32>()
                >= 100
        );
        assert!(level
            .props
            .iter()
            .filter_map(|prop| prop.item_id.as_deref())
            .any(|item_id| item_id == "ash_splinter"));
        assert_eq!(level.paths.len(), 1);
        assert!(level
            .props
            .iter()
            .all(|prop| prop.asset_id != "props/test_wall.obj"));
        assert!(
            level
                .props
                .iter()
                .filter(|prop| prop.enemy_type.is_some())
                .count()
                <= 3
        );
        assert!(level.events.iter().any(|event| {
            event.trigger.kind == LevelEventTriggerKind::Interact
                && event.trigger.prop_id.as_deref() == Some("oath_stone")
        }));

        let elite = level
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("ashwarden_elite"))
            .unwrap();
        assert!(elite.enemy_health >= 200.0);
        assert_eq!(elite.loot_table_id.as_deref(), Some("ashwarden_drop"));
        assert!(level
            .props
            .iter()
            .any(|prop| { prop.trigger_level_id.as_deref() == Some("foundation_test") }));
    }

    #[test]
    fn level_save_round_trips_pretty_json() {
        let temp_path = std::env::temp_dir().join(format!(
            "cenotaph_level_save_test_{}_{}.json",
            std::process::id(),
            17
        ));
        let level = LevelData {
            version: CURRENT_LEVEL_VERSION,
            name: "Save Test".to_string(),
            base_map: "assets/test_movement_arena.obj".to_string(),
            player_spawn: [0.0, 128.0, 0.0],
            atmosphere: AtmosphereData::default(),
            base_material: SurfaceMaterialData::default(),
            mountain_reactions: Vec::new(),
            props: vec![PropData {
                id: None,
                display_name: None,
                asset_id: "props/test_wall.obj".to_string(),
                position: [1.0, 2.0, 3.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::Box,
                surface_material: None,
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
                    reaction_id: None,
                    flag_id: None,
                    resource_value: 0,
                    spawn_position: None,
                }],
            }],
            dialogues: vec![DialogueData {
                id: "opening".to_string(),
                speaker: "Cenotaph".to_string(),
                lines: vec!["The cenotaph remembers this room.".to_string()],
            }],
        };

        let json = serde_json::to_string_pretty(&level).unwrap();
        std::fs::write(&temp_path, format!("{}\n", json)).unwrap();
        let loaded = LevelData::try_load(temp_path.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(&temp_path);

        assert_eq!(loaded.name, "Save Test");
        assert_eq!(loaded.version, CURRENT_LEVEL_VERSION);
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
        assert!(level.mountain_reactions.is_empty());
        assert_eq!(level.atmosphere, AtmosphereData::default());
        assert_eq!(level.base_material, SurfaceMaterialData::default());
    }

    #[test]
    fn mountain_reaction_json_parses_defaults_and_particle_reversal() {
        let level = LevelData::from_json_str(
            r#"
            {
                "name": "Mountain Reaction Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [],
                "mountain_reactions": [
                    {
                        "id": "choir_reversal",
                        "particle_speed_multiplier": -1.5
                    },
                    {
                        "id": "ashen_squall",
                        "duration": 2.5,
                        "clear_color": [0.1, 0.2, 0.3],
                        "fog_color": [0.3, 0.2, 0.1],
                        "fog_density_multiplier": 1.5,
                        "key_light_color": [1.5, 1.0, 0.5],
                        "key_light_intensity_multiplier": 2.0,
                        "particle_color": [0.5, 0.75, 1.25],
                        "particle_speed_multiplier": 0.75,
                        "wind": [4.0, -2.0, 1.0],
                        "ambience_preset": "AshWind",
                        "ambience_volume_multiplier": 0.4
                    }
                ],
                "events": [
                    {
                        "id": "mountain_answers",
                        "trigger": { "kind": "Manual" },
                        "actions": [
                            {
                                "kind": "ReactMountain",
                                "reaction_id": "choir_reversal"
                            }
                        ]
                    }
                ]
            }
            "#,
        )
        .unwrap();

        let reaction = &level.mountain_reactions[0];
        assert_eq!(reaction.duration, 4.0);
        assert_eq!(reaction.clear_color, None);
        assert_eq!(reaction.fog_color, None);
        assert_eq!(reaction.fog_density_multiplier, 1.0);
        assert_eq!(reaction.key_light_color, None);
        assert_eq!(reaction.key_light_intensity_multiplier, 1.0);
        assert_eq!(reaction.particle_color, None);
        assert_eq!(reaction.particle_speed_multiplier, -1.5);
        assert_eq!(reaction.wind, None);
        assert_eq!(reaction.ambience_preset, None);
        assert_eq!(reaction.ambience_volume_multiplier, 1.0);
        assert_eq!(
            level.mountain_reactions[1],
            MountainReactionData {
                id: "ashen_squall".to_string(),
                duration: 2.5,
                clear_color: Some([0.1, 0.2, 0.3]),
                fog_color: Some([0.3, 0.2, 0.1]),
                fog_density_multiplier: 1.5,
                key_light_color: Some([1.5, 1.0, 0.5]),
                key_light_intensity_multiplier: 2.0,
                particle_color: Some([0.5, 0.75, 1.25]),
                particle_speed_multiplier: 0.75,
                wind: Some([4.0, -2.0, 1.0]),
                ambience_preset: Some(AmbiencePreset::AshWind),
                ambience_volume_multiplier: 0.4,
            }
        );
        assert_eq!(
            level.events[0].actions[0].reaction_id.as_deref(),
            Some("choir_reversal")
        );
        assert_eq!(level.validate(), Ok(()));
    }

    #[test]
    fn mountain_reaction_validation_rejects_invalid_profiles() {
        let mut level = LevelData::default_level();
        level.mountain_reactions = vec![
            MountainReactionData {
                id: "bad reaction".to_string(),
                duration: 0.0,
                clear_color: Some([-0.1, 0.0, 0.0]),
                fog_color: Some([f32::NAN, 0.0, 0.0]),
                fog_density_multiplier: -1.0,
                key_light_color: Some([3.0, 0.0, 0.0]),
                key_light_intensity_multiplier: f32::INFINITY,
                particle_color: Some([0.0, 0.0, 3.0]),
                particle_speed_multiplier: f32::NAN,
                wind: Some([0.0, f32::INFINITY, 0.0]),
                ambience_preset: None,
                ambience_volume_multiplier: -0.5,
            },
            MountainReactionData {
                id: "bad reaction".to_string(),
                ..MountainReactionData::default()
            },
        ];

        let errors = level.validation_errors();
        for field in [
            "duration",
            "clear_color",
            "fog_color",
            "fog_density_multiplier",
            "key_light_color",
            "key_light_intensity_multiplier",
            "particle_color",
            "particle_speed_multiplier",
            "wind",
            "ambience_volume_multiplier",
        ] {
            assert!(
                errors.iter().any(|error| error.contains(field)),
                "missing validation error for {field}: {errors:?}"
            );
        }
        assert!(errors
            .iter()
            .any(|error| error.contains("duplicate mountain reaction id")));
        assert!(errors
            .iter()
            .any(|error| error.contains("must use only letters")));
    }

    #[test]
    fn react_mountain_actions_require_declared_reaction_ids() {
        let level = LevelData::from_json_str(
            r#"
            {
                "name": "Broken Mountain Reactions",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [],
                "events": [
                    {
                        "id": "missing_reaction_id",
                        "trigger": { "kind": "Manual" },
                        "actions": [{ "kind": "ReactMountain" }]
                    },
                    {
                        "id": "unknown_reaction_id",
                        "trigger": { "kind": "Manual" },
                        "actions": [
                            {
                                "kind": "ReactMountain",
                                "reaction_id": "undeclared_reaction"
                            }
                        ]
                    }
                ]
            }
            "#,
        )
        .unwrap();

        let errors = level.validation_errors();
        assert!(errors
            .iter()
            .any(|error| error.contains("ReactMountain requires reaction_id")));
        assert!(errors.iter().any(|error| {
            error.contains("reaction_id references unknown id 'undeclared_reaction'")
        }));
    }

    #[test]
    fn atmosphere_and_surface_material_validation_guard_runtime_budgets() {
        let atmosphere = AtmosphereData {
            particle_count: 513,
            wind: [f32::INFINITY, 0.0, 0.0],
            ..AtmosphereData::default()
        };
        let errors = atmosphere.validation_errors();
        assert!(errors.iter().any(|error| error.contains("particle_count")));
        assert!(errors.iter().any(|error| error.contains("wind")));

        let material = SurfaceMaterialData {
            texture: Some("../Cargo.toml".to_string()),
            uv_scale: 0.0,
            emissive: 5.0,
            ..SurfaceMaterialData::default()
        };
        let errors = material.validation_errors("test material");
        assert!(errors.iter().any(|error| error.contains("safe path")));
        assert!(errors.iter().any(|error| error.contains("uv_scale")));
        assert!(errors.iter().any(|error| error.contains("emissive")));
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
            notes: Some("source-only authoring import".to_string()),
        };

        assert!(import.validation_errors(0).is_empty());
    }

    #[test]
    fn validation_accepts_custom_brush_geometry_without_asset_file() {
        let level = LevelData::from_json_str(
            r#"
            {
                "name": "Brush Geometry Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    {
                        "asset_id": "generated/brush_geometry",
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
                        "asset_id": "generated/brush_geometry",
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
            version: CURRENT_LEVEL_VERSION,
            name: "Authoring Graph".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: AtmosphereData::default(),
            base_material: SurfaceMaterialData::default(),
            mountain_reactions: Vec::new(),
            props: vec![PropData {
                id: Some("guard_01".to_string()),
                display_name: None,
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::Sphere,
                surface_material: None,
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
                    kind: LevelEventTriggerKind::Manual,
                    position: [0.0, 0.0, 0.0],
                    radius: 2.5,
                    prop_id: None,
                    flag_id: None,
                },
                actions: vec![LevelEventActionData {
                    kind: LevelEventActionKind::StartDialogue,
                    target_level_id: None,
                    loot_table_id: None,
                    dialogue_id: Some("guard_dialogue".to_string()),
                    reaction_id: None,
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
            version: CURRENT_LEVEL_VERSION,
            name: "Broken Graph".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: AtmosphereData::default(),
            base_material: SurfaceMaterialData::default(),
            mountain_reactions: Vec::new(),
            props: vec![PropData {
                id: Some("bad prop".to_string()),
                display_name: None,
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::None,
                surface_material: None,
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
                    reaction_id: None,
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

//! Versioned JSON schema, migration, and validation for playable levels.
//!
//! Authored references are validated here before runtime systems materialize
//! enemies, collision, events, loot, dialogue, and atmosphere.

use std::fs;
use std::path::Path;

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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PropData {
    /// Optional stable authoring ID used by tooling and level events.
    #[serde(default)]
    pub id: Option<String>,
    /// Optional authored name used by ritual and named-encounter HUD surfaces.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Filename of the mesh asset (relative to `assets/`).
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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelData {
    /// Version of the authored level contract.
    #[serde(default = "legacy_level_version")]
    pub version: u32,
    /// Human-readable name of the level for display purposes
    pub name: String,
    /// Path to the base map mesh (e.g. `"assets/map_001.glb"`).
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

        let mountain_reaction_ids = validation::collect_ids(
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
                validation::collect_unique_id("prop", index, id, &mut prop_ids, &mut errors);
            }
            if let Some(anchor_id) = prop.anchor_id.as_deref() {
                validation::collect_unique_id("anchor", index, anchor_id, &mut anchor_ids, &mut errors);
            }
        }

        let _asset_import_ids = validation::collect_ids(
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

        let loot_table_ids = validation::collect_ids(
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

        let path_ids = validation::collect_ids(
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

        let event_ids = validation::collect_ids(
            "event",
            self.events
                .iter()
                .enumerate()
                .map(|(index, event)| (index, event.id.as_str())),
            &mut errors,
        );
        let dialogue_ids = validation::collect_ids(
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
            validation::validate_reference(
                &label,
                "loot_table_id",
                prop.loot_table_id.as_deref(),
                &loot_table_ids,
                &mut errors,
            );
            validation::validate_reference(
                &label,
                "path_id",
                prop.path_id.as_deref(),
                &path_ids,
                &mut errors,
            );
            validation::validate_reference(
                &label,
                "dialogue_id",
                prop.dialogue_id.as_deref(),
                &dialogue_ids,
                &mut errors,
            );
            validation::validate_reference(
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

mod validation;

#[cfg(test)]
mod tests;

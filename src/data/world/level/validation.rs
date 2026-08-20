//! Authoring-time validation methods for level data types.
//!
//! Every type that participates in authored content validates itself through a
//! `validation_errors()` method so the core level schema stays focused on
//! definition, migration, and construction.

use std::path::{Component, Path};

use super::{
    BrushGeometryData, ColliderType, DialogueData, LevelEventActionData,
    LevelEventActionKind, LevelEventData, LevelEventTriggerData, LevelEventTriggerKind,
    LevelPathData, LootEntryData, LootTableData, MountainReactionData, PropData,
    RUNTIME_LOOT_ID_PREFIX, SurfaceMaterialData, AssetImportData, AtmosphereData,
};

pub(super) fn collect_ids<'a>(
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

pub(super) fn collect_unique_id(
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

pub(super) fn validate_reference(
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

pub(super) fn validate_authoring_id(label: &str, id: &str, errors: &mut Vec<String>) {
    if id.trim().is_empty() {
        errors.push(format!("{} id must not be empty", label));
    } else if !is_authoring_id(id) {
        errors.push(format!(
            "{} id '{}' must use only letters, numbers, '_' or '-'",
            label, id
        ));
    }
}

pub(super) fn validate_optional_authoring_id(
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

pub(super) fn is_authoring_id(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
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

pub(super) fn finite_color_in_range(color: [f32; 3], min: f32, max: f32) -> bool {
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
            if let Err(error) = super::validate_level_id(target) {
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
                if let Err(error) = super::validate_level_id(target) {
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

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use crate::data::config::gameplay::{parse_key, GameConfig};
use crate::data::enemy::{normalize_enemy_id, EnemyDefinition};
use crate::data::relic::{normalize_relic_id, RelicDefinition};
use crate::data::world::level::LevelData;
use crate::systems::render::mesh::{try_load_model, ModelData};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
}

impl ValidationIssue {
    fn new(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContentValidationReport {
    pub checked_levels: usize,
    pub checked_configs: usize,
    pub checked_assets: usize,
    pub checked_enemy_definitions: usize,
    pub checked_relic_definitions: usize,
    pub issues: Vec<ValidationIssue>,
}

impl ContentValidationReport {
    pub fn is_ok(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn summary(&self) -> String {
        if self.is_ok() {
            format!(
                "content validation passed: {} level file(s), {} config file(s), {} enemy definition file(s), {} relic definition file(s), {} asset file(s) checked",
                self.checked_levels,
                self.checked_configs,
                self.checked_enemy_definitions,
                self.checked_relic_definitions,
                self.checked_assets
            )
        } else {
            format!(
                "content validation failed: {} issue(s) across {} level file(s), {} config file(s), {} enemy definition file(s), {} relic definition file(s), {} asset file(s)",
                self.issues.len(),
                self.checked_levels,
                self.checked_configs,
                self.checked_enemy_definitions,
                self.checked_relic_definitions,
                self.checked_assets
            )
        }
    }

    fn add_issue(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.issues.push(ValidationIssue::new(path, message));
    }
}

impl fmt::Display for ContentValidationReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.summary())?;
        for issue in &self.issues {
            writeln!(f, "- {}: {}", issue.path, issue.message)?;
        }
        Ok(())
    }
}

pub fn validate_project_content() -> ContentValidationReport {
    let mut report = ContentValidationReport::default();
    let mut checked_asset_paths = HashSet::new();
    let enemy_ids =
        validate_enemy_definitions_dir("data/enemies", &mut report, &mut checked_asset_paths);
    let relic_ids = validate_relic_definitions_dir("data/relics", &mut report);
    validate_levels_dir_into(
        "levels",
        Some(&enemy_ids),
        Some(&relic_ids),
        &mut report,
        &mut checked_asset_paths,
    );
    validate_all_model_assets_dir("assets", &mut report, &mut checked_asset_paths);
    validate_tuning_file("config/tuning.toml", &mut report);
    validate_bindings_file("config/bindings.toml", &mut report);
    report
}

#[cfg(test)]
fn validate_levels_dir(levels_dir: impl AsRef<Path>) -> ContentValidationReport {
    let mut report = ContentValidationReport::default();
    let mut checked_asset_paths = HashSet::new();
    validate_levels_dir_into(
        levels_dir,
        None,
        None,
        &mut report,
        &mut checked_asset_paths,
    );
    report
}

fn validate_levels_dir_into(
    levels_dir: impl AsRef<Path>,
    known_enemy_ids: Option<&HashSet<String>>,
    known_relic_ids: Option<&HashSet<String>>,
    report: &mut ContentValidationReport,
    checked_asset_paths: &mut HashSet<PathBuf>,
) {
    let levels_dir = levels_dir.as_ref();

    let entries = match std::fs::read_dir(levels_dir) {
        Ok(entries) => entries,
        Err(e) => {
            report.add_issue(
                levels_dir.to_string_lossy(),
                format!("failed to read levels directory: {}", e),
            );
            return;
        }
    };

    let mut level_paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        })
        .collect();
    level_paths.sort();

    if level_paths.is_empty() {
        report.add_issue(
            levels_dir.to_string_lossy(),
            "no level JSON files found in levels directory",
        );
        return;
    }

    for level_path in level_paths {
        let path_label = level_path.to_string_lossy().to_string();
        report.checked_levels += 1;

        let level = match LevelData::try_load(&path_label) {
            Ok(level) => level,
            Err(e) => {
                report.add_issue(path_label, e);
                continue;
            }
        };

        for error in level.validation_errors() {
            report.add_issue(path_label.clone(), error);
        }
        if let Some(enemy_ids) = known_enemy_ids {
            validate_level_enemy_references(&level, &path_label, enemy_ids, report);
        }
        if let Some(relic_ids) = known_relic_ids {
            validate_level_item_references(&level, &path_label, relic_ids, report);
        }

        for asset_path in referenced_model_assets(&level) {
            if asset_path.exists() {
                validate_model_asset_once(&asset_path, report, checked_asset_paths);
            }
        }
    }
}

fn validate_enemy_definitions_dir(
    enemies_dir: impl AsRef<Path>,
    report: &mut ContentValidationReport,
    checked_asset_paths: &mut HashSet<PathBuf>,
) -> HashSet<String> {
    let enemies_dir = enemies_dir.as_ref();
    let mut enemy_ids = HashSet::new();

    let entries = match std::fs::read_dir(enemies_dir) {
        Ok(entries) => entries,
        Err(e) => {
            report.add_issue(
                enemies_dir.to_string_lossy(),
                format!("failed to read enemy definitions directory: {}", e),
            );
            return enemy_ids;
        }
    };

    let mut enemy_paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        })
        .collect();
    enemy_paths.sort();

    if enemy_paths.is_empty() {
        report.add_issue(
            enemies_dir.to_string_lossy(),
            "no enemy TOML files found in enemy definitions directory",
        );
        return enemy_ids;
    }

    for enemy_path in enemy_paths {
        let path_label = enemy_path.to_string_lossy().to_string();
        report.checked_enemy_definitions += 1;

        let enemy = match EnemyDefinition::try_load(&enemy_path) {
            Ok(enemy) => enemy,
            Err(e) => {
                report.add_issue(path_label, e);
                continue;
            }
        };

        for error in enemy.validation_errors() {
            report.add_issue(path_label.clone(), error);
        }

        let normalized_id = normalize_enemy_id(&enemy.id);
        if normalized_id.is_empty() {
            continue;
        }
        if !enemy_ids.insert(normalized_id.clone()) {
            report.add_issue(
                path_label.clone(),
                format!("duplicate enemy id '{}'", normalized_id),
            );
        }

        let model_path = enemy.model_path();
        if model_path.exists() {
            validate_model_asset_once(&model_path, report, checked_asset_paths);
        } else if !enemy.model_asset.trim().is_empty() {
            report.add_issue(
                path_label,
                format!(
                    "enemy model_asset references missing asset '{}'",
                    model_path.to_string_lossy()
                ),
            );
        }
    }

    enemy_ids
}

fn validate_all_model_assets_dir(
    assets_dir: impl AsRef<Path>,
    report: &mut ContentValidationReport,
    checked_asset_paths: &mut HashSet<PathBuf>,
) {
    let assets_dir = assets_dir.as_ref();
    let mut model_paths = Vec::new();
    collect_model_asset_paths(assets_dir, report, &mut model_paths);
    model_paths.sort();

    for model_path in model_paths {
        validate_model_asset_once(&model_path, report, checked_asset_paths);
    }
}

fn collect_model_asset_paths(
    path: &Path,
    report: &mut ContentValidationReport,
    model_paths: &mut Vec<PathBuf>,
) {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            report.add_issue(
                path.to_string_lossy(),
                format!("failed to read asset directory: {}", e),
            );
            return;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_model_asset_paths(&path, report, model_paths);
        } else if has_model_asset_extension(&path) {
            model_paths.push(path);
        }
    }
}

fn has_model_asset_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "obj" | "glb" | "gltf"))
}

fn validate_relic_definitions_dir(
    relics_dir: impl AsRef<Path>,
    report: &mut ContentValidationReport,
) -> HashSet<String> {
    let relics_dir = relics_dir.as_ref();
    let mut relic_ids = HashSet::new();

    let entries = match std::fs::read_dir(relics_dir) {
        Ok(entries) => entries,
        Err(e) => {
            report.add_issue(
                relics_dir.to_string_lossy(),
                format!("failed to read relic definitions directory: {}", e),
            );
            return relic_ids;
        }
    };

    let mut relic_paths: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            path.extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("toml"))
        })
        .collect();
    relic_paths.sort();

    if relic_paths.is_empty() {
        report.add_issue(
            relics_dir.to_string_lossy(),
            "no relic TOML files found in relic definitions directory",
        );
        return relic_ids;
    }

    for relic_path in relic_paths {
        let path_label = relic_path.to_string_lossy().to_string();
        report.checked_relic_definitions += 1;

        let relic = match RelicDefinition::try_load(&relic_path) {
            Ok(relic) => relic,
            Err(e) => {
                report.add_issue(path_label, e);
                continue;
            }
        };

        for error in relic.validation_errors() {
            report.add_issue(path_label.clone(), error);
        }

        let normalized_id = normalize_relic_id(&relic.id);
        if normalized_id.is_empty() {
            continue;
        }
        if !relic_ids.insert(normalized_id.clone()) {
            report.add_issue(
                path_label.clone(),
                format!("duplicate relic id '{}'", normalized_id),
            );
        }
    }

    relic_ids
}

fn validate_level_enemy_references(
    level: &LevelData,
    path_label: &str,
    enemy_ids: &HashSet<String>,
    report: &mut ContentValidationReport,
) {
    for (index, prop) in level.props.iter().enumerate() {
        let Some(enemy_type) = prop.enemy_type.as_ref() else {
            continue;
        };
        let normalized = normalize_enemy_id(enemy_type);
        if normalized.is_empty() {
            report.add_issue(
                path_label.to_string(),
                format!("prop {} enemy_type must not be empty", index),
            );
        } else if !enemy_ids.contains(&normalized) {
            report.add_issue(
                path_label.to_string(),
                format!(
                    "prop {} enemy_type '{}' has no matching enemy definition",
                    index, enemy_type
                ),
            );
        }
    }
}

fn validate_level_item_references(
    level: &LevelData,
    path_label: &str,
    relic_ids: &HashSet<String>,
    report: &mut ContentValidationReport,
) {
    for (index, prop) in level.props.iter().enumerate() {
        let Some(item_id) = prop.item_id.as_ref() else {
            continue;
        };
        let normalized = normalize_relic_id(item_id);
        if normalized.is_empty() {
            report.add_issue(
                path_label.to_string(),
                format!("prop {} item_id must not be empty", index),
            );
        } else if !relic_ids.contains(&normalized) {
            report.add_issue(
                path_label.to_string(),
                format!(
                    "prop {} item_id '{}' has no matching relic definition",
                    index, item_id
                ),
            );
        }
    }
}

fn referenced_model_assets(level: &LevelData) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(level.props.len() + 1);
    if !level.base_map.trim().is_empty() {
        paths.push(PathBuf::from(&level.base_map));
    }
    for prop in &level.props {
        if !prop.asset_id.trim().is_empty() {
            paths.push(PathBuf::from("assets").join(&prop.asset_id));
        }
    }
    paths
}

fn validate_model_asset_once(
    path: &Path,
    report: &mut ContentValidationReport,
    checked_asset_paths: &mut HashSet<PathBuf>,
) {
    let key = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if checked_asset_paths.insert(key) {
        validate_model_asset_file(path, report);
    }
}

fn validate_model_asset_file(path: &Path, report: &mut ContentValidationReport) {
    let path_label = path.to_string_lossy().to_string();
    report.checked_assets += 1;

    let Some(extension) = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
    else {
        report.add_issue(path_label, "model asset has no file extension");
        return;
    };

    if !matches!(extension.as_str(), "obj" | "glb" | "gltf") {
        report.add_issue(
            path_label,
            format!("unsupported model asset extension '.{}'", extension),
        );
        return;
    }

    let Some(path_str) = path.to_str() else {
        report.add_issue(path_label, "model asset path is not valid UTF-8");
        return;
    };

    let model = match try_load_model(path_str) {
        Ok(model) => model,
        Err(e) => {
            report.add_issue(path_label, format!("failed to load model asset: {}", e));
            return;
        }
    };

    for issue in validate_model_geometry(&model) {
        report.add_issue(path_label.clone(), issue);
    }
}

fn validate_model_geometry(model: &ModelData) -> Vec<String> {
    let (vertices, parts, phys_points, phys_indices) = model;
    let mut issues = Vec::new();

    if vertices.is_empty() {
        issues.push("model contains no vertices".to_string());
    }
    if parts.is_empty() {
        issues.push("model contains no render mesh parts".to_string());
    }

    let total_indices: usize = parts.iter().map(|part| part.indices.len()).sum();
    if total_indices == 0 {
        issues.push("model contains no render indices".to_string());
    }
    if !total_indices.is_multiple_of(3) {
        issues.push("model render index count should be divisible by 3".to_string());
    }

    for (vertex_index, vertex) in vertices.iter().enumerate() {
        if !vertex.position.iter().all(|value| value.is_finite()) {
            issues.push(format!(
                "vertex {} position must contain finite numbers",
                vertex_index
            ));
            break;
        }
        if !vertex.tex_coords.iter().all(|value| value.is_finite()) {
            issues.push(format!(
                "vertex {} texture coordinates must contain finite numbers",
                vertex_index
            ));
            break;
        }
        if !vertex.normal.iter().all(|value| value.is_finite()) {
            issues.push(format!(
                "vertex {} normal must contain finite numbers",
                vertex_index
            ));
            break;
        }
    }

    let vertex_count = vertices.len() as u32;
    if vertex_count > 0 {
        for (part_index, part) in parts.iter().enumerate() {
            if part.indices.iter().any(|index| *index >= vertex_count) {
                issues.push(format!(
                    "render part {} contains an index outside the vertex buffer",
                    part_index
                ));
                break;
            }
        }
    }

    if phys_points.is_empty() {
        issues.push("model contains no physics points".to_string());
    }
    if phys_indices.is_empty() {
        issues.push("model contains no physics triangles".to_string());
    }

    issues
}

fn validate_tuning_file(path: impl AsRef<Path>, report: &mut ContentValidationReport) {
    let path = path.as_ref();
    let path_label = path.to_string_lossy().to_string();
    report.checked_configs += 1;

    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            report.add_issue(path_label, format!("failed to read tuning file: {}", e));
            return;
        }
    };

    let config: GameConfig = match toml::from_str(&data) {
        Ok(config) => config,
        Err(e) => {
            report.add_issue(path_label, format!("failed to parse tuning TOML: {}", e));
            return;
        }
    };

    for issue in validate_tuning_values(&config) {
        report.add_issue(path_label.clone(), issue);
    }
}

fn validate_tuning_values(config: &GameConfig) -> Vec<String> {
    let mut issues = Vec::new();

    require_positive("player.max_health", config.player.max_health, &mut issues);
    require_non_negative("player.max_stamina", config.player.max_stamina, &mut issues);
    require_non_negative(
        "player.stamina_regen_rate",
        config.player.stamina_regen_rate,
        &mut issues,
    );
    require_non_negative(
        "player.stamina_regen_delay",
        config.player.stamina_regen_delay,
        &mut issues,
    );
    require_positive("player.walk_speed", config.player.walk_speed, &mut issues);
    require_positive(
        "player.sprint_speed",
        config.player.sprint_speed,
        &mut issues,
    );
    if config.player.sprint_speed < config.player.walk_speed {
        issues.push("player.sprint_speed should be >= player.walk_speed".to_string());
    }

    require_positive(
        "movement.dash_speed_multiplier",
        config.movement.dash_speed_multiplier,
        &mut issues,
    );
    require_non_negative(
        "movement.sprint_stamina_drain_rate",
        config.movement.sprint_stamina_drain_rate,
        &mut issues,
    );
    require_non_negative(
        "movement.dash_stamina_cost",
        config.movement.dash_stamina_cost,
        &mut issues,
    );
    require_non_negative(
        "movement.dash_cooldown",
        config.movement.dash_cooldown,
        &mut issues,
    );
    require_non_negative(
        "movement.dash_duration",
        config.movement.dash_duration,
        &mut issues,
    );

    require_positive("camera.sensitivity", config.camera.sensitivity, &mut issues);
    require_finite("physics.gravity", config.physics.gravity, &mut issues);
    require_finite(
        "physics.jump_velocity",
        config.physics.jump_velocity,
        &mut issues,
    );
    require_positive(
        "physics.player_speed",
        config.physics.player_speed,
        &mut issues,
    );

    require_non_negative("combat.base_damage", config.combat.base_damage, &mut issues);
    require_positive(
        "combat.crit_multiplier",
        config.combat.crit_multiplier,
        &mut issues,
    );
    require_positive(
        "combat.primary_fire_range",
        config.combat.primary_fire_range,
        &mut issues,
    );
    require_non_negative(
        "combat.attack_cooldown",
        config.combat.attack_cooldown,
        &mut issues,
    );
    require_non_negative(
        "combat.miss_cooldown",
        config.combat.miss_cooldown,
        &mut issues,
    );
    require_positive(
        "combat.enemy_hit_radius",
        config.combat.enemy_hit_radius,
        &mut issues,
    );
    require_non_negative(
        "combat.enemy_hit_stun",
        config.combat.enemy_hit_stun,
        &mut issues,
    );
    require_non_negative(
        "combat.hurtbox_damage_per_second",
        config.combat.hurtbox_damage_per_second,
        &mut issues,
    );
    require_positive(
        "combat.hurtbox_radius",
        config.combat.hurtbox_radius,
        &mut issues,
    );
    require_non_negative(
        "combat.hurtbox_tick_interval",
        config.combat.hurtbox_tick_interval,
        &mut issues,
    );
    require_non_negative(
        "combat.respawn_delay",
        config.combat.respawn_delay,
        &mut issues,
    );

    require_positive(
        "world.draw_distance",
        config.world.draw_distance,
        &mut issues,
    );
    require_non_negative("world.fog_density", config.world.fog_density, &mut issues);
    require_color(
        "lighting.ambient_color",
        config.lighting.ambient_color,
        &mut issues,
    );
    require_color("lighting.sun_color", config.lighting.sun_color, &mut issues);
    require_non_negative(
        "lighting.sun_intensity",
        config.lighting.sun_intensity,
        &mut issues,
    );
    require_finite(
        "lighting.sun_position_offset",
        config.lighting.sun_position_offset,
        &mut issues,
    );
    require_positive(
        "debug.position_log_interval",
        config.debug.position_log_interval,
        &mut issues,
    );

    issues
}

fn validate_bindings_file(path: impl AsRef<Path>, report: &mut ContentValidationReport) {
    let path = path.as_ref();
    let path_label = path.to_string_lossy().to_string();
    report.checked_configs += 1;

    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) => {
            report.add_issue(path_label, format!("failed to read bindings file: {}", e));
            return;
        }
    };

    let raw: toml::Value = match toml::from_str(&data) {
        Ok(raw) => raw,
        Err(e) => {
            report.add_issue(path_label, format!("failed to parse bindings TOML: {}", e));
            return;
        }
    };

    let Some(bindings) = raw.get("keybindings").and_then(|value| value.as_table()) else {
        report.add_issue(path_label, "missing [keybindings] table");
        return;
    };

    let required_actions = [
        "forward",
        "backward",
        "left",
        "right",
        "jump",
        "sprint",
        "dash",
        "attack",
        "interact",
        "inventory",
        "pause",
    ];

    for action in required_actions {
        if !bindings.contains_key(action) {
            report.add_issue(path_label.clone(), format!("missing binding '{}'", action));
        }
    }

    let mut seen_tokens: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (action, value) in bindings {
        let Some(token) = value.as_str() else {
            report.add_issue(
                path_label.clone(),
                format!("binding '{}' must be a string", action),
            );
            continue;
        };

        if !is_valid_binding_token(token) {
            report.add_issue(
                path_label.clone(),
                format!("binding '{}' uses unknown key '{}'", action, token),
            );
            continue;
        }

        let normalized = token.to_ascii_uppercase();
        if is_unbound_token(&normalized) {
            continue;
        }
        if let Some(previous_action) = seen_tokens.insert(normalized.clone(), action.clone()) {
            report.add_issue(
                path_label.clone(),
                format!(
                    "bindings '{}' and '{}' both use '{}'",
                    previous_action, action, normalized
                ),
            );
        }
    }
}

fn is_valid_binding_token(token: &str) -> bool {
    let token = token.to_ascii_uppercase();
    parse_key(&token).is_some()
        || matches!(
            token.as_str(),
            "MOUSE_LEFT" | "MOUSE_RIGHT" | "MOUSE_MIDDLE" | "NONE" | "UNBOUND"
        )
}

fn is_unbound_token(token: &str) -> bool {
    matches!(token, "NONE" | "UNBOUND")
}

fn require_finite(name: &str, value: f32, issues: &mut Vec<String>) {
    if !value.is_finite() {
        issues.push(format!("{} must be finite", name));
    }
}

fn require_positive(name: &str, value: f32, issues: &mut Vec<String>) {
    require_finite(name, value, issues);
    if value <= 0.0 {
        issues.push(format!("{} must be > 0", name));
    }
}

fn require_non_negative(name: &str, value: f32, issues: &mut Vec<String>) {
    require_finite(name, value, issues);
    if value < 0.0 {
        issues.push(format!("{} must be >= 0", name));
    }
}

fn require_color(name: &str, color: [f32; 3], issues: &mut Vec<String>) {
    for (index, component) in color.iter().enumerate() {
        require_non_negative(&format!("{}[{}]", name, index), *component, issues);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_levels_directory_validates() {
        let report = validate_levels_dir("levels");
        assert!(report.is_ok(), "{}", report);
        assert!(report.checked_levels >= 2);
        assert!(report.checked_assets >= 3);
    }

    #[test]
    fn current_project_content_validates() {
        let report = validate_project_content();
        assert!(report.is_ok(), "{}", report);
        assert!(report.checked_levels >= 2);
        assert_eq!(report.checked_configs, 2);
        assert!(report.checked_enemy_definitions >= 5);
        assert!(report.checked_relic_definitions >= 1);
        assert!(report.checked_assets >= 3);
    }

    #[test]
    fn project_validation_checks_every_model_asset() {
        let mut collection_report = ContentValidationReport::default();
        let mut model_paths = Vec::new();
        collect_model_asset_paths(
            std::path::Path::new("assets"),
            &mut collection_report,
            &mut model_paths,
        );
        assert!(collection_report.is_ok(), "{}", collection_report);

        let report = validate_project_content();
        assert!(report.is_ok(), "{}", report);
        assert_eq!(report.checked_assets, model_paths.len());
    }

    #[test]
    fn reports_malformed_level_json() {
        let dir =
            std::env::temp_dir().join(format!("cenotaph_validation_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let broken_level = dir.join("broken.json");
        std::fs::write(&broken_level, "{ definitely not valid json").unwrap();

        let report = validate_levels_dir(&dir);
        assert!(!report.is_ok());
        assert_eq!(report.checked_levels, 1);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("failed to parse level JSON")));

        let _ = std::fs::remove_file(broken_level);
        let _ = std::fs::remove_dir(dir);
    }

    #[test]
    fn reports_invalid_tuning_values() {
        let mut config = GameConfig::default();
        config.player.max_health = -1.0;
        config.player.sprint_speed = 1.0;
        config.player.walk_speed = 2.0;
        config.combat.enemy_hit_radius = 0.0;
        config.debug.position_log_interval = 0.0;

        let issues = validate_tuning_values(&config);
        assert!(issues
            .iter()
            .any(|issue| issue.contains("player.max_health")));
        assert!(issues.iter().any(|issue| issue.contains("sprint_speed")));
        assert!(issues
            .iter()
            .any(|issue| issue.contains("combat.enemy_hit_radius")));
        assert!(issues
            .iter()
            .any(|issue| issue.contains("debug.position_log_interval")));
    }

    #[test]
    fn reports_unknown_level_enemy_type() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Enemy Reference Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    {
                        "asset_id": "Cube.obj",
                        "enemy_type": "NotReal",
                        "enemy_health": 1.0
                    }
                ]
            }
            "#,
        )
        .unwrap();
        let enemy_ids = HashSet::from(["burdened".to_string()]);
        let mut report = ContentValidationReport::default();

        validate_level_enemy_references(&level, "test_level", &enemy_ids, &mut report);

        assert!(!report.is_ok());
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("no matching enemy definition")));
    }

    #[test]
    fn reports_unknown_level_item_id() {
        let level: LevelData = serde_json::from_str(
            r#"
            {
                "name": "Item Reference Test",
                "base_map": "assets/Cube.obj",
                "player_spawn": [0.0, 0.0, 0.0],
                "props": [
                    {
                        "asset_id": "Cube.obj",
                        "item_id": "NotReal"
                    }
                ]
            }
            "#,
        )
        .unwrap();
        let relic_ids = HashSet::from(["ash_splinter".to_string()]);
        let mut report = ContentValidationReport::default();

        validate_level_item_references(&level, "test_level", &relic_ids, &mut report);

        assert!(!report.is_ok());
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("no matching relic definition")));
    }

    #[test]
    fn reports_empty_model_geometry() {
        let issues = validate_model_geometry(&crate::systems::render::mesh::empty_model());
        assert!(issues
            .iter()
            .any(|issue| issue.contains("render mesh parts")));
        assert!(issues.iter().any(|issue| issue.contains("render indices")));
        assert!(issues.iter().any(|issue| issue.contains("physics points")));
        assert!(issues
            .iter()
            .any(|issue| issue.contains("physics triangles")));
    }

    #[test]
    fn reports_unsupported_model_asset_extension() {
        let dir =
            std::env::temp_dir().join(format!("cenotaph_asset_validation_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let asset_path = dir.join("bad.txt");
        std::fs::write(&asset_path, "not a model").unwrap();

        let mut report = ContentValidationReport::default();
        validate_model_asset_file(&asset_path, &mut report);
        assert!(!report.is_ok());
        assert_eq!(report.checked_assets, 1);
        assert!(report
            .issues
            .iter()
            .any(|issue| issue.message.contains("unsupported model asset extension")));

        let _ = std::fs::remove_file(asset_path);
        let _ = std::fs::remove_dir(dir);
    }
}

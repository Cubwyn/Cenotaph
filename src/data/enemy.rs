use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::data::world::level::ColliderType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnemyDefinition {
    pub id: String,
    pub display_name: String,
    pub role: String,
    pub behavior_tag: String,
    pub model_asset: String,
    pub collider_type: ColliderType,
    pub visual_tell: String,
    pub health: f32,
    pub damage: f32,
    pub move_speed: f32,
    pub activation_range: f32,
    pub attack_range: f32,
    pub attack_windup: f32,
    pub attack_cooldown: f32,
}

#[derive(Debug, Clone, Default)]
pub struct EnemyRegistry {
    definitions: HashMap<String, EnemyDefinition>,
}

impl EnemyRegistry {
    pub fn try_load_dir(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("failed to read enemy definitions directory: {}", e))?;
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
            return Err(format!(
                "no enemy definition TOML files found in '{}'",
                path.display()
            ));
        }

        let mut definitions = Vec::with_capacity(enemy_paths.len());
        for enemy_path in enemy_paths {
            let definition = EnemyDefinition::try_load(&enemy_path)
                .map_err(|error| format!("{}: {}", enemy_path.to_string_lossy(), error))?;
            let errors = definition.validation_errors();
            if !errors.is_empty() {
                return Err(format!(
                    "{} failed validation: {}",
                    enemy_path.to_string_lossy(),
                    errors.join("; ")
                ));
            }
            let model_path = definition.model_path();
            if !model_path.is_file() {
                return Err(format!(
                    "{} references missing model asset '{}'",
                    enemy_path.to_string_lossy(),
                    model_path.to_string_lossy()
                ));
            }
            definitions.push(definition);
        }

        Self::from_definitions(definitions)
    }

    pub fn from_definitions(definitions: Vec<EnemyDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        for definition in definitions {
            registry.insert(definition)?;
        }
        Ok(registry)
    }

    pub fn insert(&mut self, definition: EnemyDefinition) -> Result<(), String> {
        let normalized_id = normalize_enemy_id(&definition.id);
        if normalized_id.is_empty() {
            return Err("enemy definition id must not be empty".to_string());
        }
        if self.definitions.contains_key(&normalized_id) {
            return Err(format!("duplicate enemy definition id '{}'", normalized_id));
        }
        self.definitions.insert(normalized_id, definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&EnemyDefinition> {
        self.definitions.get(&normalize_enemy_id(id))
    }

    pub fn len(&self) -> usize {
        self.definitions.len()
    }
}

impl EnemyDefinition {
    pub fn try_load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read enemy definition: {}", e))?;
        toml::from_str(&data).map_err(|e| format!("failed to parse enemy definition TOML: {}", e))
    }

    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        require_non_empty("id", &self.id, &mut errors);
        require_non_empty("display_name", &self.display_name, &mut errors);
        require_non_empty("role", &self.role, &mut errors);
        require_non_empty("behavior_tag", &self.behavior_tag, &mut errors);
        require_non_empty("model_asset", &self.model_asset, &mut errors);
        require_non_empty("visual_tell", &self.visual_tell, &mut errors);
        if !self.behavior_tag.trim().is_empty() && !is_known_behavior_tag(&self.behavior_tag) {
            errors.push(format!("unknown behavior_tag '{}'", self.behavior_tag));
        }

        require_positive("health", self.health, &mut errors);
        require_non_negative("damage", self.damage, &mut errors);
        require_non_negative("move_speed", self.move_speed, &mut errors);
        require_positive("activation_range", self.activation_range, &mut errors);
        require_positive("attack_range", self.attack_range, &mut errors);
        require_non_negative("attack_windup", self.attack_windup, &mut errors);
        require_non_negative("attack_cooldown", self.attack_cooldown, &mut errors);
        if self.activation_range.is_finite()
            && self.attack_range.is_finite()
            && self.activation_range < self.attack_range
        {
            errors.push("activation_range must be >= attack_range".to_string());
        }
        if self.damage > 0.0 && self.attack_windup <= 0.0 && self.attack_cooldown <= 0.0 {
            errors
                .push("damaging enemies need attack_windup or attack_cooldown above 0".to_string());
        }

        errors
    }

    pub fn model_path(&self) -> std::path::PathBuf {
        Path::new("assets").join(&self.model_asset)
    }
}

pub fn normalize_enemy_id(value: &str) -> String {
    let mut normalized = String::new();
    let mut previous_was_separator = false;

    for ch in value.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            normalized.push('_');
            previous_was_separator = true;
        }
    }

    normalized.trim_matches('_').to_string()
}

fn is_known_behavior_tag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "chase_melee" | "slow_chase_melee" | "ranged_windup" | "flanker_lunge" | "aerial_dive"
    )
}

fn require_non_empty(name: &str, value: &str, errors: &mut Vec<String>) {
    if value.trim().is_empty() {
        errors.push(format!("{} must not be empty", name));
    }
}

fn require_positive(name: &str, value: f32, errors: &mut Vec<String>) {
    if !value.is_finite() {
        errors.push(format!("{} must be finite", name));
    }
    if value <= 0.0 {
        errors.push(format!("{} must be > 0", name));
    }
}

fn require_non_negative(name: &str, value: f32, errors: &mut Vec<String>) {
    if !value.is_finite() {
        errors.push(format!("{} must be finite", name));
    }
    if value < 0.0 {
        errors.push(format!("{} must be >= 0", name));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_enemy_stats() {
        let enemy = EnemyDefinition {
            id: String::new(),
            display_name: "Test".to_string(),
            role: "grunt".to_string(),
            behavior_tag: "chase_melee".to_string(),
            model_asset: "Cube.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: String::new(),
            health: 0.0,
            damage: -1.0,
            move_speed: f32::NAN,
            activation_range: 0.0,
            attack_range: 0.0,
            attack_windup: -0.25,
            attack_cooldown: -0.5,
        };

        let errors = enemy.validation_errors();
        assert!(errors.iter().any(|error| error.contains("id")));
        assert!(errors.iter().any(|error| error.contains("visual_tell")));
        assert!(errors.iter().any(|error| error.contains("health")));
        assert!(errors.iter().any(|error| error.contains("damage")));
        assert!(errors.iter().any(|error| error.contains("move_speed")));
        assert!(errors
            .iter()
            .any(|error| error.contains("activation_range")));
        assert!(errors.iter().any(|error| error.contains("attack_range")));
        assert!(errors.iter().any(|error| error.contains("attack_windup")));
        assert!(errors.iter().any(|error| error.contains("attack_cooldown")));
    }

    #[test]
    fn rejects_enemy_activation_range_below_attack_range() {
        let enemy = EnemyDefinition {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            role: "grunt".to_string(),
            behavior_tag: "chase_melee".to_string(),
            model_asset: "Cube.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "test silhouette".to_string(),
            health: 20.0,
            damage: 4.0,
            move_speed: 2.0,
            activation_range: 1.0,
            attack_range: 2.0,
            attack_windup: 0.25,
            attack_cooldown: 1.0,
        };

        let errors = enemy.validation_errors();
        assert!(errors
            .iter()
            .any(|error| error.contains("activation_range must be >= attack_range")));
    }

    #[test]
    fn rejects_unknown_behavior_tag() {
        let enemy = EnemyDefinition {
            id: "test".to_string(),
            display_name: "Test".to_string(),
            role: "grunt".to_string(),
            behavior_tag: "made_up_behavior".to_string(),
            model_asset: "Cube.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "test silhouette".to_string(),
            health: 20.0,
            damage: 4.0,
            move_speed: 2.0,
            activation_range: 8.0,
            attack_range: 2.0,
            attack_windup: 0.25,
            attack_cooldown: 1.0,
        };

        assert!(enemy
            .validation_errors()
            .iter()
            .any(|error| error.contains("unknown behavior_tag")));
    }

    #[test]
    fn normalizes_enemy_ids_for_level_references() {
        assert_eq!(normalize_enemy_id("Burdened"), "burdened");
        assert_eq!(
            normalize_enemy_id("Mirror of the Player"),
            "mirror_of_the_player"
        );
        assert_eq!(normalize_enemy_id("  Chain-Runner  "), "chain_runner");
    }

    #[test]
    fn registry_resolves_display_names_to_ids() {
        let registry = EnemyRegistry::from_definitions(vec![EnemyDefinition {
            id: "burdened".to_string(),
            display_name: "Burdened".to_string(),
            role: "tank".to_string(),
            behavior_tag: "slow_chase_melee".to_string(),
            model_asset: "Cube.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "test silhouette".to_string(),
            health: 120.0,
            damage: 18.0,
            move_speed: 1.4,
            activation_range: 14.0,
            attack_range: 1.8,
            attack_windup: 0.75,
            attack_cooldown: 1.8,
        }])
        .unwrap();

        assert!(registry.get("Burdened").is_some());
        assert!(registry.get("burdened").is_some());
        assert!(registry.get("Burdened!!!").is_some());
    }
}

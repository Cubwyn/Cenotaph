use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelicDefinition {
    pub id: String,
    pub display_name: String,
    pub family: String,
    pub rarity: String,
    pub damage_multiplier: f32,
    pub cooldown_multiplier: f32,
    pub range_multiplier: f32,
    pub hit_stun_bonus: f32,
    pub flavor_text: String,
}

#[derive(Debug, Clone, Default)]
pub struct RelicRegistry {
    definitions: HashMap<String, RelicDefinition>,
}

impl RelicRegistry {
    pub fn load_dir(path: impl AsRef<Path>) -> Self {
        match Self::try_load_dir(path.as_ref()) {
            Ok(registry) => registry,
            Err(e) => {
                eprintln!("[RELIC DATA] {}", e);
                Self::default()
            }
        }
    }

    pub fn try_load_dir(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let entries = std::fs::read_dir(path)
            .map_err(|e| format!("failed to read relic definitions directory: {}", e))?;
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

        let definitions = relic_paths
            .into_iter()
            .map(RelicDefinition::try_load)
            .collect::<Result<Vec<_>, _>>()?;

        Self::from_definitions(definitions)
    }

    pub fn from_definitions(definitions: Vec<RelicDefinition>) -> Result<Self, String> {
        let mut registry = Self::default();
        for definition in definitions {
            registry.insert(definition)?;
        }
        Ok(registry)
    }

    pub fn insert(&mut self, definition: RelicDefinition) -> Result<(), String> {
        let normalized_id = normalize_relic_id(&definition.id);
        if normalized_id.is_empty() {
            return Err("relic definition id must not be empty".to_string());
        }
        if self.definitions.contains_key(&normalized_id) {
            return Err(format!("duplicate relic definition id '{}'", normalized_id));
        }
        self.definitions.insert(normalized_id, definition);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&RelicDefinition> {
        self.definitions.get(&normalize_relic_id(id))
    }
}

impl RelicDefinition {
    pub fn try_load(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let data = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read relic definition: {}", e))?;
        toml::from_str(&data).map_err(|e| format!("failed to parse relic definition TOML: {}", e))
    }

    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        require_non_empty("id", &self.id, &mut errors);
        require_non_empty("display_name", &self.display_name, &mut errors);
        require_non_empty("family", &self.family, &mut errors);
        require_non_empty("rarity", &self.rarity, &mut errors);
        require_non_empty("flavor_text", &self.flavor_text, &mut errors);

        require_positive("damage_multiplier", self.damage_multiplier, &mut errors);
        require_positive("cooldown_multiplier", self.cooldown_multiplier, &mut errors);
        require_positive("range_multiplier", self.range_multiplier, &mut errors);
        require_non_negative("hit_stun_bonus", self.hit_stun_bonus, &mut errors);

        errors
    }
}

pub fn normalize_relic_id(value: &str) -> String {
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

    fn relic_fixture() -> RelicDefinition {
        RelicDefinition {
            id: "ash_splinter".to_string(),
            display_name: "Ash Splinter".to_string(),
            family: "Sovereign".to_string(),
            rarity: "Common".to_string(),
            damage_multiplier: 1.2,
            cooldown_multiplier: 1.1,
            range_multiplier: 1.0,
            hit_stun_bonus: 0.05,
            flavor_text: "It remembers the shape of a first wound.".to_string(),
        }
    }

    #[test]
    fn normalizes_relic_ids() {
        assert_eq!(normalize_relic_id("Ash Splinter"), "ash_splinter");
        assert_eq!(normalize_relic_id("  Moon-Thread  "), "moon_thread");
    }

    #[test]
    fn rejects_invalid_relic_values() {
        let mut relic = relic_fixture();
        relic.id.clear();
        relic.damage_multiplier = 0.0;
        relic.cooldown_multiplier = -1.0;
        relic.hit_stun_bonus = f32::NAN;

        let errors = relic.validation_errors();
        assert!(errors.iter().any(|error| error.contains("id")));
        assert!(errors
            .iter()
            .any(|error| error.contains("damage_multiplier")));
        assert!(errors
            .iter()
            .any(|error| error.contains("cooldown_multiplier")));
        assert!(errors.iter().any(|error| error.contains("hit_stun_bonus")));
    }

    #[test]
    fn registry_resolves_display_style_ids() {
        let registry = RelicRegistry::from_definitions(vec![relic_fixture()]).unwrap();

        assert!(registry.get("ash_splinter").is_some());
        assert!(registry.get("Ash Splinter").is_some());
    }
}

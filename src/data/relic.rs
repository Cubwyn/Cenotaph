use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelicDefinition {
    pub id: String,
    pub display_name: String,
    /// Model path relative to `assets/` used for the in-world pickup.
    pub pickup_asset: String,
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
        if relic_paths.is_empty() {
            return Err(format!(
                "no relic definition TOML files found in '{}'",
                path.display()
            ));
        }

        let mut definitions = Vec::with_capacity(relic_paths.len());
        for relic_path in relic_paths {
            let definition = RelicDefinition::try_load(&relic_path)
                .map_err(|error| format!("{}: {}", relic_path.to_string_lossy(), error))?;
            let errors = definition.validation_errors();
            if !errors.is_empty() {
                return Err(format!(
                    "{} failed validation: {}",
                    relic_path.to_string_lossy(),
                    errors.join("; ")
                ));
            }
            definitions.push(definition);
        }

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

    pub fn len(&self) -> usize {
        self.definitions.len()
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
        require_non_empty("pickup_asset", &self.pickup_asset, &mut errors);
        require_non_empty("family", &self.family, &mut errors);
        require_non_empty("rarity", &self.rarity, &mut errors);
        require_non_empty("flavor_text", &self.flavor_text, &mut errors);

        require_positive("damage_multiplier", self.damage_multiplier, &mut errors);
        require_positive("cooldown_multiplier", self.cooldown_multiplier, &mut errors);
        require_positive("range_multiplier", self.range_multiplier, &mut errors);
        require_non_negative("hit_stun_bonus", self.hit_stun_bonus, &mut errors);

        let pickup_asset = Path::new(&self.pickup_asset);
        let safe_pickup_asset = !self.pickup_asset.trim().is_empty()
            && self.pickup_asset.trim() == self.pickup_asset
            && !pickup_asset.is_absolute()
            && pickup_asset
                .components()
                .all(|component| matches!(component, Component::Normal(_)));
        if !safe_pickup_asset {
            errors.push("pickup_asset must be a safe path relative to assets/".to_string());
        } else {
            let supported_model = pickup_asset
                .extension()
                .and_then(|extension| extension.to_str())
                .is_some_and(|extension| {
                    matches!(
                        extension.to_ascii_lowercase().as_str(),
                        "glb" | "gltf" | "obj"
                    )
                });
            if !supported_model {
                errors.push("pickup_asset must use a supported model format".to_string());
            } else if !Path::new("assets").join(pickup_asset).is_file() {
                errors.push(format!(
                    "pickup_asset references missing asset 'assets/{}'",
                    self.pickup_asset
                ));
            }
        }

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
            pickup_asset: "pickups/relic_ash_splinter.obj".to_string(),
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
        relic.pickup_asset = "../outside.obj".to_string();

        let errors = relic.validation_errors();
        assert!(errors.iter().any(|error| error.contains("id")));
        assert!(errors
            .iter()
            .any(|error| error.contains("damage_multiplier")));
        assert!(errors
            .iter()
            .any(|error| error.contains("cooldown_multiplier")));
        assert!(errors.iter().any(|error| error.contains("hit_stun_bonus")));
        assert!(errors.iter().any(|error| error.contains("pickup_asset")));
    }

    #[test]
    fn rejects_non_model_pickup_assets_before_runtime() {
        let mut relic = relic_fixture();
        relic.pickup_asset = "pickups/relic_ash_splinter.txt".to_string();

        assert!(relic
            .validation_errors()
            .iter()
            .any(|error| error.contains("supported model format")));
    }

    #[test]
    fn registry_resolves_display_style_ids() {
        let registry = RelicRegistry::from_definitions(vec![relic_fixture()]).unwrap();

        assert!(registry.get("ash_splinter").is_some());
        assert!(registry.get("Ash Splinter").is_some());
    }
}

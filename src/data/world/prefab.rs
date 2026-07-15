use std::collections::HashSet;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::persistence::recover_interrupted_write;

use super::level::PropData;

pub const CURRENT_PREFAB_VERSION: u32 = 1;
const MAX_PREFAB_PROPS: usize = 256;

fn current_prefab_version() -> u32 {
    CURRENT_PREFAB_VERSION
}

/// A reusable group of props stored with positions relative to its placement origin.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LevelPrefabData {
    #[serde(default = "current_prefab_version")]
    pub version: u32,
    pub name: String,
    #[serde(default)]
    pub props: Vec<PropData>,
}

impl LevelPrefabData {
    pub fn try_load(file_path: impl AsRef<Path>) -> Result<Self, String> {
        let file_path = file_path.as_ref();
        recover_interrupted_write(file_path)?;
        let data = fs::read_to_string(file_path).map_err(|error| {
            format!(
                "failed to read prefab file at {}: {}",
                file_path.to_string_lossy(),
                error
            )
        })?;
        serde_json::from_str(&data).map_err(|error| {
            format!(
                "failed to parse prefab JSON at {}: {}",
                file_path.to_string_lossy(),
                error
            )
        })
    }

    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.version != CURRENT_PREFAB_VERSION {
            errors.push(format!(
                "prefab version must be {}, found {}",
                CURRENT_PREFAB_VERSION, self.version
            ));
        }
        if self.name.trim().is_empty() {
            errors.push("prefab name must not be empty".to_string());
        } else if self.name.chars().count() > 80 {
            errors.push("prefab name must not exceed 80 characters".to_string());
        }
        if self.props.is_empty() {
            errors.push("prefab must contain at least one prop".to_string());
        }
        if self.props.len() > MAX_PREFAB_PROPS {
            errors.push(format!(
                "prefab must not contain more than {} props",
                MAX_PREFAB_PROPS
            ));
        }

        let mut prop_ids = HashSet::new();
        let mut anchor_ids = HashSet::new();
        for (index, prop) in self.props.iter().enumerate() {
            errors.extend(prop.validation_errors(index));
            match prop.id.as_deref() {
                Some(id) if !id.trim().is_empty() => {
                    if !prop_ids.insert(id.to_string()) {
                        errors.push(format!("prefab prop {} duplicates id '{}'", index, id));
                    }
                }
                _ => errors.push(format!("prefab prop {} requires a stable id", index)),
            }
            if let Some(anchor_id) = prop.anchor_id.as_deref() {
                if !anchor_ids.insert(anchor_id.to_string()) {
                    errors.push(format!(
                        "prefab prop {} duplicates anchor_id '{}'",
                        index, anchor_id
                    ));
                }
            }
            for (field, value) in [
                ("loot_table_id", prop.loot_table_id.as_deref()),
                ("path_id", prop.path_id.as_deref()),
                ("dialogue_id", prop.dialogue_id.as_deref()),
                ("event_id", prop.event_id.as_deref()),
            ] {
                if value.is_some() {
                    errors.push(format!(
                        "prefab prop {} cannot carry level-local {}",
                        index, field
                    ));
                }
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prefab_json(extra_prop_fields: &str) -> String {
        format!(
            r#"{{
                "version": 1,
                "name": "Test Room",
                "props": [{{
                    "id": "wall",
                    "asset_id": "Cube.obj",
                    "position": [0.0, 0.0, 0.0],
                    "collider_type": "Box"{}
                }}]
            }}"#,
            extra_prop_fields
        )
    }

    #[test]
    fn prefab_contract_round_trips_and_validates() {
        let prefab: LevelPrefabData = serde_json::from_str(&prefab_json("")).unwrap();
        assert!(prefab.validation_errors().is_empty());

        let serialized = serde_json::to_string(&prefab).unwrap();
        let loaded: LevelPrefabData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(loaded.version, CURRENT_PREFAB_VERSION);
        assert_eq!(loaded.props.len(), 1);
    }

    #[test]
    fn prefab_rejects_level_local_graph_references() {
        let prefab: LevelPrefabData =
            serde_json::from_str(&prefab_json(r#", "path_id": "patrol""#)).unwrap();
        let errors = prefab.validation_errors();
        assert!(errors
            .iter()
            .any(|error| error.contains("level-local path_id")));
    }

    #[test]
    fn prefab_requires_unique_stable_prop_ids() {
        let mut prefab: LevelPrefabData = serde_json::from_str(&prefab_json("")).unwrap();
        prefab.props.push(prefab.props[0].clone());
        let errors = prefab.validation_errors();
        assert!(errors.iter().any(|error| error.contains("duplicates id")));
    }
}

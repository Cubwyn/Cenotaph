use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::persistence::{recover_interrupted_write, write_file_staged};
use crate::data::world::level::{
    validate_level_id, ColliderType, PropData, RUNTIME_LOOT_ID_PREFIX,
};
use crate::game::cycle::CycleState;
use crate::game::progression::RunProgress;
use crate::game::relic::EquippedRelic;

pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_SAVE_PATH: &str = "save/cenotaph_save.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SavedRuntimeLoot {
    pub id: String,
    pub asset_id: String,
    pub position: [f32; 3],
    pub scale: [f32; 3],
    pub item_id: Option<String>,
    pub resource_value: u32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct LevelSaveSnapshot {
    pub fired_level_events: Vec<String>,
    pub level_flags: Vec<String>,
    pub removed_prop_ids: Vec<String>,
    pub runtime_loot: Vec<SavedRuntimeLoot>,
    pub pending_mountain_reactions: Vec<String>,
}

impl SavedRuntimeLoot {
    pub fn from_prop(prop: &PropData) -> Option<Self> {
        let id = prop.id.as_deref()?;
        if !id.starts_with(RUNTIME_LOOT_ID_PREFIX)
            || (prop.item_id.is_none() && prop.resource_value == 0)
        {
            return None;
        }

        Some(Self {
            id: id.to_string(),
            asset_id: prop.asset_id.clone(),
            position: prop.position,
            scale: prop.scale,
            item_id: prop.item_id.clone(),
            resource_value: prop.resource_value,
        })
    }

    pub fn to_prop(&self) -> PropData {
        PropData {
            id: Some(self.id.clone()),
            display_name: None,
            asset_id: self.asset_id.clone(),
            position: self.position,
            rotation: [0.0, 0.0, 0.0],
            scale: self.scale,
            collider_type: ColliderType::None,
            surface_material: None,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: false,
            item_id: self.item_id.clone(),
            resource_value: self.resource_value,
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

    fn validation_errors(&self, index: usize) -> Vec<String> {
        let mut errors = Vec::new();
        let label = format!("runtime_loot {} ('{}')", index, self.id);
        if !self.id.starts_with(RUNTIME_LOOT_ID_PREFIX)
            || self.id.len() == RUNTIME_LOOT_ID_PREFIX.len()
        {
            errors.push(format!(
                "{} id must start with '{}' and include a suffix",
                label, RUNTIME_LOOT_ID_PREFIX
            ));
        }
        let asset_path = Path::new(&self.asset_id);
        if self.asset_id.trim().is_empty()
            || asset_path.is_absolute()
            || !asset_path
                .components()
                .all(|component| matches!(component, Component::Normal(_)))
        {
            errors.push(format!("{} asset_id must be a relative asset path", label));
        }
        if !self.position.iter().all(|value| value.is_finite()) {
            errors.push(format!("{} position must contain finite numbers", label));
        }
        if !self
            .scale
            .iter()
            .all(|value| value.is_finite() && value.abs() > f32::EPSILON)
        {
            errors.push(format!(
                "{} scale must contain finite non-zero numbers",
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
        if self.item_id.is_some() == (self.resource_value > 0) {
            errors.push(format!(
                "{} must contain exactly one of item_id or resource_value",
                label
            ));
        }
        errors
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SaveFileHealth {
    Missing,
    Healthy(SaveData),
    Recoverable {
        backup: SaveData,
        primary_error: String,
    },
    Invalid {
        primary_error: String,
        backup_error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SaveData {
    pub version: u32,
    pub level_name: String,
    pub cycle_number: u32,
    pub unsecured_resource: u32,
    pub banked_resource: u32,
    pub active_anchor_id: Option<String>,
    pub respawn_position: Option<[f32; 3]>,
    pub relic_inventory: Vec<String>,
    pub equipped_relic_id: Option<String>,
    #[serde(default)]
    pub fired_level_events: Vec<String>,
    #[serde(default)]
    pub level_flags: Vec<String>,
    #[serde(default)]
    pub removed_prop_ids: Vec<String>,
    #[serde(default)]
    pub runtime_loot: Vec<SavedRuntimeLoot>,
    #[serde(default)]
    pub pending_mountain_reactions: Vec<String>,
}

impl SaveData {
    pub fn from_runtime(
        level_name: &str,
        progress: &RunProgress,
        equipped_relic: &EquippedRelic,
        cycle: &CycleState,
    ) -> Self {
        Self {
            version: SAVE_VERSION,
            level_name: level_name.to_string(),
            cycle_number: cycle.number,
            unsecured_resource: progress.unsecured_resource,
            banked_resource: progress.banked_resource,
            active_anchor_id: progress.active_anchor_id.clone(),
            respawn_position: progress.respawn_position,
            relic_inventory: equipped_relic.owned_ids(),
            equipped_relic_id: equipped_relic.equipped_id().map(ToOwned::to_owned),
            fired_level_events: Vec::new(),
            level_flags: Vec::new(),
            removed_prop_ids: Vec::new(),
            runtime_loot: Vec::new(),
            pending_mountain_reactions: Vec::new(),
        }
    }

    pub fn from_runtime_with_level_state(
        level_name: &str,
        progress: &RunProgress,
        equipped_relic: &EquippedRelic,
        cycle: &CycleState,
        mut level_state: LevelSaveSnapshot,
    ) -> Self {
        level_state.fired_level_events.sort();
        level_state.fired_level_events.dedup();
        level_state.level_flags.sort();
        level_state.level_flags.dedup();
        level_state.removed_prop_ids.sort();
        level_state.removed_prop_ids.dedup();
        level_state
            .runtime_loot
            .sort_by(|left, right| left.id.cmp(&right.id));

        let mut save = Self::from_runtime(level_name, progress, equipped_relic, cycle);
        save.fired_level_events = level_state.fired_level_events;
        save.level_flags = level_state.level_flags;
        save.removed_prop_ids = level_state.removed_prop_ids;
        save.runtime_loot = level_state.runtime_loot;
        save.pending_mountain_reactions = level_state.pending_mountain_reactions;
        save
    }

    pub fn to_progress(&self) -> RunProgress {
        RunProgress {
            unsecured_resource: self.unsecured_resource,
            banked_resource: self.banked_resource,
            active_anchor_id: self.active_anchor_id.clone(),
            respawn_position: self.respawn_position,
        }
    }

    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.version != SAVE_VERSION {
            errors.push(format!(
                "unsupported save version {}; expected {}",
                self.version, SAVE_VERSION
            ));
        }
        if validate_level_id(&self.level_name).is_err() {
            errors.push(
                "level_name must use only letters, numbers, '-' and '_' (maximum 64 characters)"
                    .to_string(),
            );
        }
        if self.cycle_number == 0 {
            errors.push("cycle_number must be at least 1".to_string());
        }
        if self
            .respawn_position
            .is_some_and(|position| !position.iter().all(|value| value.is_finite()))
        {
            errors.push("respawn_position must contain finite numbers".to_string());
        }
        if self
            .active_anchor_id
            .as_deref()
            .is_some_and(|id| id.trim().is_empty())
        {
            errors.push("active_anchor_id must not be empty when present".to_string());
        }

        validate_unique_ids("relic_inventory", &self.relic_inventory, &mut errors);
        validate_unique_ids("fired_level_events", &self.fired_level_events, &mut errors);
        validate_unique_ids("level_flags", &self.level_flags, &mut errors);
        validate_unique_ids("removed_prop_ids", &self.removed_prop_ids, &mut errors);
        let runtime_loot_ids = self
            .runtime_loot
            .iter()
            .map(|loot| loot.id.clone())
            .collect::<Vec<_>>();
        validate_unique_ids("runtime_loot ids", &runtime_loot_ids, &mut errors);
        for (index, loot) in self.runtime_loot.iter().enumerate() {
            errors.extend(loot.validation_errors(index));
        }
        validate_non_empty_ids(
            "pending_mountain_reactions",
            &self.pending_mountain_reactions,
            &mut errors,
        );
        if let Some(equipped_id) = self.equipped_relic_id.as_deref() {
            if equipped_id.trim().is_empty() {
                errors.push("equipped_relic_id must not be empty when present".to_string());
            } else if !self
                .relic_inventory
                .iter()
                .any(|owned_id| owned_id == equipped_id)
            {
                errors.push(format!(
                    "equipped_relic_id '{}' is not present in relic_inventory",
                    equipped_id
                ));
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

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        self.validate()
            .map_err(|errors| format!("save validation failed: {}", errors.join("; ")))?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create save directory: {}", e))?;
        }

        let data = format!(
            "{}\n",
            serde_json::to_string_pretty(self)
                .map_err(|e| format!("failed to serialize save data: {}", e))?
        );

        let backup_path = save_backup_path(path);
        if read_save_file(path).is_ok_and(|save| save.is_some()) {
            let previous = std::fs::read(path)
                .map_err(|error| format!("failed to read previous save for backup: {}", error))?;
            write_file_staged(&backup_path, &previous).map_err(|error| {
                format!(
                    "failed to preserve previous save at {}: {}",
                    backup_path.display(),
                    error
                )
            })?;
        }

        if let Err(error) = write_file_staged(path, data.as_bytes()) {
            if !path.exists() && backup_path.is_file() {
                let _ = std::fs::copy(&backup_path, path);
            }
            return Err(format!("failed to write save data: {}", error));
        }
        Ok(())
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Option<Self>, String> {
        let path = path.as_ref();
        match Self::inspect_path(path) {
            SaveFileHealth::Missing => Ok(None),
            SaveFileHealth::Healthy(save) => Ok(Some(save)),
            SaveFileHealth::Recoverable {
                backup,
                primary_error,
            } => {
                eprintln!(
                    "[SAVE] Primary save is unavailable ({}). Recovering the last known-good backup.",
                    primary_error
                );
                if let Err(error) = write_file_staged(
                    path,
                    format!(
                        "{}\n",
                        serde_json::to_string_pretty(&backup).map_err(|serialize_error| {
                            format!("failed to serialize recovered save: {}", serialize_error)
                        })?
                    )
                    .as_bytes(),
                ) {
                    eprintln!(
                        "[SAVE] Backup loaded, but the primary save could not be repaired: {}",
                        error
                    );
                }
                Ok(Some(backup))
            }
            SaveFileHealth::Invalid {
                primary_error,
                backup_error,
            } => Err(match backup_error {
                Some(backup_error) => format!(
                    "primary save is invalid ({}); backup is invalid ({})",
                    primary_error, backup_error
                ),
                None => format!(
                    "primary save is invalid ({}); no valid backup exists",
                    primary_error
                ),
            }),
        }
    }

    pub fn inspect_path(path: impl AsRef<Path>) -> SaveFileHealth {
        let path = path.as_ref();
        let backup_path = save_backup_path(path);
        match read_save_file(path) {
            Ok(Some(save)) => SaveFileHealth::Healthy(save),
            Ok(None) => match read_save_file(&backup_path) {
                Ok(Some(backup)) => SaveFileHealth::Recoverable {
                    backup,
                    primary_error: "primary save is missing".to_string(),
                },
                Ok(None) => SaveFileHealth::Missing,
                Err(backup_error) => SaveFileHealth::Invalid {
                    primary_error: "primary save is missing".to_string(),
                    backup_error: Some(backup_error),
                },
            },
            Err(primary_error) => match read_save_file(&backup_path) {
                Ok(Some(backup)) => SaveFileHealth::Recoverable {
                    backup,
                    primary_error,
                },
                Ok(None) => SaveFileHealth::Invalid {
                    primary_error,
                    backup_error: None,
                },
                Err(backup_error) => SaveFileHealth::Invalid {
                    primary_error,
                    backup_error: Some(backup_error),
                },
            },
        }
    }
}

fn read_save_file(path: &Path) -> Result<Option<SaveData>, String> {
    recover_interrupted_write(path)?;
    let data = match std::fs::read_to_string(path) {
        Ok(data) => data,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("failed to read '{}': {}", path.display(), error)),
    };
    let save: SaveData = serde_json::from_str(&data)
        .map_err(|error| format!("failed to parse '{}': {}", path.display(), error))?;
    save.validate().map_err(|errors| {
        format!(
            "'{}' failed validation: {}",
            path.display(),
            errors.join("; ")
        )
    })?;
    Ok(Some(save))
}

fn save_backup_path(path: &Path) -> PathBuf {
    let extension = path.extension().and_then(|value| value.to_str());
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("save");
    let backup_name = match extension {
        Some(extension) => format!("{}.backup.{}", stem, extension),
        None => format!("{}.backup", stem),
    };
    path.with_file_name(backup_name)
}

fn validate_unique_ids(label: &str, values: &[String], errors: &mut Vec<String>) {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() {
            errors.push(format!("{} must not contain empty ids", label));
        } else if !seen.insert(value) {
            errors.push(format!("{} contains duplicate id '{}'", label, value));
        }
    }
}

fn validate_non_empty_ids(label: &str, values: &[String], errors: &mut Vec<String>) {
    if values.iter().any(|value| value.trim().is_empty()) {
        errors.push(format!("{} must not contain empty ids", label));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::relic::RelicDefinition;

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

    fn save_fixture(level_name: &str, cycle_number: u32) -> SaveData {
        SaveData {
            version: SAVE_VERSION,
            level_name: level_name.to_string(),
            cycle_number,
            unsecured_resource: 4,
            banked_resource: 12,
            active_anchor_id: Some("anchor".to_string()),
            respawn_position: Some([1.0, 2.0, 3.0]),
            relic_inventory: vec!["ash_splinter".to_string()],
            equipped_relic_id: Some("ash_splinter".to_string()),
            fired_level_events: vec!["intro".to_string()],
            level_flags: vec!["door_open".to_string()],
            removed_prop_ids: vec!["claimed_pickup".to_string()],
            runtime_loot: Vec::new(),
            pending_mountain_reactions: Vec::new(),
        }
    }

    fn unique_save_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "cenotaph_{}_{}_{}.json",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    fn remove_save_files(path: &Path) {
        std::fs::remove_file(path).ok();
        std::fs::remove_file(save_backup_path(path)).ok();
    }

    #[test]
    fn save_data_snapshots_runtime_state() {
        let mut progress = RunProgress::new();
        progress.collect_resource(10);
        progress.activate_anchor("foundation_anchor", [1.0, 2.0, 3.0]);

        let mut equipped = EquippedRelic::new();
        equipped.acquire(relic_fixture());

        let save =
            SaveData::from_runtime("foundation_test", &progress, &equipped, &CycleState::new(3));

        assert_eq!(save.level_name, "foundation_test");
        assert_eq!(save.cycle_number, 3);
        assert_eq!(save.banked_resource, 10);
        assert_eq!(save.active_anchor_id.as_deref(), Some("foundation_anchor"));
        assert_eq!(save.relic_inventory, vec!["ash_splinter".to_string()]);
        assert_eq!(save.equipped_relic_id.as_deref(), Some("ash_splinter"));
        assert!(save.fired_level_events.is_empty());
        assert!(save.level_flags.is_empty());
        assert!(save.removed_prop_ids.is_empty());
        assert!(save.runtime_loot.is_empty());
        assert!(save.pending_mountain_reactions.is_empty());
    }

    #[test]
    fn save_data_snapshots_level_event_state() {
        let progress = RunProgress::new();
        let equipped = EquippedRelic::new();

        let save = SaveData::from_runtime_with_level_state(
            "foundation_test",
            &progress,
            &equipped,
            &CycleState::new(1),
            LevelSaveSnapshot {
                fired_level_events: vec!["arrival".to_string(), "arrival".to_string()],
                level_flags: vec!["opened_gate".to_string()],
                removed_prop_ids: vec!["spent_pickup".to_string()],
                runtime_loot: Vec::new(),
                pending_mountain_reactions: vec!["last_chain_severed".to_string()],
            },
        );

        assert_eq!(save.fired_level_events, vec!["arrival".to_string()]);
        assert_eq!(save.level_flags, vec!["opened_gate".to_string()]);
        assert_eq!(save.removed_prop_ids, vec!["spent_pickup".to_string()]);
        assert_eq!(
            save.pending_mountain_reactions,
            vec!["last_chain_severed".to_string()]
        );
    }

    #[test]
    fn mountain_reaction_queue_allows_reusing_a_profile_in_order() {
        let mut save = save_fixture("foundation_test", 1);
        save.pending_mountain_reactions =
            vec!["mountain_answer".to_string(), "mountain_answer".to_string()];

        assert_eq!(save.validate(), Ok(()));
    }

    #[test]
    fn runtime_loot_round_trips_without_serializing_engine_state() {
        let loot = SavedRuntimeLoot {
            id: "runtime_loot_0123_0".to_string(),
            asset_id: "pickups/relic_chain_sigil.obj".to_string(),
            position: [2.0, 3.0, 4.0],
            scale: [0.35, 0.35, 0.35],
            item_id: Some("debt_of_the_last_keeper".to_string()),
            resource_value: 0,
        };

        assert!(loot.validation_errors(0).is_empty());
        let restored = SavedRuntimeLoot::from_prop(&loot.to_prop()).unwrap();
        assert_eq!(restored, loot);
    }

    #[test]
    fn older_save_json_defaults_level_event_state() {
        let save: SaveData = serde_json::from_str(
            r#"
            {
                "version": 1,
                "level_name": "foundation_test",
                "cycle_number": 1,
                "unsecured_resource": 0,
                "banked_resource": 0,
                "active_anchor_id": null,
                "respawn_position": null,
                "relic_inventory": [],
                "equipped_relic_id": null
            }
            "#,
        )
        .unwrap();

        assert!(save.fired_level_events.is_empty());
        assert!(save.level_flags.is_empty());
        assert!(save.removed_prop_ids.is_empty());
        assert!(save.runtime_loot.is_empty());
        assert!(save.pending_mountain_reactions.is_empty());
    }

    #[test]
    fn save_file_round_trips() {
        let path = unique_save_path("save_round_trip");
        let save = save_fixture("foundation_test", 2);

        save.save_to_path(&path).unwrap();
        let loaded = SaveData::load_from_path(&path).unwrap();
        remove_save_files(&path);

        assert_eq!(loaded, Some(save));
    }

    #[test]
    fn invalid_save_is_rejected_before_disk_write() {
        let path = unique_save_path("invalid_save");
        let mut save = save_fixture("../outside", 0);
        save.respawn_position = Some([f32::NAN, 0.0, 0.0]);

        let error = save.save_to_path(&path).unwrap_err();

        assert!(error.contains("save validation failed"));
        assert!(!path.exists());
        remove_save_files(&path);
    }

    #[test]
    fn damaged_primary_recovers_last_known_good_backup() {
        let path = unique_save_path("save_recovery");
        let first = save_fixture("movement_test", 2);
        let second = save_fixture("foundation_test", 3);
        first.save_to_path(&path).unwrap();
        second.save_to_path(&path).unwrap();
        std::fs::write(&path, "{ damaged save").unwrap();

        match SaveData::inspect_path(&path) {
            SaveFileHealth::Recoverable { backup, .. } => assert_eq!(backup, first),
            health => panic!("expected recoverable save, got {health:?}"),
        }

        let recovered = SaveData::load_from_path(&path).unwrap();
        assert_eq!(recovered, Some(first.clone()));
        assert_eq!(
            SaveData::inspect_path(&path),
            SaveFileHealth::Healthy(first)
        );
        remove_save_files(&path);
    }

    #[test]
    fn invalid_primary_and_backup_fail_loudly() {
        let path = unique_save_path("invalid_save_pair");
        std::fs::write(&path, "invalid primary").unwrap();
        std::fs::write(save_backup_path(&path), "invalid backup").unwrap();

        let error = SaveData::load_from_path(&path).unwrap_err();

        assert!(error.contains("primary save is invalid"));
        assert!(error.contains("backup is invalid"));
        remove_save_files(&path);
    }
}

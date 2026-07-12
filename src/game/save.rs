use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::game::cycle::CycleState;
use crate::game::progression::RunProgress;
use crate::game::relic::EquippedRelic;

pub const SAVE_VERSION: u32 = 1;
pub const DEFAULT_SAVE_PATH: &str = "save/cenotaph_save.json";

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
        }
    }

    pub fn to_progress(&self) -> RunProgress {
        RunProgress {
            unsecured_resource: self.unsecured_resource,
            banked_resource: self.banked_resource,
            active_anchor_id: self.active_anchor_id.clone(),
            respawn_position: self.respawn_position,
        }
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create save directory: {}", e))?;
        }

        let data = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize save data: {}", e))?;
        std::fs::write(path, data).map_err(|e| format!("failed to write save data: {}", e))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Option<Self>, String> {
        let path = path.as_ref();
        let data = match std::fs::read_to_string(path) {
            Ok(data) => data,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(format!("failed to read save data: {}", error)),
        };

        let save: Self =
            serde_json::from_str(&data).map_err(|e| format!("failed to parse save data: {}", e))?;
        if save.version != SAVE_VERSION {
            return Err(format!(
                "unsupported save version {}; expected {}",
                save.version, SAVE_VERSION
            ));
        }

        Ok(Some(save))
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
    }

    #[test]
    fn save_file_round_trips() {
        let path = std::env::temp_dir().join(format!(
            "cenotaph_save_test_{}_{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let save = SaveData {
            version: SAVE_VERSION,
            level_name: "foundation_test".to_string(),
            cycle_number: 2,
            unsecured_resource: 4,
            banked_resource: 12,
            active_anchor_id: Some("anchor".to_string()),
            respawn_position: Some([1.0, 2.0, 3.0]),
            relic_inventory: vec!["ash_splinter".to_string()],
            equipped_relic_id: Some("ash_splinter".to_string()),
        };

        save.save_to_path(&path).unwrap();
        let loaded = SaveData::load_from_path(&path).unwrap();
        std::fs::remove_file(path).ok();

        assert_eq!(loaded, Some(save));
    }
}

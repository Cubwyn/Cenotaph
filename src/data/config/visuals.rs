//! Designer-owned visual profiles for prototype models and world props.
//!
//! The renderer consumes this small data contract; model paths, gameplay
//! behavior, and level geometry stay independent from presentation tuning.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_tint() -> [f32; 3] {
    [0.68, 0.66, 0.62]
}

fn default_uv_scale() -> f32 {
    2.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VisualProfile {
    pub tint: [f32; 3],
    pub texture: Option<String>,
    pub uv_scale: f32,
    pub emissive: f32,
    /// Shader role: 0 static, 1 pickup/relic, 2 enemy, 3 anchor/gate, 4 hazard.
    pub animation_role: f32,
}

impl Default for VisualProfile {
    fn default() -> Self {
        Self {
            tint: default_tint(),
            texture: None,
            uv_scale: default_uv_scale(),
            emissive: 0.0,
            animation_role: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct VisualConfig {
    pub default_profile: VisualProfile,
    pub profiles: HashMap<String, VisualProfile>,
}

impl Default for VisualConfig {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        let mut insert =
            |name: &str, tint: [f32; 3], texture: &str, emissive: f32, animation_role: f32| {
                profiles.insert(
                    name.to_string(),
                    VisualProfile {
                        tint,
                        texture: Some(texture.to_string()),
                        uv_scale: 2.0,
                        emissive,
                        animation_role,
                    },
                );
            };

        insert(
            "ashbound",
            [0.72, 0.60, 0.48],
            "cenotaph/ash_stone.png",
            0.012,
            2.0,
        );
        insert(
            "burdened",
            [0.58, 0.60, 0.64],
            "cenotaph/black_iron.png",
            0.012,
            2.0,
        );
        insert(
            "censer",
            [0.96, 0.32, 0.12],
            "cenotaph/ember_cracks.png",
            0.10,
            2.0,
        );
        insert(
            "chainrunner",
            [0.68, 0.20, 0.16],
            "cenotaph/black_iron.png",
            0.012,
            2.0,
        );
        insert(
            "harpy",
            [0.50, 0.58, 0.68],
            "cenotaph/ash_stone.png",
            0.012,
            2.0,
        );
        insert(
            "resource_shard",
            [0.32, 0.88, 1.0],
            "cenotaph/pale_waystone.png",
            0.07,
            1.0,
        );
        insert(
            "relic_ash",
            [1.0, 0.52, 0.18],
            "cenotaph/ember_cracks.png",
            0.07,
            1.0,
        );
        insert(
            "relic_veil",
            [0.36, 0.88, 0.72],
            "cenotaph/pale_waystone.png",
            0.06,
            1.0,
        );
        insert(
            "relic_chain",
            [0.92, 0.72, 0.24],
            "cenotaph/black_iron.png",
            0.06,
            1.0,
        );
        insert(
            "anchor",
            [0.28, 0.78, 1.0],
            "cenotaph/pale_waystone.png",
            0.04,
            3.0,
        );
        insert(
            "transition_gate",
            [0.58, 0.76, 0.88],
            "cenotaph/black_iron.png",
            0.02,
            3.0,
        );
        insert(
            "hurtbox",
            [1.0, 0.16, 0.06],
            "cenotaph/ember_cracks.png",
            0.16,
            4.0,
        );
        insert(
            "props",
            [0.68, 0.66, 0.62],
            "cenotaph/weathered_stone.png",
            0.0,
            0.0,
        );

        Self {
            default_profile: VisualProfile::default(),
            profiles,
        }
    }
}

impl VisualConfig {
    pub fn profile_for(
        &self,
        asset_id: &str,
        enemy_type: Option<&str>,
        is_hurtbox: bool,
    ) -> &VisualProfile {
        let asset = asset_id.to_ascii_lowercase();
        let enemy = enemy_type.unwrap_or_default().to_ascii_lowercase();

        if is_hurtbox {
            return self
                .profiles
                .get("hurtbox")
                .unwrap_or(&self.default_profile);
        }

        let keys = [
            "resource_shard",
            "relic_ash",
            "relic_veil",
            "relic_chain",
            "transition_gate",
            "anchor",
            "ashbound",
            "burdened",
            "censer",
            "chainrunner",
            "harpy",
        ];
        for key in keys {
            if asset.contains(key) {
                if let Some(profile) = self.profiles.get(key) {
                    return profile;
                }
            }
        }
        if !enemy.is_empty() {
            for (key, profile) in &self.profiles {
                if enemy.contains(key) {
                    return profile;
                }
            }
        }
        if asset.contains("props/") {
            return self.profiles.get("props").unwrap_or(&self.default_profile);
        }
        &self.default_profile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_cover_runtime_visual_roles() {
        let config = VisualConfig::default();
        assert_eq!(
            config
                .profile_for("enemies/censer.obj", Some("Censer"), false)
                .animation_role,
            2.0
        );
        assert_eq!(
            config
                .profile_for("world/hurtbox_warning.obj", None, true)
                .emissive,
            0.16
        );
    }

    #[test]
    fn visual_profiles_round_trip_through_toml() {
        let source = r#"
            [default_profile]
            tint = [0.2, 0.3, 0.4]
            texture = "cenotaph/black_iron.png"
            uv_scale = 3.0
            emissive = 0.1
            animation_role = 2.0

            [profiles.test]
            tint = [1.0, 0.5, 0.2]
            texture = "cenotaph/ember_cracks.png"
            uv_scale = 4.0
            emissive = 0.2
            animation_role = 1.0
        "#;
        let parsed: VisualConfig = toml::from_str(source).expect("visual config should parse");
        assert_eq!(parsed.profiles["test"].animation_role, 1.0);
        assert_eq!(parsed.default_profile.uv_scale, 3.0);
    }
}

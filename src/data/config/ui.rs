//! Designer-owned presentation tokens for the in-game interface.
//!
//! Widgets consume semantic colors, so changing the palette here rethemes the
//! HUD without changing layout or gameplay code.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct HudThemeConfig {
    pub void: [f32; 4],
    pub surface: [f32; 4],
    pub surface_raised: [f32; 4],
    pub line: [f32; 4],
    pub bone: [f32; 4],
    pub bone_dim: [f32; 4],
    pub ash: [f32; 4],
    pub gold: [f32; 4],
    pub gold_bright: [f32; 4],
    pub cold: [f32; 4],
    pub blood: [f32; 4],
    pub ember: [f32; 4],
    pub stamina: [f32; 4],
    pub success: [f32; 4],
}

impl Default for HudThemeConfig {
    fn default() -> Self {
        Self {
            void: [0.008, 0.009, 0.010, 0.78],
            surface: [0.026, 0.028, 0.030, 0.72],
            surface_raised: [0.065, 0.062, 0.055, 0.82],
            line: [0.42, 0.40, 0.36, 0.34],
            bone: [0.88, 0.85, 0.77, 0.94],
            bone_dim: [0.58, 0.57, 0.53, 0.68],
            ash: [0.43, 0.49, 0.52, 0.76],
            gold: [0.76, 0.57, 0.25, 0.88],
            gold_bright: [1.0, 0.78, 0.34, 0.96],
            cold: [0.30, 0.70, 0.78, 0.90],
            blood: [0.72, 0.13, 0.09, 0.92],
            ember: [1.0, 0.38, 0.08, 0.94],
            stamina: [0.78, 0.68, 0.30, 0.90],
            success: [0.32, 0.72, 0.49, 0.90],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(default)]
pub struct UiConfig {
    pub hud: HudThemeConfig,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_config_can_be_rethemed_from_toml() {
        let parsed: UiConfig = toml::from_str(
            r#"
            [hud]
            gold = [0.1, 0.2, 0.3, 1.0]
            "#,
        )
        .expect("ui config should parse");

        assert_eq!(parsed.hud.gold, [0.1, 0.2, 0.3, 1.0]);
        assert_eq!(parsed.hud.bone, HudThemeConfig::default().bone);
    }
}

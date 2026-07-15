use crate::data::config::ui::HudThemeConfig;

pub(super) type HudColor = [f32; 4];

#[derive(Debug, Clone, Copy)]
pub(super) struct HudTheme {
    pub void: HudColor,
    pub surface: HudColor,
    pub surface_raised: HudColor,
    pub line: HudColor,
    pub bone: HudColor,
    pub bone_dim: HudColor,
    pub ash: HudColor,
    pub gold: HudColor,
    pub gold_bright: HudColor,
    pub cold: HudColor,
    pub blood: HudColor,
    pub ember: HudColor,
    pub stamina: HudColor,
}

pub(super) fn with_alpha(mut color: HudColor, alpha: f32) -> HudColor {
    color[3] *= alpha.clamp(0.0, 1.0);
    color
}

impl HudTheme {
    pub(super) fn from_config(config: &HudThemeConfig) -> Self {
        Self {
            void: config.void,
            surface: config.surface,
            surface_raised: config.surface_raised,
            line: config.line,
            bone: config.bone,
            bone_dim: config.bone_dim,
            ash: config.ash,
            gold: config.gold,
            gold_bright: config.gold_bright,
            cold: config.cold,
            blood: config.blood,
            ember: config.ember,
            stamina: config.stamina,
        }
    }

    pub fn health(self, ratio: f32) -> HudColor {
        if ratio <= 0.25 {
            self.ember
        } else {
            self.blood
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_accents_remain_visually_distinct() {
        let theme = std::hint::black_box(HudTheme::from_config(&HudThemeConfig::default()));
        assert_ne!(theme.gold, theme.cold);
        assert_ne!(theme.blood, theme.stamina);
        assert!(theme.bone[0] > theme.surface[0]);
    }

    #[test]
    fn alpha_multiplier_is_clamped() {
        assert_eq!(with_alpha([1.0, 1.0, 1.0, 0.5], 2.0)[3], 0.5);
        assert_eq!(with_alpha([1.0, 1.0, 1.0, 0.5], -1.0)[3], 0.0);
    }
}

const REFERENCE_HEIGHT: f32 = 840.0;
const REFERENCE_ASPECT: f32 = 4.0 / 3.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct HudRect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl HudRect {
    pub fn right(self) -> f32 {
        self.x + self.w
    }

    pub fn top(self) -> f32 {
        self.y + self.h
    }

    pub fn center_x(self) -> f32 {
        self.x + self.w * 0.5
    }

    pub fn overlaps(self, other: Self) -> bool {
        self.x < other.right()
            && self.right() > other.x
            && self.y < other.top()
            && self.top() > other.y
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct HudLayout {
    pub scale: f32,
    pub horizontal_scale: f32,
    pub glyph_x_scale: f32,
    pub ascent: HudRect,
    pub player: HudRect,
    pub event_feed: HudRect,
    pub interaction: HudRect,
    pub dialogue: HudRect,
    pub anchor_rite: HudRect,
    pub objective: HudRect,
    pub boss: HudRect,
    pub status_effects: HudRect,
}

impl HudLayout {
    pub fn new(viewport: [u32; 2]) -> Self {
        let width = viewport[0].max(1) as f32;
        let height = viewport[1].max(1) as f32;
        let aspect = (width / height).clamp(0.75, 3.0);
        let scale = (REFERENCE_HEIGHT / height).clamp(0.60, 1.30);
        let horizontal_scale = (scale * REFERENCE_ASPECT / aspect).clamp(0.42, 1.38);
        let glyph_x_scale = (0.70 * REFERENCE_ASPECT / aspect).clamp(0.36, 0.82);
        let left = -0.95;
        let right = 0.95;

        let ascent_w = 0.72 * horizontal_scale;
        let ascent_h = 0.17 * scale;
        let player_w = 0.76 * horizontal_scale;
        let player_h = 0.19 * scale;
        let feed_w = 0.44 * horizontal_scale;
        let prompt_w = 0.60 * horizontal_scale;
        let prompt_h = 0.13 * scale;
        let dialogue_w = (1.34 * horizontal_scale).min(1.72);
        let dialogue_h = 0.22 * scale;
        let anchor_rite_w = (1.20 * horizontal_scale).min(1.72);
        let anchor_rite_h = (1.18 * scale).min(1.68);
        let objective_w = 0.58 * horizontal_scale;
        let boss_w = (0.88 * horizontal_scale).min(1.10);

        let player = HudRect {
            x: left,
            y: -0.95,
            w: player_w,
            h: player_h,
        };
        let dialogue = HudRect {
            x: -dialogue_w * 0.5,
            y: (-0.76_f32).max(player.top() + 0.02 * scale),
            w: dialogue_w,
            h: dialogue_h,
        };
        let mut boss = HudRect {
            x: -boss_w * 0.5,
            y: -0.90,
            w: boss_w,
            h: 0.08 * scale,
        };
        if boss.overlaps(player) {
            let available_left = player.right() + 0.025 * horizontal_scale;
            boss.x = available_left;
            boss.w = (right - available_left).max(0.01).min(boss_w);
        }

        let layout = Self {
            scale,
            horizontal_scale,
            glyph_x_scale,
            ascent: HudRect {
                x: left,
                y: 0.95 - ascent_h,
                w: ascent_w,
                h: ascent_h,
            },
            player,
            event_feed: HudRect {
                x: right - feed_w,
                y: 0.34,
                w: feed_w,
                h: 0.57,
            },
            interaction: HudRect {
                x: -prompt_w * 0.5,
                y: -0.64,
                w: prompt_w,
                h: prompt_h,
            },
            dialogue,
            anchor_rite: HudRect {
                x: -anchor_rite_w * 0.5,
                y: -anchor_rite_h * 0.5,
                w: anchor_rite_w,
                h: anchor_rite_h,
            },
            objective: HudRect {
                x: -objective_w * 0.5,
                y: 0.78,
                w: objective_w,
                h: 0.12 * scale,
            },
            boss,
            status_effects: HudRect {
                x: left,
                y: -0.72,
                w: 0.42 * horizontal_scale,
                h: 0.11 * scale,
            },
        };

        debug_assert!(layout.is_valid());
        layout
    }

    pub fn sx(self, value: f32) -> f32 {
        value * self.horizontal_scale
    }

    pub fn sy(self, value: f32) -> f32 {
        value * self.scale
    }

    pub fn text_scale(self, value: f32) -> f32 {
        value * self.scale
    }

    fn is_valid(self) -> bool {
        let regions = [
            self.ascent,
            self.player,
            self.event_feed,
            self.interaction,
            self.dialogue,
            self.anchor_rite,
            self.objective,
            self.boss,
            self.status_effects,
        ];
        regions.iter().all(|rect| {
            rect.w > 0.0
                && rect.h > 0.0
                && rect.x >= -1.0
                && rect.right() <= 1.0
                && rect.y >= -1.0
                && rect.top() <= 1.0
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regions_stay_on_screen_at_common_aspects() {
        for viewport in [[800, 600], [1280, 720], [1920, 1080], [2560, 1080]] {
            assert!(HudLayout::new(viewport).is_valid(), "{viewport:?}");
        }
    }

    #[test]
    fn persistent_corner_regions_do_not_overlap() {
        for viewport in [[800, 600], [1280, 720], [1920, 1080], [2560, 1080]] {
            let layout = HudLayout::new(viewport);
            assert!(!layout.ascent.overlaps(layout.player), "{viewport:?}");
            assert!(!layout.ascent.overlaps(layout.event_feed), "{viewport:?}");
            assert!(!layout.player.overlaps(layout.dialogue), "{viewport:?}");
            assert!(!layout.player.overlaps(layout.boss), "{viewport:?}");
            assert!(!layout.dialogue.overlaps(layout.boss), "{viewport:?}");
        }
    }

    #[test]
    fn widescreen_layout_keeps_physical_proportions() {
        let standard = HudLayout::new([960, 720]);
        let wide = HudLayout::new([1280, 720]);
        assert!(wide.ascent.w < standard.ascent.w);
        assert!(wide.glyph_x_scale < standard.glyph_x_scale);
    }

    #[test]
    fn anchor_rite_region_stays_centered_and_on_screen() {
        for viewport in [[800, 600], [1280, 720], [1920, 1080], [2560, 1080]] {
            let layout = HudLayout::new(viewport);
            let rite = layout.anchor_rite;
            assert!(rite.center_x().abs() <= f32::EPSILON, "{viewport:?}");
            assert!(
                (rite.y + rite.h * 0.5).abs() <= f32::EPSILON,
                "{viewport:?}"
            );
            assert!(rite.x >= -1.0 && rite.right() <= 1.0, "{viewport:?}");
            assert!(rite.y >= -1.0 && rite.top() <= 1.0, "{viewport:?}");
        }
    }
}

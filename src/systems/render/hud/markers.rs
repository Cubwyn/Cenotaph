//! Projected world markers. These are intentionally lighter than HUD panels:
//! they identify a world-space target without covering the scene.

use super::{
    with_alpha, HudLayout, HudMarkerKind, HudMarkerState, HudSystem, HudTheme, HudVertex,
    HudWorldMarker,
};

impl HudSystem {
    pub(super) fn push_world_marker(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        marker: HudWorldMarker,
        time: f32,
    ) {
        let [x, y] = marker.screen_pos;
        let pulse = 0.5 + 0.5 * (time * 4.2).sin();
        let ratio = marker.ratio.clamp(0.0, 1.0);
        let distance_fade = (1.0 - (marker.distance_m as f32 - 18.0) / 100.0).clamp(0.30, 1.0);

        match marker.kind {
            HudMarkerKind::Enemy => {
                let state_ratio = marker.state_ratio.clamp(0.0, 1.0);
                let (accent, state_label) = match marker.state {
                    HudMarkerState::Neutral => (with_alpha(theme.blood, 0.54), ""),
                    HudMarkerState::Aggro => (theme.blood, ""),
                    HudMarkerState::Windup => (theme.gold_bright, "STRIKE"),
                    HudMarkerState::Staggered => (theme.cold, "BROKEN"),
                };
                let size = 0.034 + (1.0 - ratio) * 0.006;
                Self::push_diamond_outline(
                    verts,
                    [x, y],
                    [layout.sx(size), layout.sy(size)],
                    layout.sy(0.0025),
                    with_alpha(accent, distance_fade),
                );
                Self::push_diamond(
                    verts,
                    [x, y],
                    [layout.sx(0.009), layout.sy(0.009)],
                    with_alpha(accent, 0.80 * distance_fade),
                );

                if marker.state != HudMarkerState::Neutral {
                    let outer_size = size + 0.018 + pulse * 0.005;
                    Self::push_diamond_outline(
                        verts,
                        [x, y],
                        [layout.sx(outer_size), layout.sy(outer_size)],
                        layout.sy(0.0018),
                        with_alpha(accent, 0.32 + pulse * 0.20),
                    );
                }

                match marker.state {
                    HudMarkerState::Windup => {
                        let bar_w = layout.sx(0.070);
                        let bar_h = layout.sy(0.006);
                        Self::push_rect(
                            verts,
                            x - bar_w * 0.5,
                            y + layout.sy(0.041),
                            bar_w,
                            bar_h,
                            theme.void,
                        );
                        Self::push_rect(
                            verts,
                            x - bar_w * 0.5,
                            y + layout.sy(0.041),
                            bar_w * state_ratio,
                            bar_h,
                            theme.gold_bright,
                        );
                    }
                    HudMarkerState::Staggered => {
                        let cross = [
                            layout.sx(0.014 + state_ratio * 0.004),
                            layout.sy(0.014 + state_ratio * 0.004),
                        ];
                        Self::push_line(
                            verts,
                            [x - cross[0], y - cross[1]],
                            [x + cross[0], y + cross[1]],
                            layout.sy(0.002),
                            theme.cold,
                        );
                        Self::push_line(
                            verts,
                            [x - cross[0], y + cross[1]],
                            [x + cross[0], y - cross[1]],
                            layout.sy(0.002),
                            theme.cold,
                        );
                    }
                    HudMarkerState::Neutral | HudMarkerState::Aggro => {}
                }

                if !state_label.is_empty() {
                    Self::push_ui_centered_text(
                        verts,
                        layout,
                        x,
                        y + layout.sy(0.056),
                        state_label,
                        0.27,
                        with_alpha(accent, distance_fade),
                    );
                }

                if ratio < 0.995 || marker.state != HudMarkerState::Neutral {
                    let health_w = layout.sx(0.064);
                    let health_y = y - layout.sy(0.040);
                    Self::push_rect(
                        verts,
                        x - health_w * 0.5,
                        health_y,
                        health_w,
                        layout.sy(0.006),
                        theme.void,
                    );
                    Self::push_rect(
                        verts,
                        x - health_w * 0.5,
                        health_y,
                        health_w * ratio,
                        layout.sy(0.006),
                        with_alpha(theme.blood, distance_fade),
                    );
                }
            }
            HudMarkerKind::Loot => {
                let size = 0.027 + pulse * 0.004;
                Self::push_diamond_outline(
                    verts,
                    [x, y],
                    [layout.sx(size), layout.sy(size)],
                    layout.sy(0.002),
                    with_alpha(theme.gold_bright, distance_fade),
                );
                Self::push_diamond(
                    verts,
                    [x, y],
                    [layout.sx(0.011), layout.sy(0.011)],
                    with_alpha(theme.gold_bright, 0.75 * distance_fade),
                );
                Self::push_line(
                    verts,
                    [x, y + layout.sy(0.020)],
                    [x, y + layout.sy(0.040 + pulse * 0.006)],
                    layout.sy(0.002),
                    with_alpha(theme.gold, 0.62 * distance_fade),
                );
                Self::push_ui_centered_text(
                    verts,
                    layout,
                    x,
                    y - layout.sy(0.050),
                    &marker.distance_m.to_string(),
                    0.29,
                    with_alpha(theme.gold, 0.74 * distance_fade),
                );
            }
            HudMarkerKind::Anchor => {
                Self::push_diamond_outline(
                    verts,
                    [x, y],
                    [layout.sx(0.058), layout.sy(0.058)],
                    layout.sy(0.003),
                    with_alpha(theme.cold, distance_fade),
                );
                Self::push_diamond_outline(
                    verts,
                    [x, y],
                    [layout.sx(0.032), layout.sy(0.032)],
                    layout.sy(0.002),
                    with_alpha(theme.bone, 0.70 * distance_fade),
                );
                Self::push_diamond(
                    verts,
                    [x, y],
                    [layout.sx(0.009), layout.sy(0.009)],
                    with_alpha(theme.cold, distance_fade),
                );
                Self::push_ui_centered_text(
                    verts,
                    layout,
                    x,
                    y - layout.sy(0.058),
                    &marker.distance_m.to_string(),
                    0.30,
                    with_alpha(theme.cold, 0.78 * distance_fade),
                );
            }
            HudMarkerKind::Hazard => {
                let left = [x - layout.sx(0.025), y - layout.sy(0.022)];
                let right = [x + layout.sx(0.025), y - layout.sy(0.022)];
                let top = [x, y + layout.sy(0.030)];
                let color = with_alpha(theme.ember, distance_fade);
                Self::push_line(verts, left, top, layout.sy(0.003), color);
                Self::push_line(verts, top, right, layout.sy(0.003), color);
                Self::push_line(verts, right, left, layout.sy(0.003), color);
                Self::push_line(
                    verts,
                    [x, y - layout.sy(0.010)],
                    [x, y + layout.sy(0.014)],
                    layout.sy(0.003),
                    color,
                );
                Self::push_diamond(
                    verts,
                    [x, y - layout.sy(0.016)],
                    [layout.sx(0.006), layout.sy(0.006)],
                    color,
                );
            }
        }
    }
}

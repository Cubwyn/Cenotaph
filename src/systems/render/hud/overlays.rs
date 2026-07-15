//! Full-screen states and death presentation.

use super::{HudIcon, HudLayout, HudSystem, HudTheme, HudVertex};

impl HudSystem {
    pub(super) fn push_pause_overlay(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
    ) {
        Self::push_rect(verts, -1.0, -1.0, 2.0, 2.0, [0.0, 0.0, 0.0, 0.58]);
        Self::push_icon(
            verts,
            HudIcon::Pause,
            [0.0, 0.18],
            [layout.sx(0.12), layout.sy(0.15)],
            theme.gold,
        );
        let rail_width = layout.sx(0.52);
        Self::push_line(
            verts,
            [-rail_width * 0.5, 0.04],
            [rail_width * 0.5, 0.04],
            layout.sy(0.003),
            theme.line,
        );
        Self::push_ui_centered_text(verts, layout, 0.0, -0.045, "PAUSED", 0.82, theme.bone);
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            -0.145,
            "THE RITE IS HELD",
            0.36,
            theme.bone_dim,
        );
    }

    pub(super) fn push_death_overlay(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        respawn_remaining: f32,
    ) {
        Self::push_rect(verts, -1.0, -1.0, 2.0, 2.0, [0.055, 0.004, 0.003, 0.66]);
        Self::push_icon(
            verts,
            HudIcon::Death,
            [0.0, 0.22],
            [layout.sx(0.14), layout.sy(0.14)],
            theme.ember,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            0.065,
            "THE VESSEL FALLS",
            0.86,
            theme.bone,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            -0.060,
            "RETURNING TO THE ANCHOR",
            0.40,
            theme.bone_dim,
        );
        let countdown = respawn_remaining.ceil().max(0.0) as u32;
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            -0.170,
            &countdown.to_string(),
            0.92,
            theme.gold_bright,
        );
    }
}

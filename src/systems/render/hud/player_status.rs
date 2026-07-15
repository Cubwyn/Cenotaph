//! Player health, stamina, and dash widget.

use super::{
    with_alpha, HudIcon, HudLayout, HudMeterStyle, HudRect, HudSystem, HudTheme, HudVertex,
    PlayerHudState,
};

impl HudSystem {
    pub(super) fn push_player_status_panel(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        state: PlayerHudState,
    ) {
        let rect = layout.player;
        let health_color = theme.health(state.health_ratio);
        Self::push_cut_panel(
            verts,
            rect,
            [layout.sx(0.014), layout.sy(0.014)],
            theme.surface,
            theme.line,
        );
        Self::push_rect(
            verts,
            rect.x + layout.sx(0.018),
            rect.y + layout.sy(0.018),
            layout.sx(0.004),
            rect.h - layout.sy(0.036),
            health_color,
        );

        let meter_x = rect.x + layout.sx(0.090);
        let meter_w = rect.w - layout.sx(0.275);
        Self::push_icon(
            verts,
            HudIcon::Vital,
            [rect.x + layout.sx(0.052), rect.y + layout.sy(0.120)],
            [layout.sx(0.046), layout.sy(0.055)],
            health_color,
        );
        Self::push_ui_text(
            verts,
            layout,
            meter_x,
            rect.y + layout.sy(0.139),
            "VITAL",
            0.34,
            theme.bone_dim,
        );
        let health_value = format!("{}/{}", state.health_current, state.health_max);
        let health_value_w = Self::ui_text_width(layout, &health_value, 0.46);
        Self::push_ui_text(
            verts,
            layout,
            meter_x + meter_w - health_value_w,
            rect.y + layout.sy(0.131),
            &health_value,
            0.46,
            theme.bone,
        );
        Self::push_themed_meter(
            verts,
            HudRect {
                x: meter_x,
                y: rect.y + layout.sy(0.092),
                w: meter_w,
                h: layout.sy(0.026),
            },
            [state.health_ratio, state.health_trail_ratio],
            HudMeterStyle {
                fill: health_color,
                trail: with_alpha(theme.ember, 0.52),
                segments: 10,
            },
            theme,
        );

        Self::push_icon(
            verts,
            HudIcon::Stamina,
            [rect.x + layout.sx(0.052), rect.y + layout.sy(0.046)],
            [layout.sx(0.048), layout.sy(0.040)],
            theme.stamina,
        );
        Self::push_ui_text(
            verts,
            layout,
            meter_x,
            rect.y + layout.sy(0.054),
            "BREATH",
            0.30,
            theme.bone_dim,
        );
        let stamina_value = format!("{}/{}", state.stamina_current, state.stamina_max);
        let stamina_value_w = Self::ui_text_width(layout, &stamina_value, 0.38);
        Self::push_ui_text(
            verts,
            layout,
            meter_x + meter_w - stamina_value_w,
            rect.y + layout.sy(0.049),
            &stamina_value,
            0.38,
            theme.bone_dim,
        );
        Self::push_themed_meter(
            verts,
            HudRect {
                x: meter_x,
                y: rect.y + layout.sy(0.026),
                w: meter_w,
                h: layout.sy(0.015),
            },
            [state.stamina_ratio, state.stamina_ratio],
            HudMeterStyle {
                fill: theme.stamina,
                trail: theme.stamina,
                segments: 6,
            },
            theme,
        );

        let dash_center = [rect.right() - layout.sx(0.080), rect.y + layout.sy(0.095)];
        let dash_ready = state.dash_cooldown_ratio <= 0.01;
        let dash_color = if dash_ready { theme.cold } else { theme.ash };
        Self::push_diamond_outline(
            verts,
            dash_center,
            [layout.sx(0.108), layout.sy(0.108)],
            layout.sy(0.003),
            with_alpha(dash_color, 0.58),
        );
        let readiness = if dash_ready {
            1.0
        } else {
            1.0 - state.dash_cooldown_ratio.clamp(0.0, 1.0)
        };
        Self::push_diamond(
            verts,
            dash_center,
            [layout.sx(0.078 * readiness), layout.sy(0.078 * readiness)],
            with_alpha(dash_color, 0.24 + readiness * 0.34),
        );
        Self::push_icon(
            verts,
            HudIcon::Dash,
            dash_center,
            [layout.sx(0.050), layout.sy(0.045)],
            dash_color,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            dash_center[0],
            rect.y + layout.sy(0.022),
            if dash_ready { "DASH" } else { "WAIT" },
            0.31,
            if dash_ready {
                theme.cold
            } else {
                theme.bone_dim
            },
        );
    }
}

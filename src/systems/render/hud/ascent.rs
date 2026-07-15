//! Run identity, relic, and resource widget.

use super::{
    with_alpha, AscentHudState, HudIcon, HudLayout, HudSystem, HudTextFit, HudTheme, HudVertex,
};

impl HudSystem {
    pub(super) fn push_ascent_panel(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        ascent: &AscentHudState,
    ) {
        let rect = layout.ascent;
        Self::push_cut_panel(
            verts,
            rect,
            [layout.sx(0.014), layout.sy(0.014)],
            theme.surface,
            theme.line,
        );
        Self::push_rect(
            verts,
            rect.x + layout.sx(0.022),
            rect.top() - layout.sy(0.006),
            rect.w - layout.sx(0.044),
            layout.sy(0.003),
            theme.gold,
        );

        let top_center_y = rect.y + layout.sy(0.121);
        Self::push_icon(
            verts,
            HudIcon::Relic,
            [rect.x + layout.sx(0.040), top_center_y],
            [layout.sx(0.040), layout.sy(0.048)],
            theme.gold,
        );
        Self::push_ui_text(
            verts,
            layout,
            rect.x + layout.sx(0.072),
            rect.y + layout.sy(0.111),
            "ASCENT",
            0.42,
            theme.bone_dim,
        );
        let cycle = ascent.cycle.to_string();
        Self::push_ui_text(
            verts,
            layout,
            rect.x + layout.sx(0.185),
            rect.y + layout.sy(0.101),
            &cycle,
            0.62,
            theme.gold_bright,
        );
        Self::push_ui_fit_text(
            verts,
            layout,
            rect.x + layout.sx(0.245),
            rect.y + layout.sy(0.109),
            &ascent.cycle_modifier,
            HudTextFit {
                scale: 0.46,
                max_width: layout.sx(0.225),
                color: theme.bone,
            },
        );

        Self::push_line(
            verts,
            [rect.x + layout.sx(0.025), rect.y + layout.sy(0.078)],
            [rect.right() - layout.sx(0.025), rect.y + layout.sy(0.078)],
            layout.sy(0.002),
            with_alpha(theme.line, 0.72),
        );
        Self::push_ui_fit_text(
            verts,
            layout,
            rect.x + layout.sx(0.028),
            rect.y + layout.sy(0.024),
            &ascent.relic_name,
            HudTextFit {
                scale: 0.50,
                max_width: layout.sx(0.290),
                color: theme.gold_bright,
            },
        );

        let ash_x = rect.x + layout.sx(0.370);
        Self::push_icon(
            verts,
            HudIcon::Ash,
            [ash_x, rect.y + layout.sy(0.043)],
            [layout.sx(0.030), layout.sy(0.040)],
            theme.ash,
        );
        Self::push_ui_text(
            verts,
            layout,
            ash_x + layout.sx(0.024),
            rect.y + layout.sy(0.043),
            "ASH",
            0.32,
            theme.bone_dim,
        );
        let ash_value = ascent.unsecured_resource.to_string();
        Self::push_ui_text(
            verts,
            layout,
            ash_x + layout.sx(0.085),
            rect.y + layout.sy(0.023),
            &ash_value,
            0.50,
            theme.bone,
        );

        let bank_x = rect.x + layout.sx(0.545);
        Self::push_icon(
            verts,
            HudIcon::Bank,
            [bank_x, rect.y + layout.sy(0.043)],
            [layout.sx(0.032), layout.sy(0.040)],
            theme.cold,
        );
        Self::push_ui_text(
            verts,
            layout,
            bank_x + layout.sx(0.024),
            rect.y + layout.sy(0.043),
            "BOUND",
            0.30,
            theme.bone_dim,
        );
        let bank_value = ascent.banked_resource.to_string();
        Self::push_ui_text(
            verts,
            layout,
            bank_x + layout.sx(0.095),
            rect.y + layout.sy(0.023),
            &bank_value,
            0.50,
            theme.cold,
        );
    }
}

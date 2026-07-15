//! Named encounter identity and health presentation.

use super::{with_alpha, HudLayout, HudSystem, HudTheme, HudVertex, NamedEncounterHudState};

impl HudSystem {
    pub(super) fn push_named_encounter(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        state: &NamedEncounterHudState,
    ) {
        if !state.active || state.name.trim().is_empty() {
            return;
        }

        let region = layout.boss;
        let health = state.health_ratio.clamp(0.0, 1.0);
        let name = Self::wrap_ui_lines(layout, &state.name, 0.34, region.w, 1);
        let bar_y = region.y + layout.sy(0.006);
        let bar_h = layout.sy(0.012);

        Self::push_ui_centered_text(
            verts,
            layout,
            region.center_x(),
            region.y + layout.sy(0.046),
            &name[0],
            0.34,
            theme.bone,
        );
        Self::push_rect(
            verts,
            region.x,
            bar_y,
            region.w,
            bar_h,
            with_alpha(theme.void, 0.92),
        );
        Self::push_rect(
            verts,
            region.x,
            bar_y,
            region.w * health,
            bar_h,
            theme.blood,
        );
        Self::push_line(
            verts,
            [region.x, bar_y + bar_h + layout.sy(0.005)],
            [region.right(), bar_y + bar_h + layout.sy(0.005)],
            layout.sy(0.0015),
            with_alpha(theme.gold, 0.52),
        );
    }
}

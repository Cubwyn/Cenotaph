//! Interaction prompt and dialogue presentation.

use super::{
    with_alpha, DialogueHudState, HudIcon, HudLayout, HudRect, HudSystem, HudTextFit, HudTheme,
    HudVertex,
};

impl HudSystem {
    pub(super) fn push_interaction_prompt(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        prompt: &str,
    ) {
        if prompt.is_empty() {
            return;
        }

        let rect = layout.interaction;
        Self::push_cut_panel(
            verts,
            rect,
            [layout.sx(0.014), layout.sy(0.014)],
            with_alpha(theme.surface, 0.92),
            with_alpha(theme.cold, 0.64),
        );
        let icon_center = [rect.x + layout.sx(0.050), rect.y + rect.h * 0.5];
        Self::push_icon(
            verts,
            HudIcon::Interact,
            icon_center,
            [layout.sx(0.058), layout.sy(0.070)],
            theme.cold,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            icon_center[0],
            icon_center[1] - layout.sy(0.015),
            "E",
            0.50,
            theme.bone,
        );
        Self::push_ui_fit_text(
            verts,
            layout,
            rect.x + layout.sx(0.098),
            rect.y + layout.sy(0.039),
            prompt,
            HudTextFit {
                scale: 0.55,
                max_width: rect.w - layout.sx(0.125),
                color: theme.bone,
            },
        );
    }

    pub(super) fn push_dialogue(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        dialogue: &DialogueHudState,
    ) {
        if dialogue.line.is_empty() {
            return;
        }

        let rect = layout.dialogue;
        Self::push_cut_panel(
            verts,
            rect,
            [layout.sx(0.018), layout.sy(0.018)],
            with_alpha(theme.surface, 0.96),
            theme.line,
        );
        Self::push_rect(
            verts,
            rect.x + layout.sx(0.018),
            rect.y + layout.sy(0.018),
            layout.sx(0.004),
            rect.h - layout.sy(0.036),
            theme.gold,
        );
        Self::push_icon(
            verts,
            HudIcon::Bell,
            [rect.x + layout.sx(0.055), rect.y + layout.sy(0.151)],
            [layout.sx(0.036), layout.sy(0.044)],
            theme.gold,
        );
        Self::push_ui_fit_text(
            verts,
            layout,
            rect.x + layout.sx(0.084),
            rect.y + layout.sy(0.145),
            &dialogue.speaker,
            HudTextFit {
                scale: 0.40,
                max_width: rect.w - layout.sx(0.180),
                color: theme.gold_bright,
            },
        );
        let line_width = rect.w - layout.sx(0.170);
        for (index, line) in Self::wrap_ui_lines(layout, &dialogue.line, 0.48, line_width, 2)
            .iter()
            .enumerate()
        {
            Self::push_ui_text(
                verts,
                layout,
                rect.x + layout.sx(0.048),
                rect.y + layout.sy(0.092 - index as f32 * 0.052),
                line,
                0.48,
                theme.bone,
            );
        }
        let key_rect = HudRect {
            x: rect.right() - layout.sx(0.080),
            y: rect.y + layout.sy(0.032),
            w: layout.sx(0.052),
            h: layout.sy(0.052),
        };
        Self::push_keycap(verts, layout, key_rect, "E", theme.cold, theme);
        Self::push_rect(
            verts,
            rect.x + layout.sx(0.018),
            rect.y + layout.sy(0.008),
            (rect.w - layout.sx(0.036)) * dialogue.remaining_ratio.clamp(0.0, 1.0),
            layout.sy(0.004),
            theme.cold,
        );
    }
}

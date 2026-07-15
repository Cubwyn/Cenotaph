//! Transient event feed and level-arrival title widgets.

use super::{
    with_alpha, HudFeedEvent, HudIcon, HudLayout, HudRect, HudSystem, HudTextFit, HudTheme,
    HudVertex, NamedNoticeHudState, MAX_EVENT_FEED_ITEMS,
};

impl HudSystem {
    pub(super) fn push_named_notice(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        state: &NamedNoticeHudState,
    ) {
        if !state.active || state.title.trim().is_empty() {
            return;
        }

        let ratio = state.remaining_ratio.clamp(0.0, 1.0);
        let progress = 1.0 - ratio;
        let alpha = (progress / 0.12).clamp(0.0, 1.0) * (ratio / 0.18).clamp(0.0, 1.0);
        if alpha <= 0.0 {
            return;
        }

        let region = layout.objective;
        let title = Self::wrap_ui_lines(layout, &state.title, 0.52, region.w, 1);
        let subtitle = Self::wrap_ui_lines(layout, &state.subtitle, 0.32, region.w, 1);
        let rail_y = region.y + layout.sy(0.018);
        Self::push_line(
            verts,
            [region.x, rail_y],
            [region.right(), rail_y],
            layout.sy(0.002),
            with_alpha(theme.gold, 0.62 * alpha),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            region.center_x(),
            region.y + layout.sy(0.066),
            &title[0],
            0.52,
            with_alpha(theme.bone, alpha),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            region.center_x(),
            region.y - layout.sy(0.002),
            &subtitle[0],
            0.32,
            with_alpha(theme.gold_bright, 0.9 * alpha),
        );
    }

    pub(super) fn push_event_feed(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        events: &[HudFeedEvent],
    ) {
        if events.is_empty() {
            return;
        }

        let region = layout.event_feed;
        let row_h = layout.sy(0.060);
        let gap = layout.sy(0.010);

        for (index, event) in events.iter().take(MAX_EVENT_FEED_ITEMS).enumerate() {
            let y = region.top() - row_h - index as f32 * (row_h + gap);
            let ratio = event.ratio.clamp(0.0, 1.0);
            let alpha = 0.20 + ratio * 0.80;
            let color = with_alpha(event.color, alpha);
            let row = HudRect {
                x: region.x,
                y,
                w: region.w,
                h: row_h,
            };

            Self::push_cut_panel(
                verts,
                row,
                [layout.sx(0.008), layout.sy(0.008)],
                with_alpha(theme.surface, 0.64 * ratio),
                with_alpha(theme.line, 0.42 * ratio),
            );
            Self::push_diamond(
                verts,
                [row.x + layout.sx(0.025), row.y + row.h * 0.5],
                [layout.sx(0.018), layout.sy(0.018)],
                color,
            );
            Self::push_ui_fit_text(
                verts,
                layout,
                row.x + layout.sx(0.050),
                row.y + layout.sy(0.015),
                event.label,
                HudTextFit {
                    scale: 0.46,
                    max_width: row.w - layout.sx(if event.has_value { 0.135 } else { 0.070 }),
                    color: with_alpha(theme.bone, alpha),
                },
            );

            if event.has_value {
                let value = event.value.to_string();
                let value_w = Self::ui_text_width(layout, &value, 0.50);
                Self::push_ui_text(
                    verts,
                    layout,
                    row.right() - value_w - layout.sx(0.025),
                    row.y + layout.sy(0.013),
                    &value,
                    0.50,
                    color,
                );
            }
        }
    }

    pub(super) fn push_level_arrival(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        ratio: f32,
        title: &str,
        subtitle: &str,
    ) {
        let ratio = ratio.clamp(0.0, 1.0);
        if ratio <= 0.0 || title.is_empty() {
            return;
        }
        let progress = 1.0 - ratio;
        let alpha = (progress / 0.16).clamp(0.0, 1.0) * (ratio / 0.22).clamp(0.0, 1.0);
        let y = 0.43 + (1.0 - alpha) * layout.sy(0.025);
        let line_width = layout.sx(0.78) * (0.72 + alpha * 0.28);

        Self::push_icon(
            verts,
            HudIcon::Bell,
            [0.0, y + layout.sy(0.125)],
            [layout.sx(0.050), layout.sy(0.060)],
            with_alpha(theme.gold, alpha),
        );
        Self::push_line(
            verts,
            [-line_width * 0.5, y - layout.sy(0.014)],
            [line_width * 0.5, y - layout.sy(0.014)],
            layout.sy(0.003),
            with_alpha(theme.gold, 0.54 * alpha),
        );
        Self::push_diamond(
            verts,
            [-line_width * 0.5, y - layout.sy(0.014)],
            [layout.sx(0.014), layout.sy(0.014)],
            with_alpha(theme.gold, alpha),
        );
        Self::push_diamond(
            verts,
            [line_width * 0.5, y - layout.sy(0.014)],
            [layout.sx(0.014), layout.sy(0.014)],
            with_alpha(theme.gold, alpha),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            y + layout.sy(0.030),
            title,
            0.92,
            with_alpha(theme.bone, alpha),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            0.0,
            y - layout.sy(0.078),
            subtitle,
            0.40,
            with_alpha(theme.ash, 0.88 * alpha),
        );
    }
}

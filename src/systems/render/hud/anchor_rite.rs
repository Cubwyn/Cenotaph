//! Modal presentation for the ritual choices made at an Anchor.

use super::{
    with_alpha, AnchorRiteHudState, HudIcon, HudLayout, HudRect, HudSystem, HudTheme, HudVertex,
};

const OPTION_TITLES: [&str; 3] = ["BIND THE CINDERS", "MEND THE VESSEL", "TURN FROM THE STONE"];

#[derive(Debug, Clone, Copy)]
struct AnchorRiteMetrics {
    options: [HudRect; 3],
    text_x: f32,
    text_width: f32,
}

impl AnchorRiteMetrics {
    fn new(layout: HudLayout) -> Self {
        let panel = layout.anchor_rite;
        let inset = layout.sx(0.035);
        let option_h = layout.sy(0.205);
        let option_gap = layout.sy(0.012);
        let first_top = panel.top() - layout.sy(0.335);
        let option_w = panel.w - inset * 2.0;
        let options = std::array::from_fn(|index| HudRect {
            x: panel.x + inset,
            y: first_top - option_h - index as f32 * (option_h + option_gap),
            w: option_w,
            h: option_h,
        });
        let text_x = options[0].x + layout.sx(0.082);
        let text_width = options[0].right() - text_x - layout.sx(0.035);

        Self {
            options,
            text_x,
            text_width,
        }
    }
}

fn selected_option(index: usize) -> usize {
    index.min(OPTION_TITLES.len() - 1)
}

fn option_available(index: usize, state: &AnchorRiteHudState) -> bool {
    match index {
        0 => state.can_bind,
        1 => state.can_mend && state.vessel_wounded,
        _ => true,
    }
}

fn option_consequence(index: usize, state: &AnchorRiteHudState) -> String {
    match index {
        0 if !state.can_bind => state.bind_requirement.clone(),
        0 => "CARRIED ASH BECOMES BOUND".to_owned(),
        1 if !state.vessel_wounded => "THE VESSEL BEARS NO WOUND".to_owned(),
        1 if !state.can_mend => format!("REQUIRES {} BOUND ASH", state.mend_cost),
        1 => format!("SPEND {} BOUND ASH TO CLOSE THE WOUND", state.mend_cost),
        _ => "LEAVE THE ASH UNBOUND AND RESUME THE ASCENT".to_owned(),
    }
}

impl HudSystem {
    pub(super) fn push_anchor_rite(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        theme: HudTheme,
        state: &AnchorRiteHudState,
    ) {
        if !state.active {
            return;
        }

        Self::push_rect(verts, -1.0, -1.0, 2.0, 2.0, [0.0, 0.0, 0.0, 0.72]);

        let panel = layout.anchor_rite;
        let metrics = AnchorRiteMetrics::new(layout);
        Self::push_cut_panel(
            verts,
            panel,
            [layout.sx(0.026), layout.sy(0.026)],
            with_alpha(theme.surface, 0.98),
            with_alpha(theme.gold, 0.78),
        );

        let header_y = panel.top() - layout.sy(0.080);
        let title_width = Self::ui_text_width(layout, "ANCHOR RITE", 0.44);
        let title_rail = layout.sx(0.10);
        let rail_gap = layout.sx(0.025);
        Self::push_line(
            verts,
            [
                panel.center_x() - title_width * 0.5 - rail_gap - title_rail,
                header_y,
            ],
            [panel.center_x() - title_width * 0.5 - rail_gap, header_y],
            layout.sy(0.0025),
            with_alpha(theme.gold, 0.52),
        );
        Self::push_line(
            verts,
            [panel.center_x() + title_width * 0.5 + rail_gap, header_y],
            [
                panel.center_x() + title_width * 0.5 + rail_gap + title_rail,
                header_y,
            ],
            layout.sy(0.0025),
            with_alpha(theme.gold, 0.52),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            panel.center_x(),
            header_y - layout.sy(0.014),
            "ANCHOR RITE",
            0.44,
            theme.gold_bright,
        );

        let anchor_name = if state.anchor_name.trim().is_empty() {
            "THE NAMELESS ANCHOR".to_owned()
        } else {
            state.anchor_name.trim().to_owned()
        };
        let name_width = panel.w - layout.sx(0.16);
        let fitted_name = Self::wrap_ui_lines(layout, &anchor_name, 0.62, name_width, 1);
        Self::push_ui_centered_text(
            verts,
            layout,
            panel.center_x(),
            panel.top() - layout.sy(0.184),
            &fitted_name[0],
            0.62,
            theme.bone,
        );

        let ledger_y = panel.top() - layout.sy(0.270);
        let carried = format!("ASH {}", state.carried_ash);
        let bound = format!("BOUND {}", state.bound_ash);
        let carried_center = panel.center_x() - panel.w * 0.22;
        let bound_center = panel.center_x() + panel.w * 0.22;
        Self::push_icon(
            verts,
            HudIcon::Ash,
            [
                carried_center - layout.sx(0.072),
                ledger_y + layout.sy(0.012),
            ],
            [layout.sx(0.036), layout.sy(0.042)],
            theme.ash,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            carried_center + layout.sx(0.018),
            ledger_y,
            &carried,
            0.40,
            theme.bone,
        );
        Self::push_icon(
            verts,
            HudIcon::Bank,
            [bound_center - layout.sx(0.082), ledger_y + layout.sy(0.012)],
            [layout.sx(0.038), layout.sy(0.044)],
            theme.cold,
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            bound_center + layout.sx(0.018),
            ledger_y,
            &bound,
            0.40,
            theme.cold,
        );

        Self::push_line(
            verts,
            [panel.x + layout.sx(0.050), panel.top() - layout.sy(0.315)],
            [
                panel.right() - layout.sx(0.050),
                panel.top() - layout.sy(0.315),
            ],
            layout.sy(0.002),
            with_alpha(theme.line, 0.82),
        );

        let selected = selected_option(state.selected_option);
        for (index, rect) in metrics.options.iter().copied().enumerate() {
            let is_selected = index == selected;
            let available = option_available(index, state);
            let icon = match index {
                0 => HudIcon::Ash,
                1 => HudIcon::Vital,
                _ => HudIcon::Interact,
            };
            let title_color = if !available {
                theme.bone_dim
            } else if is_selected {
                theme.gold_bright
            } else {
                theme.bone
            };
            let icon_color = if !available {
                with_alpha(theme.ash, 0.55)
            } else if is_selected {
                theme.gold
            } else {
                theme.bone_dim
            };

            if is_selected {
                Self::push_rect(
                    verts,
                    rect.x,
                    rect.y,
                    rect.w,
                    rect.h,
                    with_alpha(theme.surface_raised, 0.88),
                );
                Self::push_rect(verts, rect.x, rect.y, layout.sx(0.006), rect.h, theme.gold);
                Self::push_diamond(
                    verts,
                    [rect.right() - layout.sx(0.026), rect.y + rect.h * 0.5],
                    [layout.sx(0.018), layout.sy(0.025)],
                    theme.gold_bright,
                );
            }

            Self::push_line(
                verts,
                [rect.x, rect.y],
                [rect.right(), rect.y],
                layout.sy(0.0018),
                if is_selected {
                    with_alpha(theme.gold, 0.62)
                } else {
                    with_alpha(theme.line, 0.58)
                },
            );
            Self::push_icon(
                verts,
                icon,
                [rect.x + layout.sx(0.040), rect.y + rect.h * 0.52],
                [layout.sx(0.042), layout.sy(0.052)],
                icon_color,
            );
            Self::push_ui_text(
                verts,
                layout,
                metrics.text_x,
                rect.y + layout.sy(0.122),
                OPTION_TITLES[index],
                0.48,
                title_color,
            );

            if index != 2 && !available {
                let unavailable = "UNAVAILABLE";
                let status_width = Self::ui_text_width(layout, unavailable, 0.30);
                Self::push_ui_text(
                    verts,
                    layout,
                    rect.right() - layout.sx(0.035) - status_width,
                    rect.y + layout.sy(0.128),
                    unavailable,
                    0.30,
                    with_alpha(theme.ash, 0.78),
                );
            }

            let consequence = option_consequence(index, state);
            let fitted = Self::wrap_ui_lines(layout, &consequence, 0.36, metrics.text_width, 1);
            Self::push_ui_text(
                verts,
                layout,
                metrics.text_x,
                rect.y + layout.sy(0.052),
                &fitted[0],
                0.36,
                if available {
                    theme.bone_dim
                } else {
                    with_alpha(theme.ash, 0.72)
                },
            );
        }

        Self::push_ui_centered_text(
            verts,
            layout,
            panel.center_x(),
            panel.y + layout.sy(0.085),
            "THE STONE REMEMBERS",
            0.32,
            with_alpha(theme.bone_dim, 0.72),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_indices_and_labels_are_stable() {
        assert_eq!(OPTION_TITLES[0], "BIND THE CINDERS");
        assert_eq!(OPTION_TITLES[1], "MEND THE VESSEL");
        assert_eq!(OPTION_TITLES[2], "TURN FROM THE STONE");
        assert_eq!(selected_option(usize::MAX), 2);
    }

    #[test]
    fn mend_availability_and_consequence_reflect_vessel_state() {
        let ready = AnchorRiteHudState {
            mend_cost: 12,
            can_mend: true,
            vessel_wounded: true,
            ..AnchorRiteHudState::default()
        };
        assert!(option_available(1, &ready));
        assert_eq!(
            option_consequence(1, &ready),
            "SPEND 12 BOUND ASH TO CLOSE THE WOUND"
        );

        let underfunded = AnchorRiteHudState {
            can_mend: false,
            ..ready.clone()
        };
        assert!(!option_available(1, &underfunded));
        assert_eq!(option_consequence(1, &underfunded), "REQUIRES 12 BOUND ASH");

        let healed = AnchorRiteHudState {
            vessel_wounded: false,
            ..ready
        };
        assert!(!option_available(1, &healed));
        assert_eq!(option_consequence(1, &healed), "THE VESSEL BEARS NO WOUND");
    }

    #[test]
    fn bind_option_exposes_an_unmet_ritual_requirement() {
        let blocked = AnchorRiteHudState {
            can_bind: false,
            bind_requirement: "REQUIRES LAST CHAIN BROKEN".to_string(),
            ..AnchorRiteHudState::default()
        };

        assert!(!option_available(0, &blocked));
        assert_eq!(
            option_consequence(0, &blocked),
            "REQUIRES LAST CHAIN BROKEN"
        );
    }

    #[test]
    fn option_rows_are_equal_and_contained_at_common_viewports() {
        for viewport in [[800, 600], [1280, 720], [1920, 1080], [2560, 1080]] {
            let layout = HudLayout::new(viewport);
            let panel = layout.anchor_rite;
            let metrics = AnchorRiteMetrics::new(layout);
            let expected = metrics.options[0];

            for rect in metrics.options {
                assert_eq!(rect.w, expected.w, "{viewport:?}");
                assert_eq!(rect.h, expected.h, "{viewport:?}");
                assert!(
                    rect.x >= panel.x && rect.right() <= panel.right(),
                    "{viewport:?}"
                );
                assert!(
                    rect.y >= panel.y && rect.top() <= panel.top(),
                    "{viewport:?}"
                );
            }
        }
    }

    #[test]
    fn fixed_option_copy_fits_the_narrowest_supported_modal() {
        for viewport in [[800, 600], [2560, 1080]] {
            let layout = HudLayout::new(viewport);
            let metrics = AnchorRiteMetrics::new(layout);
            for title in OPTION_TITLES {
                assert!(
                    HudSystem::ui_text_width(layout, title, 0.48) <= metrics.text_width,
                    "{viewport:?}: {title}"
                );
            }
        }
    }
}

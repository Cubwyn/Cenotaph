//! HUD state assembly — pure read-only mapping from engine state to display structs.
//!
//! Every method here reads `EngineState` and produces a small display struct.
//! None of them mutate engine state.

use glam::Vec3;

use crate::core::engine::state::EngineState;
use crate::systems::render::hud::{
    AnchorRiteHudState, AscentHudState, DebugHudState, DialogueHudState, HudFeedEvent,
    HudMarkerKind, HudMarkerState, HudWorldMarker, NamedEncounterHudState,
    NamedNoticeHudState,
};
use crate::game::feedback::{FeedbackEvent, FeedbackEventKind};

pub(super) fn round_to_u32(value: f32) -> u32 {
    value.max(0.0).round().min(u32::MAX as f32) as u32
}

pub(super) fn hud_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '/') {
                character.to_ascii_uppercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl EngineState {
    pub(super) fn debug_hud_state(&self) -> DebugHudState {
        let enemies = self
            .level_data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .count() as u32;
        let loot = self
            .level_data
            .props
            .iter()
            .filter(|prop| {
                prop.resource_value > 0
                    || prop
                        .item_id
                        .as_ref()
                        .is_some_and(|item_id| !item_id.trim().is_empty())
            })
            .count() as u32;

        DebugHudState {
            enabled: self.debug_hud_enabled,
            enemies,
            loot,
            unsecured_resource: self.progress.unsecured_resource,
            banked_resource: self.progress.banked_resource,
            cycle: self.cycle.number,
            props: self.level_data.props.len() as u32,
            fps: if self.frame_time_ms > 0.0 {
                round_to_u32(1000.0 / self.frame_time_ms)
            } else {
                0
            },
            frame_ms: round_to_u32(self.frame_time_ms),
        }
    }

    pub(super) fn ascent_hud_state(&self) -> AscentHudState {
        let current_relic = self.equipped_relic.current();

        AscentHudState {
            cycle: self.cycle.number,
            cycle_modifier: self.cycle.modifier.display_label().to_string(),
            relic_name: current_relic
                .map(|relic| relic.display_name.clone())
                .unwrap_or_else(|| "UNCLAIMED".to_string()),
            unsecured_resource: self.progress.unsecured_resource,
            banked_resource: self.progress.banked_resource,
        }
    }

    pub(super) fn hud_event_feed(&self) -> Vec<HudFeedEvent> {
        self.feedback
            .events
            .iter()
            .filter(|event| event.is_active())
            .filter_map(Self::hud_event_for_feedback)
            .collect()
    }

    pub(super) fn interaction_prompt(&self) -> String {
        use crate::core::engine::state::GameMode;

        if self.player.is_dead
            || self.game_mode != GameMode::Playing
            || self.active_anchor_rite.is_some()
        {
            return String::new();
        }

        let player = Vec3::from_array(self.physics.get_player_pos());
        if Self::nearest_interact_event_index(
            &self.level_data.events,
            &self.level_data.props,
            &self.level_event_fired,
            player,
            &self.level_flags,
        )
        .is_some()
        {
            return "INTERACT".to_string();
        }

        Self::nearest_anchor_prop_index(
            &self.level_data.props,
            player,
            self.config_data.world.anchor_interaction_radius,
        )
        .map(|_| "COMMUNE WITH ANCHOR".to_string())
        .unwrap_or_default()
    }

    pub(super) fn anchor_rite_hud_state(&self) -> AnchorRiteHudState {
        use crate::core::engine::state::ManualLevelEventStatus;

        let Some(rite) = self.active_anchor_rite.as_ref() else {
            return AnchorRiteHudState::default();
        };
        let mend_cost = self.config_data.world.anchor_mend_cost;
        let newly_activated =
            self.progress.active_anchor_id.as_deref() != Some(rite.anchor_id.as_str());
        let bind_event_status = newly_activated
            .then_some(rite.event_id.as_deref())
            .flatten()
            .map(|event_id| self.manual_level_event_status(event_id));
        let (can_bind, bind_requirement) = match bind_event_status {
            Some(ManualLevelEventStatus::MissingFlag(flag_id)) => (
                false,
                format!("REQUIRES {}", hud_text(&flag_id.replace('_', " "))),
            ),
            Some(
                ManualLevelEventStatus::MissingEvent | ManualLevelEventStatus::WrongTrigger(_),
            ) => (false, "RITE UNAVAILABLE".to_string()),
            Some(ManualLevelEventStatus::Ready | ManualLevelEventStatus::AlreadyFired) | None => {
                (true, String::new())
            }
        };

        AnchorRiteHudState {
            active: true,
            anchor_name: hud_text(&rite.display_name),
            selected_option: rite.selected_option,
            carried_ash: self.progress.unsecured_resource,
            bound_ash: self.progress.banked_resource,
            mend_cost,
            can_bind,
            bind_requirement,
            can_mend: self.progress.banked_resource >= mend_cost
                && self.player.health.current < self.player.health.max,
            vessel_wounded: self.player.health.current < self.player.health.max,
        }
    }

    pub(super) fn dialogue_hud_state(&self) -> DialogueHudState {
        self.active_dialogue
            .as_ref()
            .map(crate::core::engine::state::ActiveDialogueState::hud_state)
            .unwrap_or_default()
    }

    pub(super) fn named_notice_hud_state(&self) -> NamedNoticeHudState {
        self.feedback
            .named_notice
            .as_ref()
            .map(|notice| NamedNoticeHudState {
                active: true,
                title: hud_text(&notice.title),
                subtitle: hud_text(&notice.subtitle),
                remaining_ratio: notice.remaining_ratio(),
            })
            .unwrap_or_default()
    }

    pub(super) fn named_encounter_hud_state(&self) -> NamedEncounterHudState {
        let player = Vec3::from_array(self.physics.get_player_pos());
        self.level_data
            .props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                if prop.enemy_type.is_none() || prop.enemy_health <= 0.0 {
                    return None;
                }
                let display_name = prop
                    .display_name
                    .as_deref()
                    .filter(|name| !name.trim().is_empty())?;
                let (state, _) =
                    self.marker_state_for_prop(index, prop, HudMarkerKind::Enemy, player);
                if state == HudMarkerState::Neutral {
                    return None;
                }
                let distance = player.distance(Vec3::from_array(prop.position));
                let health_ratio = self.marker_ratio_for_prop(index, prop, HudMarkerKind::Enemy);
                Some((distance, display_name, health_ratio))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, display_name, health_ratio)| NamedEncounterHudState {
                active: true,
                name: hud_text(display_name),
                health_ratio,
            })
            .unwrap_or_default()
    }

    fn hud_event_for_feedback(event: &FeedbackEvent) -> Option<HudFeedEvent> {
        let (label, color) = match event.kind {
            FeedbackEventKind::PlayerDamage => ("DMG", [1.0, 0.16, 0.08, 1.0]),
            FeedbackEventKind::EnemyHit => ("HIT", [1.0, 0.38, 0.14, 1.0]),
            FeedbackEventKind::EnemyKill => ("KILL", [1.0, 0.74, 0.18, 1.0]),
            FeedbackEventKind::ShotBlocked => ("BLOCK", [0.48, 0.78, 1.0, 1.0]),
            FeedbackEventKind::ShotMissed => ("MISS", [0.76, 0.80, 0.86, 1.0]),
            FeedbackEventKind::Pickup => ("PICKUP", [1.0, 0.78, 0.18, 1.0]),
            FeedbackEventKind::Resource => ("RES", [0.84, 0.82, 1.0, 1.0]),
            FeedbackEventKind::Heal => ("HEAL", [0.28, 1.0, 0.48, 1.0]),
            FeedbackEventKind::Spawn => ("SPAWN", [0.36, 1.0, 0.48, 1.0]),
            FeedbackEventKind::Reload => ("RELOAD", [0.45, 0.75, 1.0, 1.0]),
            FeedbackEventKind::Loot => ("LOOT", [1.0, 0.80, 0.24, 1.0]),
            FeedbackEventKind::Relic => ("RELIC", [1.0, 0.80, 0.24, 1.0]),
            FeedbackEventKind::Debug => ("DEBUG", [0.2, 0.85, 1.0, 1.0]),
            FeedbackEventKind::Death => ("DEATH", [1.0, 0.12, 0.08, 1.0]),
            FeedbackEventKind::None => return None,
        };

        Some(HudFeedEvent {
            label,
            value: event.value,
            has_value: event.value > 0,
            ratio: event.remaining_ratio(),
            color,
        })
    }

    pub(super) fn hud_world_markers(&self) -> Vec<HudWorldMarker> {
        let player_pos = self.physics.get_player_pos();
        let player = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let mut markers = Vec::new();

        for (index, prop) in self.level_data.props.iter().enumerate() {
            let Some(kind) = self.marker_kind_for_prop(prop) else {
                continue;
            };

            let marker_height = prop.scale[1].abs().max(1.0) * 0.75 + 0.65;
            let world_pos = Vec3::new(
                prop.position[0],
                prop.position[1] + marker_height,
                prop.position[2],
            );
            let Some(screen_pos) = self.project_to_hud(world_pos) else {
                continue;
            };
            let distance = player.distance(world_pos);
            let (state, state_ratio) = self.marker_state_for_prop(index, prop, kind, player);
            if kind == HudMarkerKind::Enemy && state == HudMarkerState::Neutral {
                continue;
            }
            markers.push((
                distance,
                HudWorldMarker {
                    screen_pos,
                    ratio: self.marker_ratio_for_prop(index, prop, kind),
                    distance_m: round_to_u32(distance),
                    kind,
                    state,
                    state_ratio,
                },
            ));
        }

        markers.sort_by(|left, right| left.0.total_cmp(&right.0));
        markers
            .into_iter()
            .take(64)
            .map(|(_, marker)| marker)
            .collect()
    }

    fn marker_kind_for_prop(&self, prop: &crate::data::world::level::PropData) -> Option<HudMarkerKind> {
        if prop.enemy_type.is_some() && prop.enemy_health > 0.0 {
            Some(HudMarkerKind::Enemy)
        } else if prop.is_hurtbox {
            Some(HudMarkerKind::Hazard)
        } else if prop.resource_value > 0
            || prop
                .item_id
                .as_ref()
                .is_some_and(|item_id| !item_id.trim().is_empty())
        {
            Some(HudMarkerKind::Loot)
        } else if prop
            .anchor_id
            .as_ref()
            .is_some_and(|anchor_id| !anchor_id.trim().is_empty())
        {
            Some(HudMarkerKind::Anchor)
        } else {
            None
        }
    }

    fn marker_state_for_prop(
        &self,
        index: usize,
        prop: &crate::data::world::level::PropData,
        kind: HudMarkerKind,
        player: Vec3,
    ) -> (HudMarkerState, f32) {
        if kind != HudMarkerKind::Enemy {
            return (HudMarkerState::Neutral, 0.0);
        }

        let Some(enemy_type) = prop.enemy_type.as_deref() else {
            return (HudMarkerState::Neutral, 0.0);
        };
        let Some(enemy) = self.enemy_registry.get(enemy_type) else {
            return (HudMarkerState::Neutral, 0.0);
        };

        if let Some(runtime) = self.enemy_runtime.get(index) {
            if runtime.stagger_remaining > 0.0 {
                let duration = self.config_data.combat.enemy_hit_stun.max(0.001);
                return (
                    HudMarkerState::Staggered,
                    (runtime.stagger_remaining / duration).clamp(0.0, 1.0),
                );
            }

            if runtime.attack_windup_remaining > 0.0 {
                let windup = enemy.attack_windup.max(0.001);
                return (
                    HudMarkerState::Windup,
                    (1.0 - runtime.attack_windup_remaining / windup).clamp(0.0, 1.0),
                );
            }
        }

        let enemy_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
        let horizontal_delta = Vec3::new(player.x - enemy_pos.x, 0.0, player.z - enemy_pos.z);
        let activation_range = enemy.activation_range.max(0.001);
        let distance_ratio = (horizontal_delta.length() / activation_range).clamp(0.0, 1.0);

        if distance_ratio < 1.0 {
            (HudMarkerState::Aggro, 1.0 - distance_ratio)
        } else {
            (HudMarkerState::Neutral, 0.0)
        }
    }

    fn marker_ratio_for_prop(
        &self,
        index: usize,
        prop: &crate::data::world::level::PropData,
        kind: HudMarkerKind,
    ) -> f32 {
        match kind {
            HudMarkerKind::Enemy => {
                let fallback_max_health = prop
                    .enemy_type
                    .as_deref()
                    .and_then(|enemy_type| self.enemy_registry.get(enemy_type))
                    .map_or(1.0, |enemy| enemy.health);
                self.enemy_runtime.get(index).map_or_else(
                    || (prop.enemy_health / fallback_max_health.max(1.0)).clamp(0.0, 1.0),
                    |runtime| runtime.health_ratio(prop.enemy_health, fallback_max_health),
                )
            }
            _ => 1.0,
        }
    }

    fn project_to_hud(&self, world_pos: Vec3) -> Option<[f32; 2]> {
        let clip = self.camera.build_view_projection_matrix() * world_pos.extend(1.0);
        if clip.w <= 0.01 {
            return None;
        }

        let ndc = clip.truncate() / clip.w;
        if ndc.x.abs() > 1.2 || ndc.y.abs() > 1.2 {
            return None;
        }

        Some([ndc.x.clamp(-0.97, 0.97), ndc.y.clamp(-0.92, 0.92)])
    }
}

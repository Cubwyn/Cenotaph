use glam::Vec3;

pub const FEEDBACK_EVENT_CAPACITY: usize = 5;
const FEEDBACK_EVENT_DURATION: f32 = 4.5;
const LEVEL_ARRIVAL_DURATION: f32 = 3.4;
const NAMED_NOTICE_DURATION: f32 = 5.2;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NamedNotice {
    pub title: String,
    pub subtitle: String,
    pub timer: f32,
    pub duration: f32,
}

impl NamedNotice {
    pub fn remaining_ratio(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.timer / self.duration).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FeedbackEventKind {
    #[default]
    None,
    PlayerDamage,
    EnemyHit,
    EnemyKill,
    ShotBlocked,
    ShotMissed,
    Pickup,
    Resource,
    Heal,
    Spawn,
    Reload,
    Loot,
    Relic,
    Debug,
    Death,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeedbackEvent {
    pub kind: FeedbackEventKind,
    pub value: u32,
    pub timer: f32,
    pub duration: f32,
}

impl FeedbackEvent {
    pub fn remaining_ratio(&self) -> f32 {
        if self.duration <= 0.0 {
            0.0
        } else {
            (self.timer / self.duration).clamp(0.0, 1.0)
        }
    }

    pub fn is_active(&self) -> bool {
        self.kind != FeedbackEventKind::None && self.timer > 0.0
    }
}

impl Default for FeedbackEvent {
    fn default() -> Self {
        Self {
            kind: FeedbackEventKind::None,
            value: 0,
            timer: 0.0,
            duration: 0.0,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FeedbackState {
    pub time: f32,
    pub shot_flash_timer: f32,
    pub hit_marker_timer: f32,
    pub kill_marker_timer: f32,
    pub blocked_flash_timer: f32,
    pub miss_flash_timer: f32,
    pub pickup_flash_timer: f32,
    pub damage_flash_timer: f32,
    pub debug_flash_timer: f32,
    pub spawn_flash_timer: f32,
    pub reload_flash_timer: f32,
    pub loot_flash_timer: f32,
    pub heal_flash_timer: f32,
    pub level_arrival_timer: f32,
    pub named_notice: Option<NamedNotice>,
    pub events: [FeedbackEvent; FEEDBACK_EVENT_CAPACITY],
    shake_timer: f32,
    shake_duration: f32,
    shake_strength: f32,
    stride_phase: f32,
    stride_weight: f32,
    movement_speed_ratio: f32,
    landing_timer: f32,
    landing_duration: f32,
    landing_strength: f32,
    recoil_timer: f32,
    recoil_duration: f32,
    recoil_strength: f32,
    motion_fov: f32,
    was_grounded: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MotionSample {
    pub horizontal_speed: f32,
    pub walk_speed: f32,
    pub grounded: bool,
    pub sprinting: bool,
    pub dashing: bool,
    pub landing_speed: f32,
}

impl FeedbackState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        self.time += dt;
        self.shot_flash_timer = decay(self.shot_flash_timer, dt);
        self.hit_marker_timer = decay(self.hit_marker_timer, dt);
        self.kill_marker_timer = decay(self.kill_marker_timer, dt);
        self.blocked_flash_timer = decay(self.blocked_flash_timer, dt);
        self.miss_flash_timer = decay(self.miss_flash_timer, dt);
        self.pickup_flash_timer = decay(self.pickup_flash_timer, dt);
        self.damage_flash_timer = decay(self.damage_flash_timer, dt);
        self.debug_flash_timer = decay(self.debug_flash_timer, dt);
        self.spawn_flash_timer = decay(self.spawn_flash_timer, dt);
        self.reload_flash_timer = decay(self.reload_flash_timer, dt);
        self.loot_flash_timer = decay(self.loot_flash_timer, dt);
        self.heal_flash_timer = decay(self.heal_flash_timer, dt);
        self.level_arrival_timer = decay(self.level_arrival_timer, dt);
        if let Some(notice) = self.named_notice.as_mut() {
            notice.timer = decay(notice.timer, dt);
            if notice.timer <= 0.0 {
                self.named_notice = None;
            }
        }
        self.shake_timer = decay(self.shake_timer, dt);
        self.landing_timer = decay(self.landing_timer, dt);
        self.recoil_timer = decay(self.recoil_timer, dt);
        for event in &mut self.events {
            if !event.is_active() {
                continue;
            }

            event.timer = decay(event.timer, dt);
            if event.timer <= 0.0 {
                *event = FeedbackEvent::default();
            }
        }
        if self.shake_timer <= 0.0 {
            self.shake_duration = 0.0;
            self.shake_strength = 0.0;
        }
        if self.landing_timer <= 0.0 {
            self.landing_duration = 0.0;
            self.landing_strength = 0.0;
        }
        if self.recoil_timer <= 0.0 {
            self.recoil_duration = 0.0;
            self.recoil_strength = 0.0;
        }
    }

    pub fn on_fire(&mut self) {
        self.shot_flash_timer = self.shot_flash_timer.max(0.08);
        self.add_shake(0.025, 0.08);
        self.recoil_timer = 0.13;
        self.recoil_duration = 0.13;
        self.recoil_strength = 0.020;
    }

    pub fn on_dash(&mut self) {
        self.add_shake(0.045, 0.14);
        self.motion_fov = self.motion_fov.max(0.055);
    }

    pub fn on_enemy_hit_amount(&mut self, amount: f32) {
        self.hit_marker_timer = self.hit_marker_timer.max(0.18);
        self.add_shake(0.045, 0.12);
        self.push_event(FeedbackEventKind::EnemyHit, rounded_amount(amount));
    }

    pub fn on_enemy_kill_amount(&mut self, amount: f32) {
        self.hit_marker_timer = self.hit_marker_timer.max(0.15);
        self.kill_marker_timer = self.kill_marker_timer.max(0.35);
        self.add_shake(0.08, 0.18);
        self.push_event(FeedbackEventKind::EnemyKill, rounded_amount(amount));
    }

    pub fn on_shot_blocked(&mut self) {
        self.blocked_flash_timer = self.blocked_flash_timer.max(0.32);
        self.add_shake(0.035, 0.10);
        self.push_event(FeedbackEventKind::ShotBlocked, 0);
    }

    pub fn on_shot_missed(&mut self) {
        self.miss_flash_timer = self.miss_flash_timer.max(0.22);
        self.push_event(FeedbackEventKind::ShotMissed, 0);
    }

    pub fn on_pickup(&mut self) {
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.35);
        self.add_shake(0.03, 0.12);
        self.push_event(FeedbackEventKind::Pickup, 0);
    }

    pub fn on_resource_pickup(&mut self, amount: u32) {
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.35);
        self.add_shake(0.03, 0.12);
        self.push_event(FeedbackEventKind::Resource, amount);
    }

    pub fn on_relic_changed(&mut self) {
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.35);
        self.loot_flash_timer = self.loot_flash_timer.max(0.5);
        self.add_shake(0.045, 0.16);
        self.push_event(FeedbackEventKind::Relic, 0);
    }

    pub fn on_relic_acquired(&mut self, display_name: &str, rarity: &str, outcome: &str) {
        self.on_relic_changed();
        self.named_notice = Some(NamedNotice {
            title: display_name.trim().to_string(),
            subtitle: format!("{} RELIC / {}", rarity.trim(), outcome.trim()),
            timer: NAMED_NOTICE_DURATION,
            duration: NAMED_NOTICE_DURATION,
        });
    }

    pub fn on_heal(&mut self) {
        self.heal_flash_timer = self.heal_flash_timer.max(0.45);
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.25);
        self.add_shake(0.035, 0.12);
        self.push_event(FeedbackEventKind::Heal, 0);
    }

    pub fn on_player_damage_amount(&mut self, amount: f32) {
        self.damage_flash_timer = self.damage_flash_timer.max(0.45);
        self.add_shake(0.11, 0.22);
        self.push_event(FeedbackEventKind::PlayerDamage, rounded_amount(amount));
    }

    pub fn on_death(&mut self) {
        self.damage_flash_timer = self.damage_flash_timer.max(0.8);
        self.add_shake(0.18, 0.45);
        self.push_event(FeedbackEventKind::Death, 0);
    }

    pub fn on_transition(&mut self) {
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.25);
        self.add_shake(0.06, 0.2);
        self.on_level_enter();
    }

    pub fn on_level_enter(&mut self) {
        self.level_arrival_timer = LEVEL_ARRIVAL_DURATION;
    }

    pub fn level_arrival_ratio(&self) -> f32 {
        (self.level_arrival_timer / LEVEL_ARRIVAL_DURATION).clamp(0.0, 1.0)
    }

    pub fn on_debug(&mut self) {
        self.debug_flash_timer = self.debug_flash_timer.max(0.38);
        self.add_shake(0.02, 0.08);
        self.push_event(FeedbackEventKind::Debug, 0);
    }

    pub fn on_debug_spawn_count(&mut self, count: u32) {
        self.spawn_flash_timer = self.spawn_flash_timer.max(0.5);
        self.add_shake(0.04, 0.14);
        self.push_event(FeedbackEventKind::Spawn, count);
    }

    pub fn on_debug_reload(&mut self) {
        self.reload_flash_timer = self.reload_flash_timer.max(0.65);
        self.add_shake(0.075, 0.22);
        self.push_event(FeedbackEventKind::Reload, 0);
    }

    pub fn on_debug_loot_count(&mut self, count: u32) {
        self.loot_flash_timer = self.loot_flash_timer.max(0.5);
        self.pickup_flash_timer = self.pickup_flash_timer.max(0.25);
        self.add_shake(0.035, 0.14);
        self.push_event(FeedbackEventKind::Loot, count);
    }

    pub fn update_motion(&mut self, dt: f32, sample: MotionSample) {
        let dt = dt.clamp(0.0, 0.1);
        let speed_ratio = if sample.walk_speed > 0.001 {
            (sample.horizontal_speed / sample.walk_speed).clamp(0.0, 1.8)
        } else {
            0.0
        };
        let moving = sample.grounded && speed_ratio > 0.08;
        let target_weight = if moving { 1.0 } else { 0.0 };
        let weight_lerp = 1.0 - (-dt * if moving { 12.0 } else { 8.0 }).exp();
        self.stride_weight += (target_weight - self.stride_weight) * weight_lerp;
        self.movement_speed_ratio +=
            (speed_ratio - self.movement_speed_ratio) * (1.0 - (-dt * 9.0).exp());

        if moving {
            let cadence = if sample.sprinting { 11.5 } else { 8.2 };
            self.stride_phase = (self.stride_phase + dt * cadence * speed_ratio.clamp(0.55, 1.65))
                .rem_euclid(std::f32::consts::TAU);
        }

        if sample.grounded && !self.was_grounded && sample.landing_speed > 2.0 {
            self.landing_duration = 0.30;
            self.landing_timer = self.landing_duration;
            self.landing_strength = ((sample.landing_speed - 2.0) / 9.0).clamp(0.15, 1.0);
            self.add_shake(0.025 + self.landing_strength * 0.035, 0.14);
        }
        self.was_grounded = sample.grounded;

        let target_fov = if sample.dashing {
            0.080
        } else if sample.sprinting && moving {
            0.035
        } else {
            0.0
        };
        self.motion_fov += (target_fov - self.motion_fov) * (1.0 - (-dt * 8.5).exp());
    }

    pub fn camera_offset(&self, yaw: f32) -> Vec3 {
        let right = Vec3::new(-yaw.sin(), 0.0, yaw.cos());
        let mut offset = Vec3::ZERO;

        if self.shake_timer > 0.0 && self.shake_duration > 0.0 {
            let remaining = (self.shake_timer / self.shake_duration).clamp(0.0, 1.0);
            let amplitude = self.shake_strength * remaining * remaining;
            let phase = self.time * 71.0;
            offset += right * (phase.sin() * amplitude);
            offset += Vec3::Y * ((phase * 1.37).cos() * amplitude * 0.55);
        }

        let bob_strength = self.stride_weight * self.movement_speed_ratio.clamp(0.0, 1.35);
        offset += right * (self.stride_phase.sin() * 0.012 * bob_strength);
        offset += Vec3::Y * ((self.stride_phase * 2.0).cos() * 0.010 * bob_strength);

        if self.landing_timer > 0.0 && self.landing_duration > 0.0 {
            let progress = 1.0 - (self.landing_timer / self.landing_duration).clamp(0.0, 1.0);
            offset -=
                Vec3::Y * (progress * std::f32::consts::PI).sin() * self.landing_strength * 0.065;
        }

        offset
    }

    pub fn camera_rotation_offset(&self) -> [f32; 2] {
        let bob_strength = self.stride_weight * self.movement_speed_ratio.clamp(0.0, 1.35);
        let mut yaw = self.stride_phase.sin() * 0.0018 * bob_strength;
        let mut pitch = (self.stride_phase * 2.0).cos() * 0.0022 * bob_strength;

        if self.recoil_timer > 0.0 && self.recoil_duration > 0.0 {
            let remaining = (self.recoil_timer / self.recoil_duration).clamp(0.0, 1.0);
            pitch += self.recoil_strength * remaining * remaining;
            yaw += (self.time * 47.0).sin() * self.recoil_strength * remaining * 0.12;
        }
        if self.landing_timer > 0.0 && self.landing_duration > 0.0 {
            let progress = 1.0 - (self.landing_timer / self.landing_duration).clamp(0.0, 1.0);
            pitch -= (progress * std::f32::consts::PI).sin() * self.landing_strength * 0.010;
        }

        [yaw, pitch]
    }

    pub fn camera_fov_offset(&self) -> f32 {
        let recoil = if self.recoil_duration > 0.0 {
            (self.recoil_timer / self.recoil_duration).clamp(0.0, 1.0) * 0.012
        } else {
            0.0
        };
        self.motion_fov + recoil
    }

    fn add_shake(&mut self, strength: f32, duration: f32) {
        if strength >= self.shake_strength || self.shake_timer <= 0.0 {
            self.shake_strength = strength.max(0.0);
            self.shake_duration = duration.max(0.0);
            self.shake_timer = self.shake_duration;
        }
    }

    fn push_event(&mut self, kind: FeedbackEventKind, value: u32) {
        for index in (1..self.events.len()).rev() {
            self.events[index] = self.events[index - 1];
        }

        self.events[0] = FeedbackEvent {
            kind,
            value: value.min(9999),
            timer: FEEDBACK_EVENT_DURATION,
            duration: FEEDBACK_EVENT_DURATION,
        };
    }
}

fn decay(value: f32, dt: f32) -> f32 {
    (value - dt).max(0.0)
}

fn rounded_amount(value: f32) -> u32 {
    value.max(0.0).round().min(9999.0) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timers_decay_to_zero() {
        let mut feedback = FeedbackState::new();
        feedback.on_enemy_hit_amount(0.0);
        feedback.tick(1.0);

        assert_eq!(feedback.hit_marker_timer, 0.0);
        assert_eq!(feedback.camera_offset(0.0), Vec3::ZERO);
    }

    #[test]
    fn kill_feedback_sets_hit_and_kill_markers() {
        let mut feedback = FeedbackState::new();
        feedback.on_enemy_kill_amount(0.0);

        assert!(feedback.hit_marker_timer > 0.0);
        assert!(feedback.kill_marker_timer > 0.0);
        assert!(feedback.camera_offset(0.0).length() > 0.0);
    }

    #[test]
    fn blocked_and_missed_shots_emit_distinct_feedback() {
        let mut feedback = FeedbackState::new();

        feedback.on_shot_missed();
        feedback.on_shot_blocked();

        assert!(feedback.miss_flash_timer > 0.0);
        assert!(feedback.blocked_flash_timer > 0.0);
        assert_eq!(feedback.events[0].kind, FeedbackEventKind::ShotBlocked);
        assert_eq!(feedback.events[1].kind, FeedbackEventKind::ShotMissed);
    }

    #[test]
    fn weaker_shake_does_not_replace_stronger_active_shake() {
        let mut feedback = FeedbackState::new();
        feedback.on_death();
        let strong_offset = feedback.camera_offset(0.0).length();
        feedback.on_fire();
        let after_fire_offset = feedback.camera_offset(0.0).length();

        assert_eq!(strong_offset, after_fire_offset);
    }

    #[test]
    fn events_keep_newest_first_and_expire() {
        let mut feedback = FeedbackState::new();
        feedback.on_enemy_hit_amount(12.4);
        feedback.on_player_damage_amount(3.0);

        assert_eq!(feedback.events[0].kind, FeedbackEventKind::PlayerDamage);
        assert_eq!(feedback.events[0].value, 3);
        assert_eq!(feedback.events[1].kind, FeedbackEventKind::EnemyHit);
        assert_eq!(feedback.events[1].value, 12);

        feedback.tick(10.0);

        assert!(feedback.events.iter().all(|event| !event.is_active()));
    }

    #[test]
    fn relic_change_uses_loot_feedback_channel() {
        let mut feedback = FeedbackState::new();

        feedback.on_relic_changed();

        assert!(feedback.pickup_flash_timer > 0.0);
        assert!(feedback.loot_flash_timer > 0.0);
        assert_eq!(feedback.events[0].kind, FeedbackEventKind::Relic);
    }

    #[test]
    fn named_relic_notice_carries_identity_rarity_and_outcome() {
        let mut feedback = FeedbackState::new();

        feedback.on_relic_acquired("Debt of the Last Keeper", "Rare", "Stored");

        let notice = feedback.named_notice.as_ref().unwrap();
        assert_eq!(notice.title, "Debt of the Last Keeper");
        assert_eq!(notice.subtitle, "Rare RELIC / Stored");
        assert_eq!(notice.remaining_ratio(), 1.0);

        feedback.tick(NAMED_NOTICE_DURATION + 0.1);
        assert!(feedback.named_notice.is_none());
    }

    #[test]
    fn grounded_motion_drives_bob_and_sprint_fov() {
        let mut feedback = FeedbackState::new();
        feedback.update_motion(
            0.1,
            MotionSample {
                horizontal_speed: 7.0,
                walk_speed: 5.0,
                grounded: true,
                sprinting: true,
                ..MotionSample::default()
            },
        );

        assert!(feedback.camera_offset(0.0).length() > 0.0);
        assert!(feedback.camera_fov_offset() > 0.0);
    }

    #[test]
    fn recoil_is_visual_only_and_decays() {
        let mut feedback = FeedbackState::new();
        feedback.on_fire();
        let initial = feedback.camera_rotation_offset();
        assert!(initial[1] > 0.0);

        feedback.tick(1.0);
        assert_eq!(feedback.camera_rotation_offset(), [0.0, 0.0]);
    }

    #[test]
    fn level_arrival_timer_expires() {
        let mut feedback = FeedbackState::new();
        feedback.on_level_enter();
        assert_eq!(feedback.level_arrival_ratio(), 1.0);

        feedback.tick(LEVEL_ARRIVAL_DURATION + 0.1);
        assert_eq!(feedback.level_arrival_ratio(), 0.0);
    }
}

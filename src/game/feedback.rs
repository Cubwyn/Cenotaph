use glam::Vec3;

pub const FEEDBACK_EVENT_CAPACITY: usize = 5;
const FEEDBACK_EVENT_DURATION: f32 = 4.5;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FeedbackEventKind {
    #[default]
    None,
    PlayerDamage,
    EnemyHit,
    EnemyKill,
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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FeedbackState {
    pub time: f32,
    pub shot_flash_timer: f32,
    pub hit_marker_timer: f32,
    pub kill_marker_timer: f32,
    pub pickup_flash_timer: f32,
    pub damage_flash_timer: f32,
    pub debug_flash_timer: f32,
    pub spawn_flash_timer: f32,
    pub reload_flash_timer: f32,
    pub loot_flash_timer: f32,
    pub heal_flash_timer: f32,
    pub events: [FeedbackEvent; FEEDBACK_EVENT_CAPACITY],
    shake_timer: f32,
    shake_duration: f32,
    shake_strength: f32,
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
        self.pickup_flash_timer = decay(self.pickup_flash_timer, dt);
        self.damage_flash_timer = decay(self.damage_flash_timer, dt);
        self.debug_flash_timer = decay(self.debug_flash_timer, dt);
        self.spawn_flash_timer = decay(self.spawn_flash_timer, dt);
        self.reload_flash_timer = decay(self.reload_flash_timer, dt);
        self.loot_flash_timer = decay(self.loot_flash_timer, dt);
        self.heal_flash_timer = decay(self.heal_flash_timer, dt);
        self.shake_timer = decay(self.shake_timer, dt);
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
    }

    pub fn on_fire(&mut self) {
        self.shot_flash_timer = self.shot_flash_timer.max(0.08);
        self.add_shake(0.025, 0.08);
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

    pub fn camera_offset(&self, yaw: f32) -> Vec3 {
        if self.shake_timer <= 0.0 || self.shake_duration <= 0.0 {
            return Vec3::ZERO;
        }

        let remaining = (self.shake_timer / self.shake_duration).clamp(0.0, 1.0);
        let amplitude = self.shake_strength * remaining * remaining;
        let right = Vec3::new(-yaw.sin(), 0.0, yaw.cos());
        let phase = self.time * 71.0;
        let x = phase.sin() * amplitude;
        let y = (phase * 1.37).cos() * amplitude * 0.55;
        right * x + Vec3::Y * y
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
}

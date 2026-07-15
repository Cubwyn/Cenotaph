use crate::data::config::gameplay::{MovementConfig, PlayerConfig};

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub health: HealthState,
    pub stamina: StaminaState,
    pub is_sprinting: bool,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_cooldown_timer: f32,
    pub dash_direction: [f32; 3],
    pub speed_multiplier_smoothed: f32,
    pub hit_flash_timer: f32,
    pub hurtbox_cooldown: f32,
    pub is_dead: bool,
    pub respawn_timer: f32,
    dash_was_pressed: bool,
}

impl PlayerState {
    pub fn new(config: &PlayerConfig) -> Self {
        Self {
            health: HealthState::new(config.max_health),
            stamina: StaminaState::new(config.max_stamina),
            is_sprinting: false,
            is_dashing: false,
            dash_timer: 0.0,
            dash_cooldown_timer: 0.0,
            dash_direction: [0.0, 0.0, 0.0],
            speed_multiplier_smoothed: 1.0,
            hit_flash_timer: 0.0,
            hurtbox_cooldown: 0.0,
            is_dead: false,
            respawn_timer: 0.0,
            dash_was_pressed: false,
        }
    }

    pub fn reset_for_level_transition(&mut self, config: &PlayerConfig) {
        self.stamina.restore_full(config.max_stamina);
        self.is_sprinting = false;
        self.reset_dash();
        self.speed_multiplier_smoothed = 1.0;
        self.hurtbox_cooldown = 0.0;
        self.hit_flash_timer = 0.0;
    }

    pub fn reconfigure(&mut self, config: &PlayerConfig) {
        self.health.set_max_preserving_ratio(config.max_health);
        self.stamina.set_max_preserving_ratio(config.max_stamina);
    }

    pub fn restore_after_respawn(&mut self, config: &PlayerConfig) {
        self.health.restore_full(config.max_health);
        self.stamina.restore_full(config.max_stamina);
        self.is_sprinting = false;
        self.reset_dash();
        self.speed_multiplier_smoothed = 1.0;
        self.hurtbox_cooldown = 0.0;
        self.hit_flash_timer = 0.0;
        self.is_dead = false;
        self.respawn_timer = 0.0;
    }

    pub fn begin_death(&mut self, respawn_delay: f32) {
        self.health.current = 0.0;
        self.is_dead = true;
        self.is_sprinting = false;
        self.reset_dash();
        self.respawn_timer = respawn_delay.max(0.0);
    }

    pub fn tick_timers(&mut self, dt: f32) {
        self.health.smooth_trail(dt, 2.8);
        self.hit_flash_timer = (self.hit_flash_timer - dt).max(0.0);
        self.hurtbox_cooldown = (self.hurtbox_cooldown - dt).max(0.0);
        self.dash_cooldown_timer = (self.dash_cooldown_timer - dt).max(0.0);
        self.dash_timer = (self.dash_timer - dt).max(0.0);
        self.is_dashing = self.dash_timer > 0.0;
        if !self.is_dashing {
            self.dash_direction = [0.0, 0.0, 0.0];
        }
    }

    pub fn flash_hit(&mut self, duration: f32) {
        self.hit_flash_timer = duration.max(0.0);
    }

    pub fn update_dash_input(
        &mut self,
        dash_held: bool,
        has_movement: bool,
        movement: &MovementConfig,
        player: &PlayerConfig,
        direction: [f32; 3],
    ) -> bool {
        let dash_just_pressed = dash_held && !self.dash_was_pressed;
        self.dash_was_pressed = dash_held;

        if !dash_just_pressed
            || !has_movement
            || self.is_dashing
            || self.dash_cooldown_timer > 0.0
            || self.stamina.current < movement.dash_stamina_cost
        {
            return false;
        }

        self.stamina.drain(movement.dash_stamina_cost);
        self.stamina.delay_regen(player.stamina_regen_delay);
        self.dash_timer = movement.dash_duration.max(0.0);
        self.dash_cooldown_timer = movement.dash_cooldown.max(0.0);
        self.is_dashing = self.dash_timer > 0.0;
        self.dash_direction = direction;
        self.is_sprinting = false;
        self.is_dashing
    }

    fn reset_dash(&mut self) {
        self.is_dashing = false;
        self.dash_timer = 0.0;
        self.dash_cooldown_timer = 0.0;
        self.dash_direction = [0.0, 0.0, 0.0];
        self.dash_was_pressed = false;
    }
}

#[derive(Debug, Clone)]
pub struct HealthState {
    pub current: f32,
    pub max: f32,
    pub trail: f32,
}

impl HealthState {
    pub fn new(max: f32) -> Self {
        let max = max.max(1.0);
        Self {
            current: max,
            max,
            trail: max,
        }
    }

    pub fn ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.current / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn trail_ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.trail / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount.max(0.0)).clamp(0.0, self.max);
        self.trail = self.trail.max(self.current);
    }

    pub fn smooth_trail(&mut self, dt: f32, smoothing_rate: f32) {
        if self.current >= self.trail {
            self.trail = self.current;
            return;
        }
        let lerp = (smoothing_rate.max(0.0) * dt.max(0.0)).min(1.0);
        self.trail += (self.current - self.trail) * lerp;
        self.trail = self.trail.clamp(self.current, self.max);
    }

    pub fn restore_full(&mut self, max: f32) {
        self.max = max.max(1.0);
        self.current = self.max;
        self.trail = self.max;
    }

    fn set_max_preserving_ratio(&mut self, max: f32) {
        let ratio = self.ratio();
        let trail_ratio = self.trail_ratio();
        self.max = max.max(1.0);
        self.current = self.max * ratio;
        self.trail = (self.max * trail_ratio).max(self.current);
    }

    pub fn is_depleted(&self) -> bool {
        self.current <= 0.0
    }
}

#[derive(Debug, Clone)]
pub struct StaminaState {
    pub current: f32,
    pub max: f32,
    pub regen_delay_timer: f32,
    pub smoothed: f32,
}

impl StaminaState {
    pub fn new(max: f32) -> Self {
        let max = max.max(0.0);
        Self {
            current: max,
            max,
            regen_delay_timer: 0.0,
            smoothed: max,
        }
    }

    pub fn display_ratio(&self) -> f32 {
        if self.max <= 0.0 {
            0.0
        } else {
            (self.smoothed / self.max).clamp(0.0, 1.0)
        }
    }

    pub fn drain(&mut self, amount: f32) {
        self.current = (self.current - amount.max(0.0)).clamp(0.0, self.max);
    }

    pub fn tick_regen(&mut self, dt: f32, regen_rate: f32) {
        self.regen_delay_timer = (self.regen_delay_timer - dt).max(0.0);
        if self.regen_delay_timer <= 0.0 {
            self.current = (self.current + regen_rate.max(0.0) * dt).min(self.max);
        }
    }

    pub fn delay_regen(&mut self, delay: f32) {
        self.regen_delay_timer = delay.max(0.0);
    }

    pub fn smooth_display(&mut self, dt: f32, smoothing_rate: f32) {
        let lerp = (smoothing_rate.max(0.0) * dt).min(1.0);
        self.smoothed += (self.current - self.smoothed) * lerp;
        self.smoothed = self.smoothed.clamp(0.0, self.max);
    }

    pub fn restore_full(&mut self, max: f32) {
        self.max = max.max(0.0);
        self.current = self.max;
        self.smoothed = self.max;
        self.regen_delay_timer = 0.0;
    }

    fn set_max_preserving_ratio(&mut self, max: f32) {
        let (current_ratio, display_ratio) = if self.max <= 0.0 {
            (1.0, 1.0)
        } else {
            (
                (self.current / self.max).clamp(0.0, 1.0),
                (self.smoothed / self.max).clamp(0.0, 1.0),
            )
        };
        self.max = max.max(0.0);
        self.current = self.max * current_ratio;
        self.smoothed = self.max * display_ratio;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_damage_clamps_at_zero() {
        let mut health = HealthState::new(100.0);
        health.damage(250.0);
        assert_eq!(health.current, 0.0);
        assert!(health.is_depleted());
        assert_eq!(health.ratio(), 0.0);
    }

    #[test]
    fn health_trail_follows_damage_without_hiding_the_hit() {
        let mut health = HealthState::new(100.0);
        health.damage(40.0);

        assert_eq!(health.current, 60.0);
        assert_eq!(health.trail, 100.0);
        health.smooth_trail(0.1, 2.0);
        assert!(health.trail > health.current);
        assert!(health.trail < 100.0);

        health.smooth_trail(1.0, 2.0);
        assert_eq!(health.trail, health.current);
    }

    #[test]
    fn stamina_waits_for_delay_before_regen() {
        let mut stamina = StaminaState::new(100.0);
        stamina.drain(50.0);
        stamina.delay_regen(1.0);
        stamina.tick_regen(0.5, 10.0);
        assert_eq!(stamina.current, 50.0);
        stamina.tick_regen(0.5, 10.0);
        assert_eq!(stamina.current, 55.0);
    }

    #[test]
    fn dash_consumes_stamina_and_starts_cooldown() {
        let player_config = PlayerConfig::default();
        let movement_config = MovementConfig::default();
        let mut player = PlayerState::new(&player_config);

        assert!(player.update_dash_input(
            true,
            true,
            &movement_config,
            &player_config,
            [1.0, 0.0, 0.0],
        ));
        assert!(player.is_dashing);
        assert_eq!(
            player.stamina.current,
            player_config.max_stamina - movement_config.dash_stamina_cost
        );
        assert_eq!(player.dash_cooldown_timer, movement_config.dash_cooldown);
        assert_eq!(player.dash_direction, [1.0, 0.0, 0.0]);
    }

    #[test]
    fn dash_is_edge_triggered_and_expires() {
        let player_config = PlayerConfig::default();
        let movement_config = MovementConfig::default();
        let mut player = PlayerState::new(&player_config);

        assert!(player.update_dash_input(
            true,
            true,
            &movement_config,
            &player_config,
            [0.0, 0.0, 1.0],
        ));
        assert!(!player.update_dash_input(
            true,
            true,
            &movement_config,
            &player_config,
            [0.0, 0.0, 1.0],
        ));

        player.tick_timers(movement_config.dash_duration);
        assert!(!player.is_dashing);
        assert_eq!(player.dash_direction, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn reconfigure_preserves_health_and_stamina_ratios() {
        let mut player = PlayerState::new(&PlayerConfig::default());
        player.health.damage(25.0);
        player.stamina.drain(50.0);
        player.stamina.smoothed = 60.0;
        let config = PlayerConfig {
            max_health: 200.0,
            max_stamina: 50.0,
            ..PlayerConfig::default()
        };

        player.reconfigure(&config);

        assert_eq!(player.health.current, 150.0);
        assert_eq!(player.health.max, 200.0);
        assert_eq!(player.stamina.current, 25.0);
        assert!((player.stamina.smoothed - 30.0).abs() < 0.0001);
        assert_eq!(player.stamina.max, 50.0);
    }

    #[test]
    fn respawn_restores_foundation_state() {
        let config = PlayerConfig::default();
        let mut player = PlayerState::new(&config);
        player.health.damage(100.0);
        player.stamina.drain(80.0);
        player.begin_death(3.0);
        player.restore_after_respawn(&config);

        assert!(!player.is_dead);
        assert_eq!(player.health.current, config.max_health);
        assert_eq!(player.stamina.current, config.max_stamina);
    }
}

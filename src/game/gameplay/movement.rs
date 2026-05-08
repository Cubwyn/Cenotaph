// src/gameplay/movement.rs
// Weighty movement system with stamina management
#![allow(dead_code)]

use crate::game::gameplay::stamina::StaminaSystem;

#[derive(Debug, Clone)]
pub struct MovementConfig {
    pub walk_speed: f32,           // Base walking speed
    pub sprint_speed: f32,         // Sprinting speed
    pub dash_speed_multiplier: f32, // How much faster dash is than sprint
    pub dash_stamina_cost: f32,    // Stamina cost for dashing
    pub dash_cooldown: f32,        // Time between dashes
    pub dash_duration: f32,        // How long dash lasts
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            walk_speed: 3.0,
            sprint_speed: 8.0,
            dash_speed_multiplier: 2.0,
            dash_stamina_cost: 25.0,
            dash_cooldown: 2.0,
            dash_duration: 0.5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MovementManager {
    pub stamina: StaminaSystem,
    pub config: MovementConfig,
    pub is_sprinting: bool,
    pub is_dashing: bool,
    pub dash_timer: f32,
    pub dash_timer_current: f32,
}

impl MovementManager {
    pub fn new() -> Self {
        Self {
            stamina: StaminaSystem::new(100.0),
            config: MovementConfig::default(),
            is_sprinting: false,
            is_dashing: false,
            dash_timer: 0.0,
            dash_timer_current: 0.0,
        }
    }

    pub fn new_with_config(stamina: StaminaSystem, config: MovementConfig) -> Self {
        Self {
            stamina,
            config,
            is_sprinting: false,
            is_dashing: false,
            dash_timer: 0.0,
            dash_timer_current: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.stamina.update(dt);

        if self.is_dashing {
            self.dash_timer_current -= dt;
            if self.dash_timer_current <= 0.0 {
                self.is_dashing = false;
            }
        } else if self.dash_timer > 0.0 {
            self.dash_timer -= dt;
        }
    }

    pub fn try_sprint(&mut self) -> bool {
        if self.stamina.current > 0.0 {
            self.is_sprinting = true;
            true
        } else {
            self.is_sprinting = false;
            false
        }
    }

    pub fn stop_sprint(&mut self) {
        self.is_sprinting = false;
    }

    pub fn try_dash(&mut self) -> bool {
        if self.dash_timer <= 0.0 && self.stamina.consume(self.config.dash_stamina_cost) {
            self.is_dashing = true;
            self.dash_timer_current = self.config.dash_duration;
            self.dash_timer = self.config.dash_cooldown;
            true
        } else {
            false
        }
    }

    pub fn get_movement_speed(&self) -> f32 {
        if self.is_dashing {
            self.config.sprint_speed * self.config.dash_speed_multiplier
        } else if self.is_sprinting && self.stamina.current > 0.0 {
            self.config.sprint_speed
        } else {
            self.config.walk_speed
        }
    }

    pub fn get_stamina_percentage(&self) -> f32 {
        self.stamina.current / self.stamina.max
    }

    pub fn is_dash_ready(&self) -> bool {
        self.dash_timer <= 0.0
    }

    pub fn get_dash_cooldown(&self) -> f32 {
        self.dash_timer.max(0.0)
    }
}

#[derive(Debug, Clone)]
pub struct PlayerController {
    pub movement: MovementManager,
    pub input_forward: f32,
    pub input_right: f32,
    pub input_jump: bool,
    pub input_sprint: bool,
    pub input_dash: bool,
}

impl PlayerController {
    pub fn new() -> Self {
        Self {
            movement: MovementManager::new(),
            input_forward: 0.0,
            input_right: 0.0,
            input_jump: false,
            input_sprint: false,
            input_dash: false,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.movement.update(dt);

        // Handle sprint input
        if self.input_sprint {
            self.movement.try_sprint();
        } else {
            self.movement.stop_sprint();
        }

        // Handle dash input
        if self.input_dash {
            self.movement.try_dash();
        }
    }

    pub fn get_movement_vector(&self) -> [f32; 3] {
        let speed = self.movement.get_movement_speed();
        [self.input_forward * speed, 0.0, self.input_right * speed]
    }

    pub fn should_jump(&self) -> bool {
        self.input_jump
    }
}
// src/gameplay/stamina.rs
// Weighty combat stamina management
#![allow(dead_code)]

#[derive(Debug, Clone)]
pub struct StaminaConfig {
    pub regen_rate: f32,           // How fast stamina regenerates per second
    pub recovery_delay: f32,       // How long to wait before regen starts after depletion
    pub depletion_multiplier: f32, // Multiplier for stamina consumption
}

impl Default for StaminaConfig {
    fn default() -> Self {
        Self {
            regen_rate: 10.0,
            recovery_delay: 1.0,
            depletion_multiplier: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StaminaSystem {
    pub current: f32,
    pub max: f32,
    pub config: StaminaConfig,
    pub recovery_timer: f32,
}

impl StaminaSystem {
    pub fn new(max_stamina: f32) -> Self {
        Self {
            current: max_stamina,
            max: max_stamina,
            config: StaminaConfig::default(),
            recovery_timer: 0.0,
        }
    }

    pub fn new_with_config(max_stamina: f32, config: StaminaConfig) -> Self {
        Self {
            current: max_stamina,
            max: max_stamina,
            config,
            recovery_timer: 0.0,
        }
    }

    pub fn consume(&mut self, amount: f32) -> bool {
        if self.current >= amount {
            self.current -= amount;
            self.recovery_timer = self.config.recovery_delay;
            true
        } else {
            false
        }
    }
    
    pub fn update(&mut self, dt: f32) {
        if self.recovery_timer > 0.0 {
            self.recovery_timer -= dt;
        } else {
            self.current = (self.current + self.config.regen_rate * dt).min(self.max);
        }
    }
    
    pub fn regenerate(&mut self, dt: f32) {
        self.current = (self.current + self.config.regen_rate * dt).min(self.max);
    }
}
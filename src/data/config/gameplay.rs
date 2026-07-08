// src/config/gameplay.rs
// Game configuration system - everything a designer needs to tweak
// without touching code. All values have clear descriptions and sane defaults.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct GameConfig {
    pub player: PlayerConfig,
    pub movement: MovementConfig,
    pub camera: CameraConfig,
    pub physics: PhysicsConfig,
    pub combat: CombatConfig,
    pub world: WorldConfig,
    pub lighting: LightingConfig,
    pub debug: DebugConfig,
    // Key bindings — manually loaded from config/bindings.toml via load_bindings()
    // Note: serde skip is required because winit::KeyCode does not implement Serialize/Deserialize
    #[serde(skip_serializing, skip_deserializing)]
    pub keys: std::collections::HashMap<String, Option<winit::keyboard::KeyCode>>,
}

impl GameConfig {
    /// Returns the KeyCode bound to `action`, or `None` if unbound / unknown.
    pub fn key(&self, action: &str) -> Option<winit::keyboard::KeyCode> {
        self.keys.get(action).copied().flatten()
    }

    /// Load configuration from config/bindings.toml and config/tuning.toml
    pub fn load() -> Self {
        // Load tuning.toml for all game constants
        let tuning = Self::load_tuning();

        // Load bindings.toml for key bindings
        let keys = Self::load_bindings();

        Self {
            player: tuning.player,
            movement: tuning.movement,
            camera: tuning.camera,
            physics: tuning.physics,
            combat: tuning.combat,
            world: tuning.world,
            lighting: tuning.lighting,
            debug: tuning.debug,
            keys,
        }
    }

    fn load_tuning() -> Self {
        let tuning_str = match std::fs::read_to_string("config/tuning.toml") {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Warning: Could not find config/tuning.toml. Using compiled defaults.");
                return Self::default();
            }
        };

        match toml::from_str(&tuning_str) {
            Ok(config) => config,
            Err(e) => {
                eprintln!(
                    "Warning: Failed to parse tuning.toml ({}). Using compiled defaults.",
                    e
                );
                Self::default()
            }
        }
    }

    fn load_bindings() -> std::collections::HashMap<String, Option<winit::keyboard::KeyCode>> {
        let default_keys = vec![
            ("forward", "W"),
            ("backward", "S"),
            ("left", "A"),
            ("right", "D"),
            ("jump", "SPACE"),
            ("dash", "Q"),
            ("sprint", "SHIFT"),
            ("attack", "MOUSE_LEFT"),
            ("interact", "E"),
            ("inventory", "I"),
            ("pause", "ESCAPE"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), parse_key(v)))
        .collect();

        let bindings_str = match std::fs::read_to_string("config/bindings.toml") {
            Ok(s) => s,
            Err(_) => {
                eprintln!("Warning: Could not find config/bindings.toml. Using default keys.");
                return default_keys;
            }
        };

        let raw: toml::Value = match toml::from_str(&bindings_str) {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Warning: Failed to parse bindings.toml. Using default keys.");
                return default_keys;
            }
        };

        if let Some(keybindings) = raw.get("keybindings") {
            if let Some(keybindings_table) = keybindings.as_table() {
                let mut keys = default_keys;
                for (action, key_val) in keybindings_table.iter() {
                    if let Some(key_str) = key_val.as_str() {
                        keys.insert(action.clone(), parse_key(key_str));
                    }
                }
                return keys;
            }
        }

        default_keys
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Camera sensitivity for mouse look (higher = more sensitive)
    /// Higher values = faster camera turning
    pub sensitivity: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self { sensitivity: 0.002 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PhysicsConfig {
    pub gravity: f32,
    pub jump_velocity: f32,
    pub player_speed: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            gravity: -9.8,
            jump_velocity: 5.0,
            player_speed: 5.0,
        }
    }
}

pub(crate) fn parse_key(key_str: &str) -> Option<winit::keyboard::KeyCode> {
    match key_str.to_uppercase().as_str() {
        // Letters
        "A" => Some(winit::keyboard::KeyCode::KeyA),
        "B" => Some(winit::keyboard::KeyCode::KeyB),
        "C" => Some(winit::keyboard::KeyCode::KeyC),
        "D" => Some(winit::keyboard::KeyCode::KeyD),
        "E" => Some(winit::keyboard::KeyCode::KeyE),
        "F" => Some(winit::keyboard::KeyCode::KeyF),
        "G" => Some(winit::keyboard::KeyCode::KeyG),
        "H" => Some(winit::keyboard::KeyCode::KeyH),
        "I" => Some(winit::keyboard::KeyCode::KeyI),
        "J" => Some(winit::keyboard::KeyCode::KeyJ),
        "K" => Some(winit::keyboard::KeyCode::KeyK),
        "L" => Some(winit::keyboard::KeyCode::KeyL),
        "M" => Some(winit::keyboard::KeyCode::KeyM),
        "N" => Some(winit::keyboard::KeyCode::KeyN),
        "O" => Some(winit::keyboard::KeyCode::KeyO),
        "P" => Some(winit::keyboard::KeyCode::KeyP),
        "Q" => Some(winit::keyboard::KeyCode::KeyQ),
        "R" => Some(winit::keyboard::KeyCode::KeyR),
        "S" => Some(winit::keyboard::KeyCode::KeyS),
        "T" => Some(winit::keyboard::KeyCode::KeyT),
        "U" => Some(winit::keyboard::KeyCode::KeyU),
        "V" => Some(winit::keyboard::KeyCode::KeyV),
        "W" => Some(winit::keyboard::KeyCode::KeyW),
        "X" => Some(winit::keyboard::KeyCode::KeyX),
        "Y" => Some(winit::keyboard::KeyCode::KeyY),
        "Z" => Some(winit::keyboard::KeyCode::KeyZ),

        // Digits
        "0" => Some(winit::keyboard::KeyCode::Digit0),
        "1" => Some(winit::keyboard::KeyCode::Digit1),
        "2" => Some(winit::keyboard::KeyCode::Digit2),
        "3" => Some(winit::keyboard::KeyCode::Digit3),
        "4" => Some(winit::keyboard::KeyCode::Digit4),
        "5" => Some(winit::keyboard::KeyCode::Digit5),
        "6" => Some(winit::keyboard::KeyCode::Digit6),
        "7" => Some(winit::keyboard::KeyCode::Digit7),
        "8" => Some(winit::keyboard::KeyCode::Digit8),
        "9" => Some(winit::keyboard::KeyCode::Digit9),

        // Special
        "SPACE" => Some(winit::keyboard::KeyCode::Space),
        "TAB" => Some(winit::keyboard::KeyCode::Tab),
        "SHIFT" => Some(winit::keyboard::KeyCode::ShiftLeft),
        "CTRL" => Some(winit::keyboard::KeyCode::ControlLeft),
        "ALT" => Some(winit::keyboard::KeyCode::AltLeft),
        "ESCAPE" => Some(winit::keyboard::KeyCode::Escape),
        "ENTER" => Some(winit::keyboard::KeyCode::Enter),
        "BACKSPACE" => Some(winit::keyboard::KeyCode::Backspace),
        "DELETE" => Some(winit::keyboard::KeyCode::Delete),

        // Arrow keys
        "UP" => Some(winit::keyboard::KeyCode::ArrowUp),
        "DOWN" => Some(winit::keyboard::KeyCode::ArrowDown),
        "LEFT" => Some(winit::keyboard::KeyCode::ArrowLeft),
        "RIGHT" => Some(winit::keyboard::KeyCode::ArrowRight),

        // Function keys
        "F1" => Some(winit::keyboard::KeyCode::F1),
        "F2" => Some(winit::keyboard::KeyCode::F2),
        "F3" => Some(winit::keyboard::KeyCode::F3),
        "F4" => Some(winit::keyboard::KeyCode::F4),
        "F5" => Some(winit::keyboard::KeyCode::F5),
        "F6" => Some(winit::keyboard::KeyCode::F6),
        "F7" => Some(winit::keyboard::KeyCode::F7),
        "F8" => Some(winit::keyboard::KeyCode::F8),
        "F9" => Some(winit::keyboard::KeyCode::F9),
        "F10" => Some(winit::keyboard::KeyCode::F10),
        "F11" => Some(winit::keyboard::KeyCode::F11),
        "F12" => Some(winit::keyboard::KeyCode::F12),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CombatConfig {
    /// Damage dealt by one primary-fire hit.
    pub base_damage: f32,
    /// Reserved multiplier for future precision or weak-point hits.
    pub crit_multiplier: f32,
    /// Cooldown after a successful primary-fire hit.
    pub attack_cooldown: f32,
    /// Cooldown after firing without hitting an enemy.
    pub miss_cooldown: f32,
    /// Radius used by the current prototype ray/sphere hit test.
    pub enemy_hit_radius: f32,
    /// Damage per second applied by hurtbox props.
    pub hurtbox_damage_per_second: f32,
    /// Distance at which hurtbox props can damage the player.
    pub hurtbox_radius: f32,
    /// Minimum interval between hurtbox damage ticks.
    pub hurtbox_tick_interval: f32,
    /// Delay before the player respawns after death.
    pub respawn_delay: f32,
}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            base_damage: 25.0,
            crit_multiplier: 2.0,
            attack_cooldown: 0.25,
            miss_cooldown: 0.15,
            enemy_hit_radius: 2.0,
            hurtbox_damage_per_second: 15.0,
            hurtbox_radius: 3.0,
            hurtbox_tick_interval: 0.5,
            respawn_delay: 3.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WorldConfig {
    pub draw_distance: f32,
    pub fog_density: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            draw_distance: 500.0,
            fog_density: 0.008,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LightingConfig {
    pub ambient_color: [f32; 3],
    pub sun_color: [f32; 3],
    pub sun_intensity: f32,
    pub sun_position_offset: f32,
}

impl Default for LightingConfig {
    fn default() -> Self {
        Self {
            ambient_color: [0.08, 0.07, 0.06],
            sun_color: [1.0, 0.85, 0.6],
            sun_intensity: 1.8,
            sun_position_offset: 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DebugConfig {
    /// Enables periodic player position/stamina logging to the console.
    pub position_log_enabled: bool,
    /// Seconds between player position/stamina log lines.
    pub position_log_interval: f32,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            position_log_enabled: true,
            position_log_interval: 5.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PlayerConfig {
    /// Maximum health points the player can have
    /// Higher values = more survivability, longer fights
    pub max_health: f32,

    /// Maximum stamina points for sprinting and dashing
    /// Higher values = more mobility options
    pub max_stamina: f32,

    /// How fast stamina regenerates when not in use (per second)
    /// Higher values = faster recovery between actions
    pub stamina_regen_rate: f32,

    /// Delay before stamina starts regenerating after depletion (seconds)
    /// Higher values = more tactical stamina management
    pub stamina_regen_delay: f32,

    /// Base movement speed when walking (units per second)
    /// Higher values = faster overall movement
    pub walk_speed: f32,

    /// Base movement speed when sprinting (units per second)
    /// Higher values = faster sprinting
    pub sprint_speed: f32,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            max_health: 100.0,
            max_stamina: 100.0,
            stamina_regen_rate: 10.0,
            stamina_regen_delay: 1.0,
            walk_speed: 3.0,
            sprint_speed: 10.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MovementConfig {
    /// How much faster dash is compared to sprint
    /// Higher values = more dramatic dash speed
    pub dash_speed_multiplier: f32,

    /// How much stamina is consumed per second of sprinting
    /// Higher values = sprint drains faster
    pub sprint_stamina_drain_rate: f32,

    /// How much stamina is consumed per dash
    /// Higher values = more commitment to dash
    pub dash_stamina_cost: f32,

    /// Time between dashes (seconds)
    /// Higher values = less spammy dashing
    pub dash_cooldown: f32,

    /// Duration of dash effect (seconds)
    /// Higher values = longer dash duration
    pub dash_duration: f32,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            dash_speed_multiplier: 2.0,
            sprint_stamina_drain_rate: 20.0,
            dash_stamina_cost: 25.0,
            dash_cooldown: 2.0,
            dash_duration: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::KeyCode;

    #[test]
    fn parse_key_accepts_common_foundation_bindings() {
        assert_eq!(parse_key("W"), Some(KeyCode::KeyW));
        assert_eq!(parse_key("space"), Some(KeyCode::Space));
        assert_eq!(parse_key("SHIFT"), Some(KeyCode::ShiftLeft));
        assert_eq!(parse_key("not-a-key"), None);
    }

    #[test]
    fn tuning_can_omit_newer_sections() {
        let parsed: GameConfig = toml::from_str(
            r#"
            [player]
            max_health = 80.0
            "#,
        )
        .expect("partial tuning should use defaults");

        assert_eq!(parsed.player.max_health, 80.0);
        assert_eq!(
            parsed.combat.base_damage,
            CombatConfig::default().base_damage
        );
        assert_eq!(
            parsed.combat.crit_multiplier,
            CombatConfig::default().crit_multiplier
        );
        assert_eq!(
            parsed.world.draw_distance,
            WorldConfig::default().draw_distance
        );
        assert_eq!(
            parsed.debug.position_log_interval,
            DebugConfig::default().position_log_interval
        );
    }

    #[test]
    fn tuning_parses_debug_logging_controls() {
        let parsed: GameConfig = toml::from_str(
            r#"
            [debug]
            position_log_enabled = false
            position_log_interval = 12.5
            "#,
        )
        .expect("debug tuning should parse");

        assert!(!parsed.debug.position_log_enabled);
        assert_eq!(parsed.debug.position_log_interval, 12.5);
    }
}

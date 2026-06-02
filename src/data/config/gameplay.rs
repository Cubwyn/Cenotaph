// src/config/gameplay.rs
// Game configuration system - everything a designer needs to tweak
// without touching code. All values have clear descriptions and sane defaults.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub player: PlayerConfig,
    pub movement: MovementConfig,
    pub camera: CameraConfig,
    pub physics: PhysicsConfig,
    // Key bindings — manually loaded from config/bindings.toml via load_bindings()
    // Note: serde skip is required because winit::KeyCode does not implement Serialize/Deserialize
    #[serde(skip_serializing, skip_deserializing)]
    pub keys: std::collections::HashMap<String, Option<winit::keyboard::KeyCode>>,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            player: PlayerConfig::default(),
            movement: MovementConfig::default(),
            camera: CameraConfig::default(),
            physics: PhysicsConfig::default(),
            keys: std::collections::HashMap::new(),
        }
    }
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
                eprintln!("Warning: Failed to parse tuning.toml ({}). Using compiled defaults.", e);
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
pub struct CameraConfig {
    /// Camera sensitivity for mouse look (higher = more sensitive)
    /// Higher values = faster camera turning
    pub sensitivity: f32,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn parse_key(key_str: &str) -> Option<winit::keyboard::KeyCode> {
    match key_str.to_uppercase().as_str() {
        // Letters
        "A"         => Some(winit::keyboard::KeyCode::KeyA),
        "B"         => Some(winit::keyboard::KeyCode::KeyB),
        "C"         => Some(winit::keyboard::KeyCode::KeyC),
        "D"         => Some(winit::keyboard::KeyCode::KeyD),
        "E"         => Some(winit::keyboard::KeyCode::KeyE),
        "F"         => Some(winit::keyboard::KeyCode::KeyF),
        "G"         => Some(winit::keyboard::KeyCode::KeyG),
        "H"         => Some(winit::keyboard::KeyCode::KeyH),
        "I"         => Some(winit::keyboard::KeyCode::KeyI),
        "J"         => Some(winit::keyboard::KeyCode::KeyJ),
        "K"         => Some(winit::keyboard::KeyCode::KeyK),
        "L"         => Some(winit::keyboard::KeyCode::KeyL),
        "M"         => Some(winit::keyboard::KeyCode::KeyM),
        "N"         => Some(winit::keyboard::KeyCode::KeyN),
        "O"         => Some(winit::keyboard::KeyCode::KeyO),
        "P"         => Some(winit::keyboard::KeyCode::KeyP),
        "Q"         => Some(winit::keyboard::KeyCode::KeyQ),
        "R"         => Some(winit::keyboard::KeyCode::KeyR),
        "S"         => Some(winit::keyboard::KeyCode::KeyS),
        "T"         => Some(winit::keyboard::KeyCode::KeyT),
        "U"         => Some(winit::keyboard::KeyCode::KeyU),
        "V"         => Some(winit::keyboard::KeyCode::KeyV),
        "W"         => Some(winit::keyboard::KeyCode::KeyW),
        "X"         => Some(winit::keyboard::KeyCode::KeyX),
        "Y"         => Some(winit::keyboard::KeyCode::KeyY),
        "Z"         => Some(winit::keyboard::KeyCode::KeyZ),

        // Digits
        "0"         => Some(winit::keyboard::KeyCode::Digit0),
        "1"         => Some(winit::keyboard::KeyCode::Digit1),
        "2"         => Some(winit::keyboard::KeyCode::Digit2),
        "3"         => Some(winit::keyboard::KeyCode::Digit3),
        "4"         => Some(winit::keyboard::KeyCode::Digit4),
        "5"         => Some(winit::keyboard::KeyCode::Digit5),
        "6"         => Some(winit::keyboard::KeyCode::Digit6),
        "7"         => Some(winit::keyboard::KeyCode::Digit7),
        "8"         => Some(winit::keyboard::KeyCode::Digit8),
        "9"         => Some(winit::keyboard::KeyCode::Digit9),

        // Special
        "SPACE"     => Some(winit::keyboard::KeyCode::Space),
        "TAB"       => Some(winit::keyboard::KeyCode::Tab),
        "SHIFT"     => Some(winit::keyboard::KeyCode::ShiftLeft),
        "CTRL"      => Some(winit::keyboard::KeyCode::ControlLeft),
        "ALT"       => Some(winit::keyboard::KeyCode::AltLeft),
        "ESCAPE"    => Some(winit::keyboard::KeyCode::Escape),
        "ENTER"     => Some(winit::keyboard::KeyCode::Enter),
        "BACKSPACE" => Some(winit::keyboard::KeyCode::Backspace),
        "DELETE"    => Some(winit::keyboard::KeyCode::Delete),

        // Arrow keys
        "UP"        => Some(winit::keyboard::KeyCode::ArrowUp),
        "DOWN"      => Some(winit::keyboard::KeyCode::ArrowDown),
        "LEFT"      => Some(winit::keyboard::KeyCode::ArrowLeft),
        "RIGHT"     => Some(winit::keyboard::KeyCode::ArrowRight),
        
        // Function keys
        "F1"        => Some(winit::keyboard::KeyCode::F1),
        "F2"        => Some(winit::keyboard::KeyCode::F2),
        "F3"        => Some(winit::keyboard::KeyCode::F3),
        "F4"        => Some(winit::keyboard::KeyCode::F4),
        "F5"        => Some(winit::keyboard::KeyCode::F5),
        "F6"        => Some(winit::keyboard::KeyCode::F6),
        "F7"        => Some(winit::keyboard::KeyCode::F7),
        "F8"        => Some(winit::keyboard::KeyCode::F8),
        "F9"        => Some(winit::keyboard::KeyCode::F9),
        "F10"       => Some(winit::keyboard::KeyCode::F10),
        "F11"       => Some(winit::keyboard::KeyCode::F11),
        "F12"       => Some(winit::keyboard::KeyCode::F12),
        _ => None,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

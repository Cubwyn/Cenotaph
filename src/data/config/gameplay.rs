//! Gameplay tuning and input bindings loaded from the `config/` directory.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use super::ui::UiConfig;
use super::visuals::VisualConfig;

type KeyBindings = HashMap<String, Option<KeyCode>>;
const TUNING_PATH: &str = "config/tuning.toml";
const BINDINGS_PATH: &str = "config/bindings.toml";

pub(crate) const REQUIRED_BINDING_ACTIONS: &[&str] = &[
    "forward",
    "backward",
    "left",
    "right",
    "jump",
    "sprint",
    "dash",
    "attack",
    "interact",
    "inventory",
    "pause",
    "debug_help",
    "debug_heal_player",
    "debug_damage_player",
    "debug_set_player_low_health",
    "debug_reload_level",
    "debug_respawn_loot",
    "debug_spawn_ashbound",
    "debug_spawn_burdened",
    "debug_spawn_censer",
    "debug_spawn_chainrunner",
    "debug_spawn_harpy",
    "debug_clear_enemies",
];

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
    pub ui: UiConfig,
    pub visuals: VisualConfig,
    // KeyCode is loaded manually because winit does not serialize it.
    #[serde(skip_serializing, skip_deserializing)]
    pub keys: KeyBindings,
}

impl GameConfig {
    /// Returns the KeyCode bound to `action`, or `None` if unbound / unknown.
    pub fn key(&self, action: &str) -> Option<KeyCode> {
        self.keys.get(action).copied().flatten()
    }

    pub fn try_load() -> Result<Self, String> {
        Self::try_load_from_paths(TUNING_PATH, BINDINGS_PATH)
    }

    fn try_load_from_paths(
        tuning_path: impl AsRef<std::path::Path>,
        bindings_path: impl AsRef<std::path::Path>,
    ) -> Result<Self, String> {
        let tuning_path = tuning_path.as_ref();
        let tuning_str = std::fs::read_to_string(tuning_path).map_err(|error| {
            format!(
                "failed to read gameplay tuning '{}': {}",
                tuning_path.display(),
                error
            )
        })?;
        let tuning: Self = toml::from_str(&tuning_str).map_err(|error| {
            format!(
                "failed to parse gameplay tuning '{}': {}",
                tuning_path.display(),
                error
            )
        })?;

        let keys = load_bindings_from_path(bindings_path)?;
        Ok(Self {
            player: tuning.player,
            movement: tuning.movement,
            camera: tuning.camera,
            physics: tuning.physics,
            combat: tuning.combat,
            world: tuning.world,
            lighting: tuning.lighting,
            debug: tuning.debug,
            ui: tuning.ui,
            visuals: tuning.visuals,
            keys,
        })
    }
}

fn load_bindings_from_path(path: impl AsRef<std::path::Path>) -> Result<KeyBindings, String> {
    let path = path.as_ref();
    let bindings_str = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read bindings '{}': {}", path.display(), error))?;
    let raw: toml::Value = toml::from_str(&bindings_str)
        .map_err(|error| format!("failed to parse bindings '{}': {}", path.display(), error))?;
    let keybindings = raw
        .get("keybindings")
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("bindings '{}' is missing [keybindings]", path.display()))?;

    let missing = REQUIRED_BINDING_ACTIONS
        .iter()
        .filter(|action| !keybindings.contains_key(**action))
        .copied()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        return Err(format!(
            "bindings '{}' is missing required action(s): {}",
            path.display(),
            missing.join(", ")
        ));
    }

    let mut keys = default_bindings();
    let mut seen_tokens = HashMap::new();
    for (action, value) in keybindings {
        let token = value.as_str().ok_or_else(|| {
            format!(
                "binding '{}' in '{}' must be a string",
                action,
                path.display()
            )
        })?;
        if !is_valid_binding_token(token) {
            return Err(format!(
                "binding '{}' in '{}' uses unknown key '{}'",
                action,
                path.display(),
                token
            ));
        }
        let normalized = token.to_ascii_uppercase();
        if !matches!(normalized.as_str(), "NONE" | "UNBOUND") {
            if let Some(previous_action) = seen_tokens.insert(normalized.clone(), action.clone()) {
                return Err(format!(
                    "bindings '{}' assigns '{}' to both '{}' and '{}'",
                    path.display(),
                    normalized,
                    previous_action,
                    action
                ));
            }
        }
        keys.insert(action.clone(), parse_key(token));
    }
    Ok(keys)
}

pub(crate) fn is_valid_binding_token(token: &str) -> bool {
    let token = token.to_ascii_uppercase();
    parse_key(&token).is_some()
        || matches!(
            token.as_str(),
            "MOUSE_LEFT" | "MOUSE_RIGHT" | "MOUSE_MIDDLE" | "NONE" | "UNBOUND"
        )
}

fn default_bindings() -> KeyBindings {
    [
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
        ("debug_help", "F1"),
        ("debug_heal_player", "F2"),
        ("debug_damage_player", "F3"),
        ("debug_set_player_low_health", "F4"),
        ("debug_reload_level", "F5"),
        ("debug_respawn_loot", "F6"),
        ("debug_spawn_ashbound", "F7"),
        ("debug_spawn_burdened", "F8"),
        ("debug_spawn_censer", "F9"),
        ("debug_spawn_chainrunner", "F10"),
        ("debug_spawn_harpy", "F11"),
        ("debug_clear_enemies", "F12"),
    ]
    .into_iter()
    .map(|(action, key)| (action.to_string(), parse_key(key)))
    .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CameraConfig {
    /// Mouse-look radians applied per input unit.
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

pub(crate) fn parse_key(key_str: &str) -> Option<KeyCode> {
    match key_str.to_uppercase().as_str() {
        "A" => Some(KeyCode::KeyA),
        "B" => Some(KeyCode::KeyB),
        "C" => Some(KeyCode::KeyC),
        "D" => Some(KeyCode::KeyD),
        "E" => Some(KeyCode::KeyE),
        "F" => Some(KeyCode::KeyF),
        "G" => Some(KeyCode::KeyG),
        "H" => Some(KeyCode::KeyH),
        "I" => Some(KeyCode::KeyI),
        "J" => Some(KeyCode::KeyJ),
        "K" => Some(KeyCode::KeyK),
        "L" => Some(KeyCode::KeyL),
        "M" => Some(KeyCode::KeyM),
        "N" => Some(KeyCode::KeyN),
        "O" => Some(KeyCode::KeyO),
        "P" => Some(KeyCode::KeyP),
        "Q" => Some(KeyCode::KeyQ),
        "R" => Some(KeyCode::KeyR),
        "S" => Some(KeyCode::KeyS),
        "T" => Some(KeyCode::KeyT),
        "U" => Some(KeyCode::KeyU),
        "V" => Some(KeyCode::KeyV),
        "W" => Some(KeyCode::KeyW),
        "X" => Some(KeyCode::KeyX),
        "Y" => Some(KeyCode::KeyY),
        "Z" => Some(KeyCode::KeyZ),
        "0" => Some(KeyCode::Digit0),
        "1" => Some(KeyCode::Digit1),
        "2" => Some(KeyCode::Digit2),
        "3" => Some(KeyCode::Digit3),
        "4" => Some(KeyCode::Digit4),
        "5" => Some(KeyCode::Digit5),
        "6" => Some(KeyCode::Digit6),
        "7" => Some(KeyCode::Digit7),
        "8" => Some(KeyCode::Digit8),
        "9" => Some(KeyCode::Digit9),
        "SPACE" => Some(KeyCode::Space),
        "TAB" => Some(KeyCode::Tab),
        "SHIFT" => Some(KeyCode::ShiftLeft),
        "CTRL" => Some(KeyCode::ControlLeft),
        "ALT" => Some(KeyCode::AltLeft),
        "ESCAPE" => Some(KeyCode::Escape),
        "ENTER" => Some(KeyCode::Enter),
        "BACKSPACE" => Some(KeyCode::Backspace),
        "DELETE" => Some(KeyCode::Delete),
        "UP" => Some(KeyCode::ArrowUp),
        "DOWN" => Some(KeyCode::ArrowDown),
        "LEFT" => Some(KeyCode::ArrowLeft),
        "RIGHT" => Some(KeyCode::ArrowRight),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
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
    /// Maximum distance for primary-fire hitscan checks.
    pub primary_fire_range: f32,
    /// Cooldown after a successful primary-fire hit.
    pub attack_cooldown: f32,
    /// Cooldown after firing without hitting an enemy.
    pub miss_cooldown: f32,
    /// Radius used by the current prototype ray/sphere hit test.
    pub enemy_hit_radius: f32,
    /// Brief enemy hit-stun duration after being damaged by primary fire.
    pub enemy_hit_stun: f32,
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
            primary_fire_range: 80.0,
            attack_cooldown: 0.25,
            miss_cooldown: 0.15,
            enemy_hit_radius: 2.0,
            enemy_hit_stun: 0.12,
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
    pub anchor_interaction_radius: f32,
    pub anchor_mend_cost: u32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            draw_distance: 500.0,
            fog_density: 0.008,
            anchor_interaction_radius: 2.75,
            anchor_mend_cost: 25,
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
    /// Maximum player health.
    pub max_health: f32,

    /// Maximum stamina shared by sprinting and dashing.
    pub max_stamina: f32,

    /// Stamina restored per second after the regeneration delay.
    pub stamina_regen_rate: f32,

    /// Seconds before stamina regeneration begins.
    pub stamina_regen_delay: f32,

    /// Walking speed in world units per second.
    pub walk_speed: f32,

    /// Sprinting speed in world units per second.
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
    /// Dash speed as a multiplier of sprint speed.
    pub dash_speed_multiplier: f32,

    /// Stamina consumed per second while sprinting.
    pub sprint_stamina_drain_rate: f32,

    /// Stamina consumed when a dash begins.
    pub dash_stamina_cost: f32,

    /// Minimum seconds between dashes.
    pub dash_cooldown: f32,

    /// Dash duration in seconds.
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

    #[test]
    fn strict_loader_rejects_incomplete_binding_files() {
        let dir = std::env::temp_dir().join(format!(
            "cenotaph_config_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let tuning_path = dir.join("tuning.toml");
        let bindings_path = dir.join("bindings.toml");
        std::fs::write(&tuning_path, "").unwrap();
        std::fs::write(&bindings_path, "[keybindings]\nforward = \"W\"\n").unwrap();

        let error = GameConfig::try_load_from_paths(&tuning_path, &bindings_path).unwrap_err();

        assert!(error.contains("missing required action"));
        std::fs::remove_dir_all(dir).ok();
    }
}

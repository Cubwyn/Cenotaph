//! Data-only contracts between gameplay systems and individual HUD widgets.
//! Add widget state here instead of reaching into `EngineState` from rendering.

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerHudState {
    pub health_ratio: f32,
    pub health_trail_ratio: f32,
    pub stamina_ratio: f32,
    pub dash_cooldown_ratio: f32,
    pub health_current: u32,
    pub health_max: u32,
    pub stamina_current: u32,
    pub stamina_max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudMarkerKind {
    Enemy,
    Loot,
    Anchor,
    Hazard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudMarkerState {
    Neutral,
    Aggro,
    Windup,
    Staggered,
}

#[derive(Debug, Clone, Copy)]
pub struct HudWorldMarker {
    pub screen_pos: [f32; 2],
    pub ratio: f32,
    pub distance_m: u32,
    pub kind: HudMarkerKind,
    pub state: HudMarkerState,
    pub state_ratio: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct HudFeedEvent {
    pub label: &'static str,
    pub value: u32,
    pub has_value: bool,
    pub ratio: f32,
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy, Default)]
pub struct HudFeedback {
    pub shot_flash: f32,
    pub hit_marker: f32,
    pub kill_marker: f32,
    pub blocked_flash: f32,
    pub miss_flash: f32,
    pub pickup_flash: f32,
    pub damage_flash: f32,
    pub debug_flash: f32,
    pub spawn_flash: f32,
    pub reload_flash: f32,
    pub loot_flash: f32,
    pub heal_flash: f32,
}

#[derive(Debug, Clone, Default)]
pub struct DialogueHudState {
    pub speaker: String,
    pub line: String,
    pub remaining_ratio: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DebugHudState {
    pub enabled: bool,
    pub enemies: u32,
    pub loot: u32,
    pub unsecured_resource: u32,
    pub banked_resource: u32,
    pub cycle: u32,
    pub props: u32,
    pub fps: u32,
    pub frame_ms: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AscentHudState {
    pub cycle: u32,
    pub cycle_modifier: String,
    pub relic_name: String,
    pub unsecured_resource: u32,
    pub banked_resource: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AnchorRiteHudState {
    pub active: bool,
    pub anchor_name: String,
    pub selected_option: usize,
    pub carried_ash: u32,
    pub bound_ash: u32,
    pub mend_cost: u32,
    pub can_bind: bool,
    pub bind_requirement: String,
    pub can_mend: bool,
    pub vessel_wounded: bool,
}

#[derive(Debug, Clone, Default)]
pub struct NamedNoticeHudState {
    pub active: bool,
    pub title: String,
    pub subtitle: String,
    pub remaining_ratio: f32,
}

#[derive(Debug, Clone, Default)]
pub struct NamedEncounterHudState {
    pub active: bool,
    pub name: String,
    pub health_ratio: f32,
}

#[derive(Debug, Clone, Default)]
pub struct HudFrameState {
    pub viewport_size: [u32; 2],
    pub player: PlayerHudState,
    pub hit_flash: f32,
    pub paused: bool,
    pub dead: bool,
    pub respawn_remaining: f32,
    pub time: f32,
    pub feedback: HudFeedback,
    pub debug: DebugHudState,
    pub ascent: AscentHudState,
    pub anchor_rite: AnchorRiteHudState,
    pub named_notice: NamedNoticeHudState,
    pub named_encounter: NamedEncounterHudState,
    pub markers: Vec<HudWorldMarker>,
    pub event_feed: Vec<HudFeedEvent>,
    pub interaction_prompt: String,
    pub dialogue: DialogueHudState,
    pub level_arrival_ratio: f32,
    pub level_title: String,
    pub level_subtitle: String,
}

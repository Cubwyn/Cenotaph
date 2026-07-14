use std::time::SystemTime;

use crate::data::world::level::{ColliderType, PropData};

const DEFAULT_PLACEMENT_DISTANCE: f32 = 8.0;
const MIN_PLACEMENT_DISTANCE: f32 = 2.0;
const MAX_PLACEMENT_DISTANCE: f32 = 60.0;
const DEFAULT_GRID_SIZE: f32 = 1.0;
const HOT_RELOAD_INTERVAL: f32 = 0.50;
const RELOAD_CONFIRM_SECONDS: f32 = 2.0;
const PICK_PADDING: f32 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorMode {
    Geometry,
    Item,
    Enemy,
    Entity,
}

impl EditorMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Geometry => "GEOMETRY",
            Self::Item => "ITEM",
            Self::Enemy => "ENEMY",
            Self::Entity => "ENTITY",
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Geometry => Self::Item,
            Self::Item => Self::Enemy,
            Self::Enemy => Self::Entity,
            Self::Entity => Self::Geometry,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EditorTemplate {
    pub mode: EditorMode,
    pub label: &'static str,
    pub asset_id: &'static str,
    pub scale: [f32; 3],
    pub collider_type: ColliderType,
    pub is_hurtbox: bool,
    pub item_id: Option<&'static str>,
    pub resource_value: u32,
    pub anchor_id: Option<&'static str>,
    pub enemy_type: Option<&'static str>,
    pub trigger_level_id: Option<&'static str>,
}

impl EditorTemplate {
    pub fn prop_at(self, position: [f32; 3]) -> PropData {
        PropData {
            id: None,
            asset_id: self.asset_id.to_string(),
            position,
            rotation: [0.0, 0.0, 0.0],
            scale: self.scale,
            collider_type: self.collider_type,
            brush_geometry: None,
            is_climbable: false,
            is_hurtbox: self.is_hurtbox,
            item_id: self.item_id.map(str::to_string),
            resource_value: self.resource_value,
            anchor_id: self.anchor_id.map(str::to_string),
            enemy_type: self.enemy_type.map(str::to_string),
            enemy_health: 0.0,
            light_color: None,
            light_intensity: 0.0,
            ambient_sound_id: None,
            trigger_level_id: self.trigger_level_id.map(str::to_string),
            loot_table_id: None,
            path_id: None,
            dialogue_id: None,
            event_id: None,
        }
    }
}

pub const EDITOR_TEMPLATES: &[EditorTemplate] = &[
    EditorTemplate {
        mode: EditorMode::Geometry,
        label: "WALL",
        asset_id: "props/test_wall.obj",
        scale: [8.0, 3.0, 1.0],
        collider_type: ColliderType::Box,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Geometry,
        label: "FLOOR",
        asset_id: "props/test_platform.obj",
        scale: [8.0, 0.5, 8.0],
        collider_type: ColliderType::Box,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Geometry,
        label: "PILLAR",
        asset_id: "props/test_obelisk.obj",
        scale: [1.2, 3.0, 1.2],
        collider_type: ColliderType::Box,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Item,
        label: "RES SHARD",
        asset_id: "pickups/resource_shard.obj",
        scale: [0.35, 0.35, 0.35],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: None,
        resource_value: 25,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Item,
        label: "ASH RELIC",
        asset_id: "pickups/relic_ash_splinter.obj",
        scale: [0.35, 0.35, 0.35],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: Some("ash_splinter"),
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Item,
        label: "VEIL RELIC",
        asset_id: "pickups/relic_veil_cinder.obj",
        scale: [0.35, 0.35, 0.35],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: Some("veil_cinder"),
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Item,
        label: "CHAIN RELIC",
        asset_id: "pickups/relic_chain_sigil.obj",
        scale: [0.35, 0.35, 0.35],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: Some("chain_sigil"),
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Enemy,
        label: "ASHBOUND",
        asset_id: "enemies/ashbound.obj",
        scale: [1.2, 1.2, 1.2],
        collider_type: ColliderType::Sphere,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: Some("ashbound"),
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Enemy,
        label: "BURDENED",
        asset_id: "enemies/burdened.obj",
        scale: [1.5, 1.5, 1.5],
        collider_type: ColliderType::Sphere,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: Some("burdened"),
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Enemy,
        label: "CENSER",
        asset_id: "enemies/censer.obj",
        scale: [1.1, 1.1, 1.1],
        collider_type: ColliderType::Sphere,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: Some("censer"),
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Enemy,
        label: "RUNNER",
        asset_id: "enemies/chainrunner.obj",
        scale: [1.1, 1.1, 1.1],
        collider_type: ColliderType::Sphere,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: Some("chainrunner"),
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Enemy,
        label: "HARPY",
        asset_id: "enemies/harpy.obj",
        scale: [1.2, 1.2, 1.2],
        collider_type: ColliderType::Sphere,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: Some("harpy"),
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Entity,
        label: "ANCHOR",
        asset_id: "world/anchor_marker.obj",
        scale: [0.8, 2.5, 0.8],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: Some("editor_anchor"),
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Entity,
        label: "HAZARD",
        asset_id: "world/hurtbox_warning.obj",
        scale: [1.5, 1.5, 1.5],
        collider_type: ColliderType::Sphere,
        is_hurtbox: true,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: None,
    },
    EditorTemplate {
        mode: EditorMode::Entity,
        label: "GATE",
        asset_id: "world/transition_gate.obj",
        scale: [1.0, 2.0, 1.0],
        collider_type: ColliderType::None,
        is_hurtbox: false,
        item_id: None,
        resource_value: 0,
        anchor_id: None,
        enemy_type: None,
        trigger_level_id: Some("movement_test"),
    },
];

#[derive(Debug, Clone)]
pub struct LevelEditorState {
    pub enabled: bool,
    pub dirty: bool,
    pub selected_prop: Option<usize>,
    pub cursor_position: [f32; 3],
    pub placement_distance: f32,
    pub grid_size: f32,
    pub known_file_modified: Option<SystemTime>,
    mode: EditorMode,
    template_cursor: usize,
    hot_reload_timer: f32,
    reload_confirm_timer: f32,
    message: String,
    validation_label: String,
    validation_current: bool,
    validation_has_errors: bool,
}

impl Default for LevelEditorState {
    fn default() -> Self {
        Self {
            enabled: false,
            dirty: false,
            selected_prop: None,
            cursor_position: [0.0, 0.0, 0.0],
            placement_distance: DEFAULT_PLACEMENT_DISTANCE,
            grid_size: DEFAULT_GRID_SIZE,
            known_file_modified: None,
            mode: EditorMode::Geometry,
            template_cursor: 0,
            hot_reload_timer: HOT_RELOAD_INTERVAL,
            reload_confirm_timer: 0.0,
            message: "TAB OPENS EDITOR".to_string(),
            validation_label: "NOT CHECKED".to_string(),
            validation_current: false,
            validation_has_errors: false,
        }
    }
}

impl LevelEditorState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mode(&self) -> EditorMode {
        self.mode
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn validation_label(&self) -> &str {
        &self.validation_label
    }

    pub fn validation_current(&self) -> bool {
        self.validation_current
    }

    pub fn validation_has_errors(&self) -> bool {
        self.validation_has_errors
    }

    pub fn current_template(&self) -> &'static EditorTemplate {
        let count = self.template_count_for_mode(self.mode).max(1);
        let ordinal = self.template_cursor % count;
        EDITOR_TEMPLATES
            .iter()
            .filter(|template| template.mode == self.mode)
            .nth(ordinal)
            .unwrap_or(&EDITOR_TEMPLATES[0])
    }

    pub fn current_template_label(&self) -> &'static str {
        self.current_template().label
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
        self.reload_confirm_timer = 0.0;
        self.message = if self.enabled {
            "EDITOR ON ENTER PLACE".to_string()
        } else {
            "EDITOR OFF".to_string()
        };
    }

    pub fn tick(&mut self, dt: f32) {
        let dt = dt.max(0.0);
        self.reload_confirm_timer = (self.reload_confirm_timer - dt).max(0.0);
    }

    pub fn should_check_hot_reload(&mut self, dt: f32) -> bool {
        if !self.enabled {
            self.hot_reload_timer = HOT_RELOAD_INTERVAL;
            return false;
        }

        self.hot_reload_timer -= dt.max(0.0);
        if self.hot_reload_timer <= 0.0 {
            self.hot_reload_timer = HOT_RELOAD_INTERVAL;
            true
        } else {
            false
        }
    }

    pub fn set_known_file_modified(&mut self, modified: Option<SystemTime>) {
        self.known_file_modified = modified;
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn mark_dirty(&mut self, message: impl Into<String>) {
        self.dirty = true;
        self.reload_confirm_timer = 0.0;
        self.set_validation_stale();
        self.message = message.into();
    }

    pub fn mark_saved(&mut self, modified: Option<SystemTime>) {
        self.dirty = false;
        self.reload_confirm_timer = 0.0;
        self.known_file_modified = modified;
        self.set_validation_passed();
        self.message = "SAVED CLEAN".to_string();
    }

    pub fn mark_reloaded(&mut self, modified: Option<SystemTime>) {
        self.dirty = false;
        self.reload_confirm_timer = 0.0;
        self.known_file_modified = modified;
        self.set_validation_stale();
        self.message = "RELOADED FROM DISK".to_string();
    }

    pub fn next_mode(&mut self) {
        self.mode = self.mode.next();
        self.template_cursor = 0;
        self.message = format!("CATEGORY {}", self.mode.label());
    }

    pub fn next_template(&mut self) {
        let count = self.template_count_for_mode(self.mode).max(1);
        self.template_cursor = (self.template_cursor + 1) % count;
        self.message = format!("TEMPLATE {}", self.current_template_label());
    }

    pub fn previous_template(&mut self) {
        let count = self.template_count_for_mode(self.mode).max(1);
        self.template_cursor = (self.template_cursor + count - 1) % count;
        self.message = format!("TEMPLATE {}", self.current_template_label());
    }

    pub fn select_next(&mut self, prop_count: usize) {
        if prop_count == 0 {
            self.selected_prop = None;
            self.message = "NO PROPS TO SELECT".to_string();
            return;
        }

        let next = self
            .selected_prop
            .map(|index| (index + 1) % prop_count)
            .unwrap_or(0);
        self.selected_prop = Some(next);
        self.message = format!("SELECTED PROP {}", next + 1);
    }

    pub fn select_previous(&mut self, prop_count: usize) {
        if prop_count == 0 {
            self.selected_prop = None;
            self.message = "NO PROPS TO SELECT".to_string();
            return;
        }

        let next = self
            .selected_prop
            .map(|index| (index + prop_count - 1) % prop_count)
            .unwrap_or(0);
        self.selected_prop = Some(next);
        self.message = format!("SELECTED PROP {}", next + 1);
    }

    pub fn clamp_selection(&mut self, prop_count: usize) {
        if let Some(index) = self.selected_prop {
            if index >= prop_count {
                self.selected_prop = prop_count.checked_sub(1);
            }
        }
    }

    pub fn set_cursor_position(&mut self, position: [f32; 3]) {
        self.cursor_position = position;
    }

    pub fn adjust_distance(&mut self, delta: f32) {
        if delta.abs() <= f32::EPSILON {
            return;
        }
        self.placement_distance =
            (self.placement_distance + delta).clamp(MIN_PLACEMENT_DISTANCE, MAX_PLACEMENT_DISTANCE);
        self.message = format!("DISTANCE {}M", self.placement_distance.round() as u32);
    }

    pub fn request_reload(&mut self) -> bool {
        if !self.dirty {
            return true;
        }

        if self.reload_confirm_timer > 0.0 {
            self.reload_confirm_timer = 0.0;
            true
        } else {
            self.reload_confirm_timer = RELOAD_CONFIRM_SECONDS;
            self.message = "UNSAVED PRESS R AGAIN".to_string();
            false
        }
    }

    pub fn mark_validation_passed(&mut self) {
        self.set_validation_passed();
        self.message = "VALIDATION OK".to_string();
    }

    pub fn mark_validation_failed(&mut self, issue_count: usize) {
        self.set_validation_failed(issue_count);
        self.message = format!("VALIDATION {}", self.validation_label);
    }

    pub fn disk_changed(&self, modified: Option<SystemTime>) -> bool {
        let Some(modified) = modified else {
            return false;
        };
        self.known_file_modified
            .is_some_and(|known| modified > known)
    }

    fn template_count_for_mode(&self, mode: EditorMode) -> usize {
        EDITOR_TEMPLATES
            .iter()
            .filter(|template| template.mode == mode)
            .count()
    }

    fn set_validation_stale(&mut self) {
        self.validation_label = "CHECK NEEDED".to_string();
        self.validation_current = false;
        self.validation_has_errors = false;
    }

    fn set_validation_passed(&mut self) {
        self.validation_label = "OK".to_string();
        self.validation_current = true;
        self.validation_has_errors = false;
    }

    fn set_validation_failed(&mut self, issue_count: usize) {
        let noun = if issue_count == 1 { "ISSUE" } else { "ISSUES" };
        self.validation_label = format!("{} {}", issue_count, noun);
        self.validation_current = true;
        self.validation_has_errors = true;
    }
}

pub fn snap_position(position: [f32; 3], grid_size: f32) -> [f32; 3] {
    let grid = grid_size.max(0.001);
    [
        (position[0] / grid).round() * grid,
        (position[1] / grid).round() * grid,
        (position[2] / grid).round() * grid,
    ]
}

pub fn prop_pick_radius(scale: [f32; 3]) -> f32 {
    scale
        .into_iter()
        .map(f32::abs)
        .fold(0.0, f32::max)
        .mul_add(0.5, PICK_PADDING)
}

pub fn cursor_can_pick_prop(cursor: [f32; 3], prop: &PropData) -> bool {
    let dx = cursor[0] - prop.position[0];
    let dy = cursor[1] - prop.position[1];
    let dz = cursor[2] - prop.position[2];
    let radius = prop_pick_radius(prop.scale);
    dx * dx + dy * dy + dz * dz <= radius * radius
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_template_materializes_solid_prop() {
        let template = EDITOR_TEMPLATES
            .iter()
            .find(|template| template.label == "WALL")
            .unwrap();

        let prop = template.prop_at([1.0, 2.0, 3.0]);

        assert_eq!(prop.asset_id, "props/test_wall.obj");
        assert_eq!(prop.position, [1.0, 2.0, 3.0]);
        assert_eq!(prop.collider_type, ColliderType::Box);
        assert!(prop.enemy_type.is_none());
    }

    #[test]
    fn editor_cycles_templates_inside_current_mode() {
        let mut editor = LevelEditorState::new();

        assert_eq!(editor.current_template_label(), "WALL");
        editor.next_template();
        assert_eq!(editor.current_template_label(), "FLOOR");
        editor.next_mode();

        assert_eq!(editor.mode(), EditorMode::Item);
        assert_eq!(editor.current_template_label(), "RES SHARD");
    }

    #[test]
    fn dirty_reload_requires_confirmation() {
        let mut editor = LevelEditorState::new();
        editor.mark_dirty("PLACED");

        assert!(!editor.request_reload());
        assert!(editor.request_reload());
    }

    #[test]
    fn dirty_edits_mark_validation_stale_until_checked() {
        let mut editor = LevelEditorState::new();
        editor.mark_validation_passed();

        editor.mark_dirty("PLACED WALL");

        assert_eq!(editor.validation_label(), "CHECK NEEDED");
        assert!(!editor.validation_current());
        assert!(!editor.validation_has_errors());
    }

    #[test]
    fn validation_feedback_tracks_issue_count() {
        let mut editor = LevelEditorState::new();

        editor.mark_validation_failed(2);

        assert_eq!(editor.validation_label(), "2 ISSUES");
        assert!(editor.validation_current());
        assert!(editor.validation_has_errors());
        assert_eq!(editor.message(), "VALIDATION 2 ISSUES");
    }

    #[test]
    fn snap_position_uses_grid_size() {
        assert_eq!(snap_position([1.2, 2.6, -3.4], 1.0), [1.0, 3.0, -3.0]);
        assert_eq!(snap_position([1.2, 2.6, -3.4], 0.5), [1.0, 2.5, -3.5]);
    }

    #[test]
    fn cursor_pick_radius_scales_with_large_geometry() {
        assert_eq!(prop_pick_radius([8.0, 3.0, 1.0]), 6.0);

        let wall = EDITOR_TEMPLATES[0].prop_at([0.0, 0.0, 0.0]);
        assert!(cursor_can_pick_prop([5.9, 0.0, 0.0], &wall));
        assert!(!cursor_can_pick_prop([6.1, 0.0, 0.0], &wall));
    }
}

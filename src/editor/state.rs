// src/editor/state.rs
// EditorState — Minecraft creative-style in-world level editor.
//
// ── Controls ─────────────────────────────────────────────────────────────────
//  Tab          Toggle editor on/off
//  WASD         Fly (gravity disabled)
//  E / Q        Fly up / down
//  F            Toggle fast-fly (4× speed)
//  LMB          Place prop / confirm grab
//  RMB          Select prop (copies asset to hotbar) / cancel grab
//  Middle       Copy hovered prop's asset to hotbar (no select)
//  Scroll       Cycle hotbar slot
//  1 – 9        Jump to hotbar slot
//  G            Grab selected prop
//  X            Delete selected prop
//  R            Rotate 45° on Y
//  T            Cycle collider type
//  Ctrl+D       Duplicate
//  Ctrl+Z       Undo (20 steps)
//  Ctrl+S       Save level
// ─────────────────────────────────────────────────────────────────────────────
#![allow(dead_code)]

use crate::editor::scan_assets_folder;
use crate::world::level::{ColliderType, PropData};

// ── Undo record ───────────────────────────────────────────────────────────────

pub struct UndoSnapshot {
    pub props: Vec<PropData>,
}

const MAX_UNDO: usize = 20;

// ── Hotbar ────────────────────────────────────────────────────────────────────

pub type HotbarSlot = Option<String>;

// ── EditorState ───────────────────────────────────────────────────────────────

pub struct EditorState {
    pub is_enabled: bool,

    pub hotbar: [HotbarSlot; 9],
    pub active_slot: usize,
    pub spawnable_assets: Vec<String>,

    pub available_colliders: Vec<ColliderType>,
    pub current_collider_idx: usize,

    pub is_climbable: bool,
    pub is_hurtbox: bool,
    pub item_id: Option<String>,
    pub enemy_type: Option<String>,
    pub enemy_health: f32,
    pub light_color: [f32; 3],
    pub light_intensity: f32,
    pub ambient_sound_id: Option<String>,
    pub trigger_level_id: Option<String>,

    pub selected_idx: Option<usize>,
    pub is_grabbing: bool,
    pub grab_distance: f32,

    pub fast_fly: bool,
    /// Noclip: camera passes through all geometry (always true in editor,
    /// but can be toggled off to test collision placement).
    pub noclip: bool,
    pub undo_stack: Vec<UndoSnapshot>,
    pub cooldown: u32,
}

impl EditorState {
    pub fn new() -> Self {
        let assets = scan_assets_folder();
        let mut hotbar: [HotbarSlot; 9] = Default::default();
        for (i, name) in assets.iter().take(9).enumerate() {
            hotbar[i] = Some(name.clone());
        }
        Self {
            is_enabled: false,
            hotbar,
            active_slot: 0,
            spawnable_assets: assets,
            available_colliders: vec![
                ColliderType::Box,
                ColliderType::Sphere,
                ColliderType::Mesh,
                ColliderType::None,
            ],
            current_collider_idx: 0,
            is_climbable: false,
            is_hurtbox: false,
            item_id: None,
            enemy_type: None,
            enemy_health: 100.0,
            light_color: [1.0, 1.0, 1.0],
            light_intensity: 1.0,
            ambient_sound_id: None,
            trigger_level_id: None,
            selected_idx: None,
            is_grabbing: false,
            grab_distance: 5.0,
            fast_fly: false,
            noclip: true,
            undo_stack: Vec::new(),
            cooldown: 0,
        }
    }

    // ── Toggle ────────────────────────────────────────────────────────────────

    pub fn toggle(&mut self) {
        self.is_enabled = !self.is_enabled;
        if self.is_enabled {
            self.spawnable_assets = scan_assets_folder();
            println!("╔══════════════════════════════════════════════╗");
            println!("║           CENOTAPH LEVEL EDITOR              ║");
            println!("╚══════════════════════════════════════════════╝");
            println!("[EDITOR] Status    : ON");
            println!("[EDITOR] Assets    : {} found", self.spawnable_assets.len());
            println!("[EDITOR] Fly speed : {}", if self.fast_fly { "FAST (4×)" } else { "normal" });
            println!("[EDITOR] Undo stack: {}/{} steps", self.undo_stack.len(), MAX_UNDO);
            println!("[EDITOR] Collider  : {:?}", self.available_colliders[self.current_collider_idx]);
            self.print_hotbar();
            self.print_controls();
        } else {
            self.cancel_grab();
            self.selected_idx = None;
            println!("[EDITOR] ──────────────────────────────────────");
            println!("[EDITOR] Status    : OFF  (gameplay mode)");
            println!("[EDITOR] ──────────────────────────────────────");
        }
    }

    fn print_controls(&self) {
        println!("[EDITOR] ── Controls ──────────────────────────────");
        println!("[EDITOR]  WASD/E/Q   fly   |  F  fast-fly toggle");
        println!("[EDITOR]  LMB        place / drop grab");
        println!("[EDITOR]  RMB        select / cancel grab");
        println!("[EDITOR]  Middle     pick asset to hotbar");
        println!("[EDITOR]  Scroll     cycle hotbar slot");
        println!("[EDITOR]  1-9        jump to hotbar slot");
        println!("[EDITOR]  G          grab selected prop");
        println!("[EDITOR]  X          delete selected prop");
        println!("[EDITOR]  R          rotate 45° Y");
        println!("[EDITOR]  T          cycle collider type");
        println!("[EDITOR]  Ctrl+D     duplicate");
        println!("[EDITOR]  Ctrl+Z     undo ({} steps available)", self.undo_stack.len());
        println!("[EDITOR]  Ctrl+S     save level");
        println!("[EDITOR] ──────────────────────────────────────────");
    }

    // ── Hotbar ────────────────────────────────────────────────────────────────
 
    pub fn active_asset(&self) -> Option<&str> {
        self.hotbar[self.active_slot].as_deref()
    }

    pub fn set_active_slot(&mut self, slot: usize) {
        self.active_slot = slot.min(8);
        let asset = self.hotbar[self.active_slot].as_deref().unwrap_or("(empty)");
        println!("[EDITOR] Hotbar slot : {} → \"{}\"", self.active_slot + 1, asset);
        self.print_hotbar();
    }

    pub fn scroll_hotbar(&mut self, delta: i32) {
        let n = 9i32;
        self.active_slot = ((self.active_slot as i32 + delta).rem_euclid(n)) as usize;
        let asset = self.hotbar[self.active_slot].as_deref().unwrap_or("(empty)");
        println!("[EDITOR] Hotbar scroll → slot {} \"{}\"", self.active_slot + 1, asset);
        self.print_hotbar();
    }

    pub fn assign_to_active_slot(&mut self, asset_name: impl Into<String>) {
        let name = asset_name.into();
        println!("[EDITOR] Pick-block  : slot {} ← \"{}\"", self.active_slot + 1, name);
        self.hotbar[self.active_slot] = Some(name);
        self.print_hotbar();
    }

    pub fn print_hotbar(&self) {
        let slots: Vec<String> = self.hotbar.iter().enumerate().map(|(i, s)| {
            let label = s.as_deref().unwrap_or("·");
            if i == self.active_slot { format!("[{}]", label) } else { label.to_string() }
        }).collect();
        println!("[EDITOR] Hotbar      : {}", slots.join("  "));
    }

    // ── Collider cycling ──────────────────────────────────────────────────────

    pub fn cycle_collider(&mut self) {
        if !self.available_colliders.is_empty() {
            self.current_collider_idx =
                (self.current_collider_idx + 1) % self.available_colliders.len();
            let col = &self.available_colliders[self.current_collider_idx];
            println!("[EDITOR] Collider    : {:?}  ({}/{})",
                col,
                self.current_collider_idx + 1,
                self.available_colliders.len()
            );
        }
    }

    pub fn current_collider(&self) -> ColliderType {
        self.available_colliders[self.current_collider_idx].clone()
    }

    // ── Selection ─────────────────────────────────────────────────────────────

    pub fn select_prop(&mut self, index: usize, prop: &PropData) {
        self.selected_idx = Some(index);
        self.is_grabbing = false;
        self.is_climbable = prop.is_climbable;
        self.is_hurtbox = prop.is_hurtbox;
        self.item_id = prop.item_id.clone();
        self.enemy_type = prop.enemy_type.clone();
        self.enemy_health = prop.enemy_health;
        self.light_color = prop.light_color.unwrap_or([1.0, 1.0, 1.0]);
        self.light_intensity = prop.light_intensity;
        self.ambient_sound_id = prop.ambient_sound_id.clone();
        self.trigger_level_id = prop.trigger_level_id.clone();
        self.hotbar[self.active_slot] = Some(prop.asset_id.clone());

        println!("[EDITOR] ── Selected prop #{} ─────────────────────", index);
        println!("[EDITOR]  Asset      : {}", prop.asset_id);
        println!("[EDITOR]  Position   : ({:.2}, {:.2}, {:.2})",
            prop.position[0], prop.position[1], prop.position[2]);
        println!("[EDITOR]  Rotation   : ({:.1}°, {:.1}°, {:.1}°)",
            prop.rotation[0].to_degrees(),
            prop.rotation[1].to_degrees(),
            prop.rotation[2].to_degrees());
        println!("[EDITOR]  Scale      : ({:.2}, {:.2}, {:.2})",
            prop.scale[0], prop.scale[1], prop.scale[2]);
        println!("[EDITOR]  Collider   : {:?}", prop.collider_type);
        println!("[EDITOR]  Climbable  : {}  |  Hurtbox: {}", prop.is_climbable, prop.is_hurtbox);
        if let Some(et) = &prop.enemy_type {
            println!("[EDITOR]  Enemy      : {}  HP: {}", et, prop.enemy_health);
        }
        if let Some(id) = &prop.item_id {
            println!("[EDITOR]  Item ID    : {}", id);
        }
        println!("[EDITOR]  Light      : intensity={:.2}  color=({:.2},{:.2},{:.2})",
            prop.light_intensity,
            prop.light_color.unwrap_or([1.0,1.0,1.0])[0],
            prop.light_color.unwrap_or([1.0,1.0,1.0])[1],
            prop.light_color.unwrap_or([1.0,1.0,1.0])[2]);
        println!("[EDITOR]  Undo stack : {} steps", self.undo_stack.len());
        println!("[EDITOR]  G=grab  X=delete  R=rotate  T=collider  Ctrl+D=dup");
    }

    pub fn deselect(&mut self) {
        if self.selected_idx.is_some() {
            println!("[EDITOR] Deselected  : no prop selected");
        }
        self.selected_idx = None;
        self.is_grabbing = false;
    }

    // ── Grab / move ───────────────────────────────────────────────────────────

    pub fn start_grab(&mut self) {
        if self.selected_idx.is_some() {
            self.is_grabbing = true;
            println!("[EDITOR] ── GRAB MODE ─────────────────────────────");
            println!("[EDITOR]  Move camera to reposition prop");
            println!("[EDITOR]  LMB = drop here   RMB = cancel");
            println!("[EDITOR]  Grab distance: {:.1} units", self.grab_distance);
        } else {
            println!("[EDITOR] Grab        : no prop selected (RMB to select)");
        }
    }

    pub fn confirm_grab(&mut self) {
        self.is_grabbing = false;
        println!("[EDITOR] Grab drop   : prop placed at new position");
    }

    pub fn cancel_grab(&mut self) {
        if self.is_grabbing {
            println!("[EDITOR] Grab cancel : position restored via undo");
        }
        self.is_grabbing = false;
    }

    // ── Fly speed ─────────────────────────────────────────────────────────────

    pub fn toggle_fast_fly(&mut self) {
        self.fast_fly = !self.fast_fly;
        println!("[EDITOR] Fly speed   : {}",
            if self.fast_fly { "FAST (4×)" } else { "normal (1×)" });
    }

    pub fn fly_multiplier(&self) -> f32 {
        if self.fast_fly { 4.0 } else { 1.0 }
    }

    // ── Noclip ────────────────────────────────────────────────────────────────

    pub fn toggle_noclip(&mut self) {
        self.noclip = !self.noclip;
        println!("[EDITOR] Noclip      : {}  (camera {} geometry)",
            if self.noclip { "ON" } else { "OFF" },
            if self.noclip { "passes through" } else { "collides with" });
    }

    // ── Attribute toggles ─────────────────────────────────────────────────────

    pub fn toggle_climbable(&mut self) {
        self.is_climbable = !self.is_climbable;
        println!("[EDITOR] Climbable   : {}", self.is_climbable);
    }

    pub fn toggle_hurtbox(&mut self) {
        self.is_hurtbox = !self.is_hurtbox;
        println!("[EDITOR] Hurtbox     : {}", self.is_hurtbox);
    }

    pub fn adjust_light_intensity(&mut self, delta: f32) {
        self.light_intensity = (self.light_intensity + delta).max(0.0);
        println!("[EDITOR] Light       : intensity={:.2}", self.light_intensity);
    }

    pub fn set_item_id(&mut self, id: String) {
        println!("[EDITOR] Item ID     : {}", id);
        self.item_id = Some(id);
    }

    pub fn set_enemy_type(&mut self, enemy: String) {
        println!("[EDITOR] Enemy type  : {}", enemy);
        self.enemy_type = Some(enemy);
    }

    // ── Undo ──────────────────────────────────────────────────────────────────

    pub fn push_undo(&mut self, props: &[PropData]) {
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(UndoSnapshot { props: props.to_vec() });
        println!("[EDITOR] Undo push   : {} step(s) stored", self.undo_stack.len());
    }

    pub fn pop_undo(&mut self) -> Option<Vec<PropData>> {
        let result = self.undo_stack.pop().map(|s| s.props);
        if result.is_some() {
            println!("[EDITOR] Undo pop    : {} step(s) remaining", self.undo_stack.len());
        }
        result
    }

    // ── Staging → PropData ────────────────────────────────────────────────────

    pub fn build_prop(&self, asset_id: String, position: [f32; 3]) -> PropData {
        let prop = PropData {
            asset_id: asset_id.clone(),
            position,
            rotation: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            collider_type: self.current_collider(),
            is_climbable: self.is_climbable,
            is_hurtbox: self.is_hurtbox,
            item_id: self.item_id.clone(),
            enemy_type: self.enemy_type.clone(),
            enemy_health: self.enemy_health,
            light_color: Some(self.light_color),
            light_intensity: self.light_intensity,
            ambient_sound_id: self.ambient_sound_id.clone(),
            trigger_level_id: self.trigger_level_id.clone(),
        };
        println!("[EDITOR] Build prop  : \"{}\"  pos=({:.2},{:.2},{:.2})  collider={:?}",
            asset_id, position[0], position[1], position[2],
            prop.collider_type);
        prop
    }
}

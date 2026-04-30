// src/engine/update.rs
// Per-frame update logic split into two passes:
//
//   update_physics(input)  — movement intent, physics step, gameplay/editor
//   update_visuals(input)  — camera rotation, GPU buffer writes
//
// Editor input is compiled out entirely when the `editor` feature is absent.

use glam::Vec3;
use winit::event::MouseButton;
use winit::keyboard::KeyCode;

use crate::engine::state::EngineState;
use crate::input::InputManager;

// ── Ray helper ────────────────────────────────────────────────────────────────

// Returns true if the ray (origin + t*dir) passes within `radius` of `center`.
fn ray_hits_sphere(origin: Vec3, dir: Vec3, center: Vec3, radius: f32) -> bool {
    let oc = center - origin;
    let proj = oc.dot(dir);
    if proj < 0.0 { return false; }
    let closest = origin + dir * proj;
    closest.distance_squared(center) < radius * radius
}

// ── Public update interface ───────────────────────────────────────────────────

impl EngineState {
    // Physics pass: movement intent → physics step → gameplay or editor logic.
    pub fn update_physics(&mut self, input: &InputManager) {
        // Tick the shared cooldown timer
        if self.action_cooldown > 0 {
            self.action_cooldown -= 1;
        }

        // ── Editor toggle (dev builds only) ───────────────────────────────────
        #[cfg(feature = "editor")]
        {
            if input.is_key_down(self.config_data.key("editor_toggle"))
                && self.action_cooldown == 0
            {
                self.editor.toggle();
                self.action_cooldown = 20;
            }
        }

        // ── Dispatch to editor or gameplay ────────────────────────────────────
        #[cfg(feature = "editor")]
        let in_editor = self.editor.is_enabled;
        #[cfg(not(feature = "editor"))]
        let in_editor = false;

        // ── Editor: move camera directly, freeze physics body ─────────────────
        #[cfg(feature = "editor")]
        if in_editor {
            self.handle_editor_input(input);
            self.handle_editor_movement(input);
            // Freeze the physics body so it doesn't drift or fall
            self.physics.freeze_player();
            self.physics.step(&self.config_data.physics);
            return; // skip gameplay movement + second step below
        }

        // ── Gameplay movement ─────────────────────────────────────────────────
        let intent = {
            let v = self.camera_controller
                .get_movement_intent(input, &self.camera, &self.config_data);
            [v.x, v.y, v.z]
        };
        let is_jumping = input.is_key_down(self.config_data.key("jump"));

        if !in_editor {
            self.handle_gameplay_input(input);
        }

        // ── Save level (Ctrl+S, editor builds only) ───────────────────────────
        #[cfg(feature = "editor")]
        {
            if input.is_key_down(Some(KeyCode::ControlLeft))
                && input.is_key_down(Some(KeyCode::KeyS))
                && self.action_cooldown == 0
            {
                std::fs::create_dir_all("levels").unwrap_or_default();
                let save_path = format!("levels/{}.json", self.level_name);
                let _ = self.level_data.save(&save_path);
                self.action_cooldown = 60;
                println!("[EDITOR] ── SAVE ──────────────────────────────────");
                println!("[EDITOR] Save path   : {}", save_path);
                println!("[EDITOR] Props saved : {}", self.level_data.props.len());
                println!("[EDITOR] Level name  : {}", self.level_name);
                println!("[EDITOR] ──────────────────────────────────────────");
            }
        }

        self.physics.apply_player_movement(
            intent,
            is_jumping,
            in_editor,
            &self.config_data.physics,
        );
        self.physics.step(&self.config_data.physics);
        if self.action_cooldown % 60 == 0 {
            let pos = self.physics.get_player_pos();
            println!("[DEBUG] Player pos: ({:.2}, {:.2}, {:.2})", pos[0], pos[1], pos[2]);
        }
    }

    /// Visuals pass: mouse look → camera → GPU buffer write.
    /// Also handles scroll-wheel hotbar cycling (editor only).
    pub fn update_visuals(&mut self, input: &mut InputManager) {
        // ── Scroll wheel → hotbar (editor only) ──────────────────────────────
        #[cfg(feature = "editor")]
        {
            let scroll = input.take_scroll();
            if self.editor.is_enabled && scroll != 0.0 {
                // Scroll up (+) = previous slot, scroll down (-) = next slot
                // (matches Minecraft behaviour)
                let dir = if scroll > 0.0 { -1i32 } else { 1i32 };
                self.editor.scroll_hotbar(dir);
            }
        }
        #[cfg(not(feature = "editor"))]
        { let _ = input.take_scroll(); }

        if input.mouse_delta.0 != 0.0 || input.mouse_delta.1 != 0.0 {
            self.camera_controller
                .process_mouse(input.mouse_delta.0, input.mouse_delta.1, &mut self.camera);
        }

        // In editor mode the camera IS the player position (free-fly).
        // In gameplay mode the camera follows the physics body.
        #[cfg(feature = "editor")]
        let in_editor = self.editor.is_enabled;
        #[cfg(not(feature = "editor"))]
        let in_editor = false;

        if !in_editor {
            let p = self.physics.get_player_pos();
            self.camera.position = Vec3::new(p[0], p[1] + 1.0, p[2]);
        }

        // If a prop is being grabbed, keep it floating in front of the camera
        #[cfg(feature = "editor")]
        if self.editor.is_grabbing {
            if let Some(i) = self.editor.selected_idx {
                if i < self.level_data.props.len() {
                    let fwd = self.camera.get_forward();
                    let target = self.camera.position + fwd * self.editor.grab_distance;
                    self.level_data.props[i].position = [target.x, target.y, target.z];
                    self.sync_instances();
                }
            }
        }

        self.camera_uniform.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        self.update_lighting();
        input.reset_mouse_delta();
    }
}

// ── Gameplay input (always compiled) ─────────────────────────────────────────

impl EngineState {
    fn handle_gameplay_input(&mut self, input: &InputManager) {
        // Shoot (LMB)
        if input.is_mouse_down(MouseButton::Left) && self.action_cooldown == 0 {
            let ray_origin = self.camera.position;
            let ray_dir = self.camera.get_forward();

            let hit = self
                .level_data
                .props
                .iter()
                .enumerate()
                .find(|(_, p)| {
                    p.enemy_type.is_some()
                        && ray_hits_sphere(
                            ray_origin,
                            ray_dir,
                            Vec3::new(p.position[0], p.position[1], p.position[2]),
                            1.5,
                        )
                })
                .map(|(i, _)| i);

            if let Some(_i) = hit {
                println!("[COMBAT] Hit registered, but combat system is currently disabled.");
                self.action_cooldown = 10;
            }
        }

        // Note: Combat system has been removed as it was half-baked.
        // Once enemy AI and combat are properly implemented, re-integrate damage here.
    }
}

// ── Editor movement (dev builds only) ────────────────────────────────────────
// Moves the camera directly — no physics involved.

#[cfg(feature = "editor")]
impl EngineState {
    pub(crate) fn handle_editor_movement(&mut self, input: &InputManager) {
        let speed = self.config_data.physics.player_speed
            * 2.0
            * self.editor.fly_multiplier();
        let dt = 1.0 / 60.0_f32; // fixed timestep approximation

        // WASD — horizontal fly (camera-relative, flat XZ)
        let yaw = self.camera.yaw;
        let forward_flat = Vec3::new(yaw.cos(), 0.0, yaw.sin()).normalize();
        let right_flat    = Vec3::new(-yaw.sin(), 0.0, yaw.cos()).normalize();

        let mut delta = Vec3::ZERO;
        if input.is_key_down(self.config_data.key("forward"))  { delta += forward_flat; }
        if input.is_key_down(self.config_data.key("backward")) { delta -= forward_flat; }
        if input.is_key_down(self.config_data.key("right"))    { delta += right_flat; }
        if input.is_key_down(self.config_data.key("left"))     { delta -= right_flat; }

        // E / Q — vertical fly
        if input.is_key_down(self.config_data.key("fly_up"))   { delta.y += 1.0; }
        if input.is_key_down(self.config_data.key("fly_down")) { delta.y -= 1.0; }

        if delta.length_squared() > 0.0 {
            delta = delta.normalize();
        }
        self.camera.position += delta * speed * dt;

        // ── Home — teleport to origin ─────────────────────────────────────────
        if input.is_key_down(Some(KeyCode::Home)) && self.action_cooldown == 0 {
            self.camera.position = Vec3::ZERO;
            self.action_cooldown = 20;
            println!("[EDITOR] Teleport    : → origin (0, 0, 0)");
        }

        // ── N — noclip toggle (always on in editor, but toggleable for testing) ─
        if input.is_key_down(self.config_data.key("editor_noclip")) && self.action_cooldown == 0 {
            self.editor.toggle_noclip();
            self.action_cooldown = 20;
        }

        // ── Print camera position every time it changes significantly ─────────
        // (only when actually moving, not every frame)
        if delta.length_squared() > 0.0 {
            let p = self.camera.position;
            // throttle: print at most once per ~30 frames
            if self.action_cooldown == 0 {
                println!("[EDITOR] Camera pos  : ({:.2}, {:.2}, {:.2})  fly={}×",
                    p.x, p.y, p.z,
                    self.editor.fly_multiplier() as u32);
            }
        }
    }
}

// ── Editor input (dev builds only) ───────────────────────────────────────────

#[cfg(feature = "editor")]
impl EngineState {
    pub(crate) fn handle_editor_input(&mut self, input: &InputManager) {
        use crate::render::mesh::load_model;

        let ctrl = input.is_key_down(Some(KeyCode::ControlLeft))
            || input.is_key_down(Some(KeyCode::ControlRight));

        // ── Hotbar: 1-9 keys ──────────────────────────────────────────────────
        let slot_keys = [
            (KeyCode::Digit1, 0usize),
            (KeyCode::Digit2, 1),
            (KeyCode::Digit3, 2),
            (KeyCode::Digit4, 3),
            (KeyCode::Digit5, 4),
            (KeyCode::Digit6, 5),
            (KeyCode::Digit7, 6),
            (KeyCode::Digit8, 7),
            (KeyCode::Digit9, 8),
        ];
        for (key, slot) in slot_keys {
            if input.is_key_down(Some(key)) && self.action_cooldown == 0 {
                self.editor.set_active_slot(slot);
                self.action_cooldown = 10;
                break;
            }
        }

        // ── Fast-fly toggle (F) ───────────────────────────────────────────────
        if input.is_key_down(self.config_data.key("editor_fly_fast"))
            && self.action_cooldown == 0
        {
            self.editor.toggle_fast_fly();
            self.action_cooldown = 15;
        }

        // ── Collider cycle (T) ────────────────────────────────────────────────
        if input.is_key_down(self.config_data.key("editor_cycle_collider"))
            && self.action_cooldown == 0
        {
            self.editor.cycle_collider();
            // Apply to selected prop immediately
            if let Some(i) = self.editor.selected_idx {
                self.level_data.props[i].collider_type = self.editor.current_collider();
            }
            self.action_cooldown = 15;
        }

        // ── Grab (G) ──────────────────────────────────────────────────────────
        if input.is_key_down(self.config_data.key("editor_grab"))
            && self.action_cooldown == 0
        {
            if self.editor.is_grabbing {
                self.editor.confirm_grab();
            } else {
                self.editor.start_grab();
            }
            self.action_cooldown = 15;
        }

        // ── Rotate selected prop 45° on Y (R) ─────────────────────────────────
        if input.is_key_down(self.config_data.key("editor_rotate"))
            && self.action_cooldown == 0
        {
            if let Some(i) = self.editor.selected_idx {
                self.editor.push_undo(&self.level_data.props);
                self.level_data.props[i].rotation[1] += std::f32::consts::FRAC_PI_4;
                self.sync_instances();
                let deg = self.level_data.props[i].rotation[1].to_degrees();
                println!("[EDITOR] Rotate      : prop #{} Y={:.1}°  (total props: {})",
                    i, deg, self.level_data.props.len());
            } else {
                println!("[EDITOR] Rotate      : no prop selected");
            }
            self.action_cooldown = 10;
        }

        // ── Delete selected prop (X) ──────────────────────────────────────────
        if input.is_key_down(self.config_data.key("editor_delete"))
            && self.action_cooldown == 0
        {
            if let Some(i) = self.editor.selected_idx {
                self.editor.push_undo(&self.level_data.props);
                let name = self.level_data.props[i].asset_id.clone();
                let pos = self.level_data.props[i].position;
                self.level_data.props.remove(i);
                self.editor.selected_idx = None;
                self.editor.is_grabbing = false;
                self.sync_instances();
                println!("[EDITOR] Delete      : prop #{} \"{}\"  was at ({:.2},{:.2},{:.2})",
                    i, name, pos[0], pos[1], pos[2]);
                println!("[EDITOR]               {} prop(s) remaining  |  Ctrl+Z to undo",
                    self.level_data.props.len());
            } else {
                println!("[EDITOR] Delete      : no prop selected (RMB to select)");
            }
            self.action_cooldown = 20;
        }

        // ── Duplicate selected prop (Ctrl+D) ──────────────────────────────────
        if ctrl
            && input.is_key_down(Some(KeyCode::KeyD))
            && self.action_cooldown == 0
        {
            if let Some(i) = self.editor.selected_idx {
                self.editor.push_undo(&self.level_data.props);
                let mut copy = self.level_data.props[i].clone();
                copy.position[0] += 0.5;
                copy.position[2] += 0.5;
                let new_idx = self.level_data.props.len();
                let new_pos = copy.position;
                let asset = copy.asset_id.clone();
                self.level_data.props.push(copy);
                self.editor.selected_idx = Some(new_idx);
                self.sync_instances();
                println!("[EDITOR] Duplicate   : prop #{} → new prop #{}  \"{}\"",
                    i, new_idx, asset);
                println!("[EDITOR]               new pos ({:.2},{:.2},{:.2})  |  {} total props",
                    new_pos[0], new_pos[1], new_pos[2], self.level_data.props.len());
            } else {
                println!("[EDITOR] Duplicate   : no prop selected (RMB to select)");
            }
            self.action_cooldown = 20;
        }

        // ── Undo (Ctrl+Z) ─────────────────────────────────────────────────────
        if ctrl
            && input.is_key_down(Some(KeyCode::KeyZ))
            && self.action_cooldown == 0
        {
            if let Some(restored) = self.editor.pop_undo() {
                self.level_data.props = restored;
                self.editor.selected_idx = None;
                self.editor.is_grabbing = false;
                self.sync_instances();
                println!("[EDITOR] Undo — {} props remaining", self.level_data.props.len());
            } else {
                println!("[EDITOR] Nothing to undo");
            }
            self.action_cooldown = 15;
        }

        // ── LMB: place prop or confirm grab ───────────────────────────────────
        if input.is_mouse_down(MouseButton::Left) && self.action_cooldown == 0 {
            if self.editor.is_grabbing {
                // Drop the grabbed prop where it currently floats
                self.editor.confirm_grab();
                self.action_cooldown = 15;
            } else {
                // Place a new prop from the active hotbar slot
                if let Some(asset_id) = self.editor.active_asset().map(|s| s.to_string()) {
                    let fwd = self.camera.get_forward();
                    let place_pos = self.camera.position + fwd * 5.0;
                    let prop = self.editor.build_prop(
                        asset_id.clone(),
                        [place_pos.x, place_pos.y, place_pos.z],
                    );

                    self.editor.push_undo(&self.level_data.props);

                    let asset_path = format!("assets/{}", asset_id);
                    if let Ok((_v, _p, pp, pi)) = std::panic::catch_unwind(
                        std::panic::AssertUnwindSafe(|| load_model(&asset_path)),
                    ) {
                        self.physics.add_prop(&prop, &pp, &pi);
                    }

                    let prop_idx = self.level_data.props.len();
                    self.level_data.props.push(prop);
                    self.editor.selected_idx = Some(prop_idx);
                    self.sync_instances();
                    println!("[EDITOR] Place       : \"{}\"  → prop #{}  pos=({:.2},{:.2},{:.2})",
                        asset_id, prop_idx, place_pos.x, place_pos.y, place_pos.z);
                    println!("[EDITOR]               {} total props  |  collider={:?}",
                        self.level_data.props.len(),
                        self.editor.current_collider());
                    self.action_cooldown = 20;
                }
            }
        }

        // ── RMB: pick / select prop under crosshair ───────────────────────────
        if input.is_mouse_down(MouseButton::Right) && self.action_cooldown == 0 {
            if self.editor.is_grabbing {
                // Cancel grab — restore position via undo
                self.editor.cancel_grab();
                if let Some(restored) = self.editor.pop_undo() {
                    self.level_data.props = restored;
                    self.sync_instances();
                }
                self.action_cooldown = 15;
            } else {
                let ray_origin = self.camera.position;
                let ray_dir = self.camera.get_forward();
                let found = self.level_data.props.iter().enumerate().find(|(_, p)| {
                    ray_hits_sphere(
                        ray_origin,
                        ray_dir,
                        Vec3::new(p.position[0], p.position[1], p.position[2]),
                        1.5,
                    )
                });

                if let Some((i, prop)) = found {
                    let prop = prop.clone();
                    self.editor.select_prop(i, &prop);
                } else {
                    self.editor.deselect();
                }
                self.action_cooldown = 15;
            }
        }

        // ── Middle click: copy hovered prop's asset to hotbar (no select) ─────
        if input.is_mouse_down(MouseButton::Middle) && self.action_cooldown == 0 {
            let ray_origin = self.camera.position;
            let ray_dir = self.camera.get_forward();
            let found = self.level_data.props.iter().find(|p| {
                ray_hits_sphere(
                    ray_origin,
                    ray_dir,
                    Vec3::new(p.position[0], p.position[1], p.position[2]),
                    1.5,
                )
            });
            if let Some(prop) = found {
                self.editor.assign_to_active_slot(prop.asset_id.clone());
                self.action_cooldown = 10;
            }
        }
    }
}

//! Runtime ownership, construction, rendering, and transactional level commits.
//!
//! Per-frame behavior lives in `update`, instance assembly in `sync`, disk
//! asset discovery in `loader`, level preparation in `level_loader`, and HUD
//! display mapping in `hud_state`.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::core::engine::hud_state::hud_text;
use crate::core::engine::level_loader::PreparedLevel;
use crate::core::engine::loader::{load_prop_assets, load_textures_from_disk};
use crate::core::engine::validation::tuning_validation_errors;
use crate::data::config::gameplay::GameConfig;
use crate::data::enemy::EnemyRegistry;
use crate::data::relic::RelicRegistry;
use crate::data::world::level::{AtmosphereData, LevelData, LevelEventTriggerKind};
use crate::game::cycle::CycleState;
use crate::game::enemy::EnemyRuntimeState;
use crate::game::feedback::FeedbackState;
use crate::game::mountain::ActiveMountainReaction;
use crate::game::player::PlayerState;
use crate::game::progression::{ActiveAnchorRite, RunProgress};
use crate::game::relic::EquippedRelic;
use crate::game::save::{SaveData, DEFAULT_SAVE_PATH};
use crate::systems::audio::AudioSystem;
use crate::systems::physics::engine::PhysicsEngine;
use crate::systems::render::assets::{AssetManager, DrawGroup, RenderAssetMeshPart};
use crate::systems::render::camera::{Camera, CameraController, CameraUniform};
use crate::systems::render::hud::{
    DialogueHudState, HudFeedback, HudFrameState, HudSystem, PlayerHudState,
};
use crate::systems::render::lighting::LightingSystem;
use crate::systems::render::mesh::ModelData;
use crate::systems::render::particles::ParticleSystem;
use crate::systems::render::pipeline::RenderPipeline;
use crate::systems::render::texture::TextureManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Playing,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ManualLevelEventStatus {
    Ready,
    AlreadyFired,
    MissingFlag(String),
    MissingEvent,
    WrongTrigger(LevelEventTriggerKind),
}

const DIALOGUE_LINE_DURATION: f32 = 3.8;

#[derive(Debug, Clone)]
pub(super) struct ActiveDialogueState {
    speaker: String,
    lines: Vec<String>,
    line_index: usize,
    line_timer: f32,
}

impl ActiveDialogueState {
    pub(super) fn new(speaker: String, lines: Vec<String>) -> Option<Self> {
        let lines = lines
            .into_iter()
            .filter_map(|line| {
                let line = line.trim();
                (!line.is_empty()).then(|| line.to_string())
            })
            .collect::<Vec<_>>();
        (!lines.is_empty()).then_some(Self {
            speaker,
            lines,
            line_index: 0,
            line_timer: DIALOGUE_LINE_DURATION,
        })
    }

    pub(super) fn tick(&mut self, dt: f32) -> bool {
        self.line_timer = (self.line_timer - dt.max(0.0)).max(0.0);
        if self.line_timer > 0.0 {
            return false;
        }

        self.advance()
    }

    pub(super) fn advance(&mut self) -> bool {
        self.line_index += 1;
        if self.line_index >= self.lines.len() {
            return true;
        }
        self.line_timer = DIALOGUE_LINE_DURATION;
        false
    }

    pub(super) fn hud_state(&self) -> DialogueHudState {
        DialogueHudState {
            speaker: self.speaker.clone(),
            line: self.lines[self.line_index].clone(),
            remaining_ratio: (self.line_timer / DIALOGUE_LINE_DURATION).clamp(0.0, 1.0),
        }
    }
}

pub struct EngineState {
    // Window / GPU surface
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // Render pipeline
    pub render_pipeline: wgpu::RenderPipeline,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    // Map geometry
    pub map_vertex_buffer: wgpu::Buffer,
    pub map_parts: Vec<RenderAssetMeshPart>,
    pub map_instance_buffer: wgpu::Buffer,
    pub map_texture_override: Option<String>,

    // Asset / texture registries
    pub assets: AssetManager,
    pub texture_manager: TextureManager,
    pub(super) texture_bind_group_layout: wgpu::BindGroupLayout,
    pub active_draw_groups: Vec<DrawGroup>,

    // Camera
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    // Lighting system
    pub lighting: LightingSystem,
    pub particles: ParticleSystem,

    // HUD system
    pub hud: HudSystem,
    pub audio: Option<AudioSystem>,
    pub game_mode: GameMode,

    // Subsystems
    pub physics: PhysicsEngine,

    // Game data
    pub config_data: GameConfig,
    pub enemy_registry: EnemyRegistry,
    pub relic_registry: RelicRegistry,
    pub runtime_atmosphere: AtmosphereData,
    pub level_data: LevelData,
    pub level_name: String,
    pub player: PlayerState,
    pub progress: RunProgress,
    pub equipped_relic: EquippedRelic,
    pub cycle: CycleState,
    pub feedback: FeedbackState,
    pub(super) active_dialogue: Option<ActiveDialogueState>,
    pub(super) active_anchor_rite: Option<ActiveAnchorRite>,
    pub(super) mountain_reaction: Option<ActiveMountainReaction>,
    pub(super) queued_mountain_reactions: VecDeque<String>,
    pub frame_time_ms: f32,
    pub debug_hud_enabled: bool,

    /// Time remaining (seconds) before the next action is allowed.
    pub action_cooldown: f32,
    /// Per-prop enemy runtime state, aligned with `level_data.props`.
    pub enemy_runtime: Vec<EnemyRuntimeState>,
    /// Per-event fired flags, aligned with `level_data.events`.
    pub level_event_fired: Vec<bool>,
    /// Level-local event flags set by event actions during the current load.
    pub level_flags: HashSet<String>,
    /// Manual event requests waiting for the next gameplay event pass.
    pub(crate) queued_manual_level_events: HashSet<String>,
    /// Stable IDs for authored props consumed or defeated in the current level.
    pub(crate) removed_prop_ids: HashSet<String>,
    /// Accumulator for debug logging (seconds since last print).
    pub debug_timer: f32,

    /// If Some, the engine should load this level on the next frame.
    pub pending_transition: Option<String>,
    /// A failed target is suppressed until the current level is reloaded.
    pub failed_transition: Option<String>,
}

impl EngineState {
    pub async fn new(window: Arc<Window>, level_name: String) -> Result<Self, String> {
        let config_data = GameConfig::try_load()
            .map_err(|error| format!("game configuration could not be loaded: {}", error))?;
        let tuning_errors = tuning_validation_errors(&config_data);
        if !tuning_errors.is_empty() {
            return Err(format!(
                "gameplay tuning failed validation: {}",
                tuning_errors.join("; ")
            ));
        }
        let enemy_registry = EnemyRegistry::try_load_dir("data/enemies")
            .map_err(|error| format!("enemy definitions could not be loaded: {}", error))?;
        let relic_registry = RelicRegistry::try_load_dir("data/relics")
            .map_err(|error| format!("relic definitions could not be loaded: {}", error))?;
        let should_continue_saved_game = level_name.eq_ignore_ascii_case("continue");
        let saved_game = if should_continue_saved_game {
            SaveData::load_from_path(DEFAULT_SAVE_PATH)
                .map_err(|error| format!("saved game could not be resumed: {}", error))?
        } else {
            None
        };
        let starting_level_name = saved_game
            .as_ref()
            .map(|save| save.level_name.clone())
            .unwrap_or_else(|| {
                if should_continue_saved_game {
                    "movement_test".to_string()
                } else {
                    level_name
                }
            });
        let mut prepared_level =
            Self::prepare_level(&starting_level_name, &enemy_registry, &relic_registry)?;
        let mut restored_removed_prop_ids = HashSet::new();
        let mut restored_progress = None;
        let mut save_cleanup_needed = false;
        if let Some(save) = saved_game.as_ref() {
            let (removed_prop_ids, world_cleanup_needed) =
                Self::apply_saved_world_state(&mut prepared_level.data, save, &relic_registry);
            restored_removed_prop_ids = removed_prop_ids;
            save_cleanup_needed |= world_cleanup_needed;
            let (progress, progress_cleanup_needed) =
                Self::progress_for_saved_level(&prepared_level.data, save);
            if let Some(respawn_position) = progress.respawn_position {
                prepared_level.data.player_spawn = respawn_position;
            }
            restored_progress = Some(progress);
            save_cleanup_needed |= progress_cleanup_needed;
        }
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance
            .create_surface(window.clone())
            .map_err(|error| format!("failed to create rendering surface: {}", error))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .map_err(|error| format!("failed to find a compatible graphics adapter: {}", error))?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                ..Default::default()
            })
            .await
            .map_err(|error| format!("failed to create graphics device: {}", error))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or_else(|| "graphics surface reported no supported formats".to_string())?;
        let present_mode = surface_caps
            .present_modes
            .iter()
            .copied()
            .find(|mode| *mode == wgpu::PresentMode::Fifo)
            .or_else(|| surface_caps.present_modes.first().copied())
            .ok_or_else(|| "graphics surface reported no presentation modes".to_string())?;
        let alpha_mode = surface_caps
            .alpha_modes
            .first()
            .copied()
            .ok_or_else(|| "graphics surface reported no alpha modes".to_string())?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let mut texture_manager = TextureManager::new(&device, &queue, &texture_bind_group_layout);
        let texture_count = load_textures_from_disk(
            &device,
            &queue,
            &texture_bind_group_layout,
            &mut texture_manager,
        )
        .into_result("texture")?;
        println!("[ASSETS] Loaded {} texture(s)", texture_count);

        let PreparedLevel {
            name: prepared_level_name,
            path: level_path,
            data: level_data,
            model,
        } = prepared_level;
        println!(
            "[LEVEL] Prepared '{}': {} props",
            level_path,
            level_data.props.len()
        );

        let ModelData {
            vertices: map_vertices,
            parts: map_mesh_parts,
            physics_vertices,
            physics_triangles,
        } = model;
        Self::log_map_model(&map_vertices, map_mesh_parts.len());
        let map_resources = Self::build_map_resources(
            &device,
            &map_vertices,
            map_mesh_parts,
            &level_data.base_material,
        );

        let mut assets = AssetManager::new();
        let asset_count = load_prop_assets(&device, &mut assets).into_result("model asset")?;

        println!("[ASSETS] Loaded {} model asset(s)", asset_count);
        let camera = Self::camera_for_spawn(
            level_data.player_spawn,
            config.width,
            config.height,
            config_data.world.draw_distance,
        );
        let camera_controller = CameraController::new(config_data.camera.sensitivity);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, 0.0);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let lighting = LightingSystem::new(&device, &queue);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../systems/render/shader.wgsl").into(),
            ),
        });
        let render_pipeline = RenderPipeline::new(
            &device,
            &config,
            &camera_bind_group_layout,
            &texture_bind_group_layout,
            lighting.get_light_bind_group_layout(),
            lighting.get_fog_bind_group_layout(),
            &shader,
        )
        .pipeline;
        let mut particles = ParticleSystem::new(&device, &config, &camera_bind_group_layout);
        particles.configure(&queue, &level_data.atmosphere, camera.position);

        let (depth_texture, depth_view) =
            Self::create_depth_resources(&device, config.width, config.height);
        let physics = Self::build_physics_for_level(
            &level_data,
            &config_data.physics,
            physics_vertices,
            physics_triangles,
        );

        let hud = HudSystem::new(&device, config.format, &config_data.ui);
        let mut audio = AudioSystem::new();
        if let Some(audio_system) = audio.as_mut() {
            audio_system.set_ambience(
                level_data.atmosphere.ambience_preset,
                level_data.atmosphere.ambience_volume,
            );
        }

        let player = PlayerState::new(&config_data.player);
        let progress = restored_progress.unwrap_or_else(RunProgress::new);
        let mut equipped_relic = EquippedRelic::new();
        let mut cycle = CycleState::default();
        if let Some(save) = saved_game.as_ref() {
            equipped_relic.restore_from_ids(
                &save.relic_inventory,
                save.equipped_relic_id.as_deref(),
                &relic_registry,
            );
            save_cleanup_needed |= equipped_relic.owned_count() != save.relic_inventory.len();
            cycle = CycleState::new(save.cycle_number);
            println!(
                "[SAVE] Loaded '{}': cycle {}, {} banked resource, {} relic(s)",
                save.level_name,
                cycle.number,
                progress.banked_resource,
                equipped_relic.owned_count()
            );
        }
        let feedback = FeedbackState::new();
        let enemy_runtime = Self::enemy_runtime_for_level(&level_data);
        let (level_event_fired, event_cleanup_needed) =
            Self::level_event_runtime_for_saved_level(&level_data, saved_game.as_ref());
        save_cleanup_needed |= event_cleanup_needed;
        let (level_flags, flag_cleanup_needed) =
            Self::level_flags_for_saved_level(&level_data, saved_game.as_ref());
        save_cleanup_needed |= flag_cleanup_needed;
        let removed_prop_ids = restored_removed_prop_ids;
        let (queued_mountain_reactions, reaction_cleanup_needed) =
            Self::mountain_reactions_for_saved_level(&level_data, saved_game.as_ref());
        save_cleanup_needed |= reaction_cleanup_needed;
        let mut state = Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            depth_texture,
            depth_view,
            map_vertex_buffer: map_resources.vertex_buffer,
            map_parts: map_resources.parts,
            map_instance_buffer: map_resources.instance_buffer,
            map_texture_override: map_resources.texture_override,
            assets,
            texture_manager,
            texture_bind_group_layout,
            active_draw_groups: Vec::new(),
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            hud,
            audio,
            game_mode: GameMode::Playing,
            lighting,
            particles,
            physics,
            config_data,
            enemy_registry,
            relic_registry,
            runtime_atmosphere: level_data.atmosphere.clone(),
            level_data,
            level_name: prepared_level_name,
            player,
            progress,
            equipped_relic,
            cycle,
            feedback,
            active_dialogue: None,
            active_anchor_rite: None,
            mountain_reaction: None,
            queued_mountain_reactions,
            frame_time_ms: 0.0,
            debug_hud_enabled: false,
            action_cooldown: 0.0,
            enemy_runtime,
            level_event_fired,
            level_flags,
            queued_manual_level_events: HashSet::new(),
            removed_prop_ids,
            debug_timer: 0.0,
            pending_transition: None,
            failed_transition: None,
        };
        state.feedback.on_level_enter();

        state.sync_instances();
        if save_cleanup_needed {
            state.autosave("content compatibility cleanup");
        }
        Ok(state)
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        let (depth_texture, depth_view) =
            Self::create_depth_resources(&self.device, self.config.width, self.config.height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    pub fn record_frame_time(&mut self, dt: f32) {
        if !dt.is_finite() || dt <= 0.0 {
            return;
        }
        let sample_ms = (dt * 1000.0).min(250.0);
        if self.frame_time_ms <= 0.0 {
            self.frame_time_ms = sample_ms;
        } else {
            self.frame_time_ms += (sample_ms - self.frame_time_ms) * 0.10;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.runtime_atmosphere.clear_color[0] as f64,
                            g: self.runtime_atmosphere.clear_color[1] as f64,
                            b: self.runtime_atmosphere.clear_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rp.set_pipeline(&self.render_pipeline);
            rp.set_bind_group(0, &self.camera_bind_group, &[]);
            rp.set_bind_group(2, &self.lighting.light_bind_group, &[]);
            rp.set_bind_group(3, &self.lighting.fog_bind_group, &[]);

            rp.set_vertex_buffer(0, self.map_vertex_buffer.slice(..));
            rp.set_vertex_buffer(1, self.map_instance_buffer.slice(..));
            for part in &self.map_parts {
                let texture_name = self
                    .map_texture_override
                    .as_deref()
                    .unwrap_or(&part.texture_name);
                let bg = self.texture_manager.get(texture_name);
                rp.set_bind_group(1, bg, &[]);
                rp.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..part.num_indices, 0, 0..1);
            }

            for group in &self.active_draw_groups {
                if let Some(asset) = self.assets.get(&group.asset_id) {
                    rp.set_vertex_buffer(0, asset.vertex_buffer.slice(..));
                    rp.set_vertex_buffer(1, group.instance_buffer.slice(..));
                    for part in &asset.parts {
                        let texture_name = group
                            .texture_override
                            .as_deref()
                            .unwrap_or(&part.texture_name);
                        let bg = self.texture_manager.get(texture_name);
                        rp.set_bind_group(1, bg, &[]);
                        rp.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rp.draw_indexed(0..part.num_indices, 0, 0..group.num_instances);
                    }
                }
            }

            self.particles.draw(&mut rp, &self.camera_bind_group);
        }

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("HUD Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            let health_ratio = self.player.health.ratio();
            let health_trail_ratio = self.player.health.trail_ratio();
            let stamina_ratio = self.player.stamina.display_ratio();
            let hud_feedback = HudFeedback {
                shot_flash: self.feedback.shot_flash_timer,
                hit_marker: self.feedback.hit_marker_timer,
                kill_marker: self.feedback.kill_marker_timer,
                blocked_flash: self.feedback.blocked_flash_timer,
                miss_flash: self.feedback.miss_flash_timer,
                pickup_flash: self.feedback.pickup_flash_timer,
                damage_flash: self.feedback.damage_flash_timer,
                debug_flash: self.feedback.debug_flash_timer,
                spawn_flash: self.feedback.spawn_flash_timer,
                reload_flash: self.feedback.reload_flash_timer,
                loot_flash: self.feedback.loot_flash_timer,
                heal_flash: self.feedback.heal_flash_timer,
            };

            let hud_state = HudFrameState {
                viewport_size: [self.config.width, self.config.height],
                player: PlayerHudState {
                    health_ratio,
                    health_trail_ratio,
                    stamina_ratio,
                    dash_cooldown_ratio: if self.config_data.movement.dash_cooldown > 0.0 {
                        (self.player.dash_cooldown_timer / self.config_data.movement.dash_cooldown)
                            .clamp(0.0, 1.0)
                    } else {
                        0.0
                    },
                    health_current: self.player.health.current.max(0.0).round() as u32,
                    health_max: self.player.health.max.max(0.0).round() as u32,
                    stamina_current: self.player.stamina.current.max(0.0).round() as u32,
                    stamina_max: self.player.stamina.max.max(0.0).round() as u32,
                },
                hit_flash: self.player.hit_flash_timer,
                paused: self.game_mode == GameMode::Paused,
                dead: self.player.is_dead,
                respawn_remaining: self.player.respawn_timer.max(0.0),
                time: self.feedback.time,
                feedback: hud_feedback,
                debug: self.debug_hud_state(),
                ascent: self.ascent_hud_state(),
                markers: self.hud_world_markers(),
                event_feed: self.hud_event_feed(),
                interaction_prompt: self.interaction_prompt(),
                dialogue: self.dialogue_hud_state(),
                anchor_rite: self.anchor_rite_hud_state(),
                named_notice: self.named_notice_hud_state(),
                named_encounter: self.named_encounter_hud_state(),
                level_arrival_ratio: self.feedback.level_arrival_ratio(),
                level_title: hud_text(&self.level_data.name),
                level_subtitle: format!(
                    "CYCLE {} / {}",
                    self.cycle.number,
                    hud_text(self.cycle.modifier.display_label())
                ),
            };

            self.hud.draw(&mut rp, &self.queue, hud_state);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_filters_blank_lines_and_can_be_advanced() {
        let mut dialogue = ActiveDialogueState::new(
            "Waystone".to_string(),
            vec![
                " First line. ".to_string(),
                "  ".to_string(),
                "Second line.".to_string(),
            ],
        )
        .unwrap();

        assert_eq!(dialogue.hud_state().line, "First line.");
        assert!(!dialogue.advance());
        assert_eq!(dialogue.hud_state().line, "Second line.");
        assert!(dialogue.advance());
    }

    #[test]
    fn dialogue_advances_when_its_line_timer_expires() {
        let mut dialogue = ActiveDialogueState::new(
            "Waystone".to_string(),
            vec!["First".to_string(), "Second".to_string()],
        )
        .unwrap();

        assert!(!dialogue.tick(DIALOGUE_LINE_DURATION));
        assert_eq!(dialogue.hud_state().line, "Second");
        assert!(dialogue.tick(DIALOGUE_LINE_DURATION));
    }
}
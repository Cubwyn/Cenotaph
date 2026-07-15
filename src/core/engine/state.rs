// src/core/engine/state.rs
// EngineState owns every runtime subsystem and the GPU surface.
//
// Responsibilities:
//   - Struct definition (all fields)
//   - Construction (new)
//   - Resize
//   - Render (GPU draw calls)
//
// Per-frame logic lives in:
//   update.rs - update_physics / update_visuals / gameplay
//   sync.rs   - sync_instances
//   loader.rs - load_textures_from_disk / load_prop_assets

use std::collections::{HashSet, VecDeque};
use std::path::Path;
use std::sync::Arc;

use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::core::engine::loader::{load_prop_assets, load_textures_from_disk};
use crate::core::engine::validation::{tuning_validation_errors, validate_model_geometry};
use crate::data::config::gameplay::{GameConfig, PhysicsConfig};
use crate::data::enemy::EnemyRegistry;
use crate::data::relic::RelicRegistry;
use crate::data::world::level::{
    validate_level_id, AtmosphereData, LevelData, LevelEventActionKind, LevelEventData,
    LevelEventTriggerKind, PropData, BASE_MAP_Y_OFFSET,
};
use crate::game::cycle::CycleState;
use crate::game::enemy::EnemyRuntimeState;
use crate::game::feedback::{FeedbackEvent, FeedbackEventKind, FeedbackState};
use crate::game::mountain::ActiveMountainReaction;
use crate::game::player::PlayerState;
use crate::game::progression::{ActiveAnchorRite, RunProgress};
use crate::game::relic::EquippedRelic;
use crate::game::save::{SaveData, DEFAULT_SAVE_PATH};
use crate::systems::audio::AudioSystem;
use crate::systems::physics::engine::PhysicsEngine;
use crate::systems::render::assets::{AssetManager, DrawGroup, RenderAssetMeshPart};
use crate::systems::render::camera::{Camera, CameraController, CameraUniform, BASE_FOVY};
use crate::systems::render::hud::{
    AnchorRiteHudState, AscentHudState, DebugHudState, DialogueHudState, HudFeedEvent, HudFeedback,
    HudFrameState, HudMarkerKind, HudMarkerState, HudSystem, HudWorldMarker,
    NamedEncounterHudState, NamedNoticeHudState, PlayerHudState,
};
use crate::systems::render::instance::InstanceRaw;
use crate::systems::render::lighting::LightingSystem;
use crate::systems::render::mesh::{try_load_model, ModelData, RenderMeshPart, Vertex};
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

struct MapRenderResources {
    vertex_buffer: wgpu::Buffer,
    parts: Vec<RenderAssetMeshPart>,
    instance_buffer: wgpu::Buffer,
    texture_override: Option<String>,
}

struct PreparedLevel {
    name: String,
    path: String,
    data: LevelData,
    model: ModelData,
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

    fn hud_state(&self) -> DialogueHudState {
        DialogueHudState {
            speaker: self.speaker.clone(),
            line: self.lines[self.line_index].clone(),
            remaining_ratio: (self.line_timer / DIALOGUE_LINE_DURATION).clamp(0.0, 1.0),
        }
    }
}

fn round_to_u32(value: f32) -> u32 {
    value.max(0.0).round().min(u32::MAX as f32) as u32
}

fn hud_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '/') {
                character.to_ascii_uppercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
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
    texture_bind_group_layout: wgpu::BindGroupLayout,
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

        let (map_vertices, map_mesh_parts, phys_points, phys_indices) = model;
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
            phys_points,
            phys_indices,
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

    fn debug_hud_state(&self) -> DebugHudState {
        let enemies = self
            .level_data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some() && prop.enemy_health > 0.0)
            .count() as u32;
        let loot = self
            .level_data
            .props
            .iter()
            .filter(|prop| {
                prop.resource_value > 0
                    || prop
                        .item_id
                        .as_ref()
                        .is_some_and(|item_id| !item_id.trim().is_empty())
            })
            .count() as u32;

        DebugHudState {
            enabled: self.debug_hud_enabled,
            enemies,
            loot,
            unsecured_resource: self.progress.unsecured_resource,
            banked_resource: self.progress.banked_resource,
            cycle: self.cycle.number,
            props: self.level_data.props.len() as u32,
            fps: if self.frame_time_ms > 0.0 {
                round_to_u32(1000.0 / self.frame_time_ms)
            } else {
                0
            },
            frame_ms: round_to_u32(self.frame_time_ms),
        }
    }

    fn ascent_hud_state(&self) -> AscentHudState {
        let current_relic = self.equipped_relic.current();

        AscentHudState {
            cycle: self.cycle.number,
            cycle_modifier: self.cycle.modifier.display_label().to_string(),
            relic_name: current_relic
                .map(|relic| relic.display_name.clone())
                .unwrap_or_else(|| "UNCLAIMED".to_string()),
            unsecured_resource: self.progress.unsecured_resource,
            banked_resource: self.progress.banked_resource,
        }
    }

    fn hud_event_feed(&self) -> Vec<HudFeedEvent> {
        self.feedback
            .events
            .iter()
            .filter(|event| event.is_active())
            .filter_map(Self::hud_event_for_feedback)
            .collect()
    }

    fn interaction_prompt(&self) -> String {
        if self.player.is_dead
            || self.game_mode != GameMode::Playing
            || self.active_anchor_rite.is_some()
        {
            return String::new();
        }

        let player = Vec3::from_array(self.physics.get_player_pos());
        if Self::nearest_interact_event_index(
            &self.level_data.events,
            &self.level_data.props,
            &self.level_event_fired,
            player,
            &self.level_flags,
        )
        .is_some()
        {
            return "INTERACT".to_string();
        }

        Self::nearest_anchor_prop_index(
            &self.level_data.props,
            player,
            self.config_data.world.anchor_interaction_radius,
        )
        .map(|_| "COMMUNE WITH ANCHOR".to_string())
        .unwrap_or_default()
    }

    fn anchor_rite_hud_state(&self) -> AnchorRiteHudState {
        let Some(rite) = self.active_anchor_rite.as_ref() else {
            return AnchorRiteHudState::default();
        };
        let mend_cost = self.config_data.world.anchor_mend_cost;
        let newly_activated =
            self.progress.active_anchor_id.as_deref() != Some(rite.anchor_id.as_str());
        let bind_event_status = newly_activated
            .then_some(rite.event_id.as_deref())
            .flatten()
            .map(|event_id| self.manual_level_event_status(event_id));
        let (can_bind, bind_requirement) = match bind_event_status {
            Some(ManualLevelEventStatus::MissingFlag(flag_id)) => (
                false,
                format!("REQUIRES {}", hud_text(&flag_id.replace('_', " "))),
            ),
            Some(
                ManualLevelEventStatus::MissingEvent | ManualLevelEventStatus::WrongTrigger(_),
            ) => (false, "RITE UNAVAILABLE".to_string()),
            Some(ManualLevelEventStatus::Ready | ManualLevelEventStatus::AlreadyFired) | None => {
                (true, String::new())
            }
        };

        AnchorRiteHudState {
            active: true,
            anchor_name: hud_text(&rite.display_name),
            selected_option: rite.selected_option,
            carried_ash: self.progress.unsecured_resource,
            bound_ash: self.progress.banked_resource,
            mend_cost,
            can_bind,
            bind_requirement,
            can_mend: self.progress.banked_resource >= mend_cost
                && self.player.health.current < self.player.health.max,
            vessel_wounded: self.player.health.current < self.player.health.max,
        }
    }

    fn dialogue_hud_state(&self) -> DialogueHudState {
        self.active_dialogue
            .as_ref()
            .map(ActiveDialogueState::hud_state)
            .unwrap_or_default()
    }

    fn named_notice_hud_state(&self) -> NamedNoticeHudState {
        self.feedback
            .named_notice
            .as_ref()
            .map(|notice| NamedNoticeHudState {
                active: true,
                title: hud_text(&notice.title),
                subtitle: hud_text(&notice.subtitle),
                remaining_ratio: notice.remaining_ratio(),
            })
            .unwrap_or_default()
    }

    fn named_encounter_hud_state(&self) -> NamedEncounterHudState {
        let player = Vec3::from_array(self.physics.get_player_pos());
        self.level_data
            .props
            .iter()
            .enumerate()
            .filter_map(|(index, prop)| {
                if prop.enemy_type.is_none() || prop.enemy_health <= 0.0 {
                    return None;
                }
                let display_name = prop
                    .display_name
                    .as_deref()
                    .filter(|name| !name.trim().is_empty())?;
                let (state, _) =
                    self.marker_state_for_prop(index, prop, HudMarkerKind::Enemy, player);
                if state == HudMarkerState::Neutral {
                    return None;
                }
                let distance = player.distance(Vec3::from_array(prop.position));
                let health_ratio = self.marker_ratio_for_prop(index, prop, HudMarkerKind::Enemy);
                Some((distance, display_name, health_ratio))
            })
            .min_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, display_name, health_ratio)| NamedEncounterHudState {
                active: true,
                name: hud_text(display_name),
                health_ratio,
            })
            .unwrap_or_default()
    }

    fn hud_event_for_feedback(event: &FeedbackEvent) -> Option<HudFeedEvent> {
        let (label, color) = match event.kind {
            FeedbackEventKind::PlayerDamage => ("DMG", [1.0, 0.16, 0.08, 1.0]),
            FeedbackEventKind::EnemyHit => ("HIT", [1.0, 0.38, 0.14, 1.0]),
            FeedbackEventKind::EnemyKill => ("KILL", [1.0, 0.74, 0.18, 1.0]),
            FeedbackEventKind::ShotBlocked => ("BLOCK", [0.48, 0.78, 1.0, 1.0]),
            FeedbackEventKind::ShotMissed => ("MISS", [0.76, 0.80, 0.86, 1.0]),
            FeedbackEventKind::Pickup => ("PICKUP", [1.0, 0.78, 0.18, 1.0]),
            FeedbackEventKind::Resource => ("RES", [0.84, 0.82, 1.0, 1.0]),
            FeedbackEventKind::Heal => ("HEAL", [0.28, 1.0, 0.48, 1.0]),
            FeedbackEventKind::Spawn => ("SPAWN", [0.36, 1.0, 0.48, 1.0]),
            FeedbackEventKind::Reload => ("RELOAD", [0.45, 0.75, 1.0, 1.0]),
            FeedbackEventKind::Loot => ("LOOT", [1.0, 0.80, 0.24, 1.0]),
            FeedbackEventKind::Relic => ("RELIC", [1.0, 0.80, 0.24, 1.0]),
            FeedbackEventKind::Debug => ("DEBUG", [0.2, 0.85, 1.0, 1.0]),
            FeedbackEventKind::Death => ("DEATH", [1.0, 0.12, 0.08, 1.0]),
            FeedbackEventKind::None => return None,
        };

        Some(HudFeedEvent {
            label,
            value: event.value,
            has_value: event.value > 0,
            ratio: event.remaining_ratio(),
            color,
        })
    }

    fn hud_world_markers(&self) -> Vec<HudWorldMarker> {
        let player_pos = self.physics.get_player_pos();
        let player = Vec3::new(player_pos[0], player_pos[1], player_pos[2]);
        let mut markers = Vec::new();

        for (index, prop) in self.level_data.props.iter().enumerate() {
            let Some(kind) = self.marker_kind_for_prop(prop) else {
                continue;
            };

            let marker_height = prop.scale[1].abs().max(1.0) * 0.75 + 0.65;
            let world_pos = Vec3::new(
                prop.position[0],
                prop.position[1] + marker_height,
                prop.position[2],
            );
            let Some(screen_pos) = self.project_to_hud(world_pos) else {
                continue;
            };
            let distance = player.distance(world_pos);
            let (state, state_ratio) = self.marker_state_for_prop(index, prop, kind, player);
            if kind == HudMarkerKind::Enemy && state == HudMarkerState::Neutral {
                continue;
            }
            markers.push((
                distance,
                HudWorldMarker {
                    screen_pos,
                    ratio: self.marker_ratio_for_prop(index, prop, kind),
                    distance_m: round_to_u32(distance),
                    kind,
                    state,
                    state_ratio,
                },
            ));
        }

        markers.sort_by(|left, right| left.0.total_cmp(&right.0));
        markers
            .into_iter()
            .take(64)
            .map(|(_, marker)| marker)
            .collect()
    }

    fn marker_kind_for_prop(
        &self,
        prop: &crate::data::world::level::PropData,
    ) -> Option<HudMarkerKind> {
        if prop.enemy_type.is_some() && prop.enemy_health > 0.0 {
            Some(HudMarkerKind::Enemy)
        } else if prop.is_hurtbox {
            Some(HudMarkerKind::Hazard)
        } else if prop.resource_value > 0
            || prop
                .item_id
                .as_ref()
                .is_some_and(|item_id| !item_id.trim().is_empty())
        {
            Some(HudMarkerKind::Loot)
        } else if prop
            .anchor_id
            .as_ref()
            .is_some_and(|anchor_id| !anchor_id.trim().is_empty())
        {
            Some(HudMarkerKind::Anchor)
        } else {
            None
        }
    }

    fn marker_state_for_prop(
        &self,
        index: usize,
        prop: &crate::data::world::level::PropData,
        kind: HudMarkerKind,
        player: Vec3,
    ) -> (HudMarkerState, f32) {
        if kind != HudMarkerKind::Enemy {
            return (HudMarkerState::Neutral, 0.0);
        }

        let Some(enemy_type) = prop.enemy_type.as_deref() else {
            return (HudMarkerState::Neutral, 0.0);
        };
        let Some(enemy) = self.enemy_registry.get(enemy_type) else {
            return (HudMarkerState::Neutral, 0.0);
        };

        if let Some(runtime) = self.enemy_runtime.get(index) {
            if runtime.stagger_remaining > 0.0 {
                let duration = self.config_data.combat.enemy_hit_stun.max(0.001);
                return (
                    HudMarkerState::Staggered,
                    (runtime.stagger_remaining / duration).clamp(0.0, 1.0),
                );
            }

            if runtime.attack_windup_remaining > 0.0 {
                let windup = enemy.attack_windup.max(0.001);
                return (
                    HudMarkerState::Windup,
                    (1.0 - runtime.attack_windup_remaining / windup).clamp(0.0, 1.0),
                );
            }
        }

        let enemy_pos = Vec3::new(prop.position[0], prop.position[1], prop.position[2]);
        let horizontal_delta = Vec3::new(player.x - enemy_pos.x, 0.0, player.z - enemy_pos.z);
        let activation_range = enemy.activation_range.max(0.001);
        let distance_ratio = (horizontal_delta.length() / activation_range).clamp(0.0, 1.0);

        if distance_ratio < 1.0 {
            (HudMarkerState::Aggro, 1.0 - distance_ratio)
        } else {
            (HudMarkerState::Neutral, 0.0)
        }
    }

    fn marker_ratio_for_prop(
        &self,
        index: usize,
        prop: &crate::data::world::level::PropData,
        kind: HudMarkerKind,
    ) -> f32 {
        match kind {
            HudMarkerKind::Enemy => {
                let fallback_max_health = prop
                    .enemy_type
                    .as_deref()
                    .and_then(|enemy_type| self.enemy_registry.get(enemy_type))
                    .map_or(1.0, |enemy| enemy.health);
                self.enemy_runtime.get(index).map_or_else(
                    || (prop.enemy_health / fallback_max_health.max(1.0)).clamp(0.0, 1.0),
                    |runtime| runtime.health_ratio(prop.enemy_health, fallback_max_health),
                )
            }
            _ => 1.0,
        }
    }

    fn project_to_hud(&self, world_pos: Vec3) -> Option<[f32; 2]> {
        let clip = self.camera.build_view_projection_matrix() * world_pos.extend(1.0);
        if clip.w <= 0.01 {
            return None;
        }

        let ndc = clip.truncate() / clip.w;
        if ndc.x.abs() > 1.2 || ndc.y.abs() > 1.2 {
            return None;
        }

        Some([ndc.x.clamp(-0.97, 0.97), ndc.y.clamp(-0.92, 0.92)])
    }

    /// Reloads designer-owned runtime data as one transaction. Nothing live is
    /// changed unless config, registries, level data, and map geometry all pass.
    pub fn reload_runtime_content(&mut self) -> Result<(), String> {
        let config = GameConfig::try_load()
            .map_err(|error| format!("game configuration could not be reloaded: {}", error))?;
        let tuning_errors = tuning_validation_errors(&config);
        if !tuning_errors.is_empty() {
            return Err(format!(
                "gameplay tuning failed validation: {}",
                tuning_errors.join("; ")
            ));
        }

        let enemy_registry = EnemyRegistry::try_load_dir("data/enemies")
            .map_err(|error| format!("enemy definitions could not be reloaded: {}", error))?;
        let relic_registry = RelicRegistry::try_load_dir("data/relics")
            .map_err(|error| format!("relic definitions could not be reloaded: {}", error))?;
        let prepared = Self::prepare_level(&self.level_name, &enemy_registry, &relic_registry)?;

        let owned_relic_ids = self.equipped_relic.owned_ids();
        let equipped_relic_id = self.equipped_relic.equipped_id().map(ToOwned::to_owned);
        let missing_owned_relics = owned_relic_ids
            .iter()
            .filter(|id| relic_registry.get(id).is_none())
            .cloned()
            .collect::<Vec<_>>();
        if !missing_owned_relics.is_empty() {
            return Err(format!(
                "relic reload would remove owned relic(s): {}",
                missing_owned_relics.join(", ")
            ));
        }

        let mut assets = AssetManager::new();
        let asset_count =
            load_prop_assets(&self.device, &mut assets).into_result("model asset reload")?;
        let mut texture_manager =
            TextureManager::new(&self.device, &self.queue, &self.texture_bind_group_layout);
        let texture_count = load_textures_from_disk(
            &self.device,
            &self.queue,
            &self.texture_bind_group_layout,
            &mut texture_manager,
        )
        .into_result("texture reload")?;

        self.player.reconfigure(&config.player);
        self.camera_controller
            .set_sensitivity(config.camera.sensitivity);
        self.camera.zfar = config.world.draw_distance;
        self.config_data = config;
        self.hud.set_ui_config(&self.config_data.ui);
        self.enemy_registry = enemy_registry;
        self.relic_registry = relic_registry;
        self.assets = assets;
        self.texture_manager = texture_manager;
        self.equipped_relic.restore_from_ids(
            &owned_relic_ids,
            equipped_relic_id.as_deref(),
            &self.relic_registry,
        );
        self.commit_prepared_level(prepared);
        println!(
            "[RELOAD] Runtime data applied: {} models, {} textures, {} enemies, {} relics",
            asset_count,
            texture_count,
            self.enemy_registry.len(),
            self.relic_registry.len()
        );
        Ok(())
    }

    /// Fully prepares a new level before replacing any live runtime state.
    pub fn load_level(&mut self, new_level_name: &str) -> Result<(), String> {
        let prepared =
            Self::prepare_level(new_level_name, &self.enemy_registry, &self.relic_registry)?;
        self.commit_prepared_level(prepared);
        Ok(())
    }

    fn commit_prepared_level(&mut self, prepared: PreparedLevel) {
        let PreparedLevel {
            name,
            path,
            data: level_data,
            model,
        } = prepared;
        let (map_vertices, map_mesh_parts, phys_points, phys_indices) = model;
        let map_resources = Self::build_map_resources(
            &self.device,
            &map_vertices,
            map_mesh_parts,
            &level_data.base_material,
        );
        let physics = Self::build_physics_for_level(
            &level_data,
            &self.config_data.physics,
            phys_points,
            phys_indices,
        );
        let enemy_runtime = Self::enemy_runtime_for_level(&level_data);
        let level_event_fired = Self::level_event_runtime_for_level(&level_data);

        self.physics = physics;
        self.camera.position = Self::camera_position_for_spawn(level_data.player_spawn);
        self.map_vertex_buffer = map_resources.vertex_buffer;
        self.map_parts = map_resources.parts;
        self.map_instance_buffer = map_resources.instance_buffer;
        self.map_texture_override = map_resources.texture_override;
        self.level_data = level_data;
        self.runtime_atmosphere = self.level_data.atmosphere.clone();
        self.level_name = name;
        self.enemy_runtime = enemy_runtime;
        self.level_event_fired = level_event_fired;
        self.level_flags.clear();
        self.queued_manual_level_events.clear();
        self.active_dialogue = None;
        self.active_anchor_rite = None;
        self.mountain_reaction = None;
        self.queued_mountain_reactions.clear();
        self.removed_prop_ids.clear();
        self.failed_transition = None;
        self.particles
            .configure(&self.queue, &self.runtime_atmosphere, self.camera.position);
        if let Some(audio) = self.audio.as_mut() {
            audio.set_ambience(
                self.runtime_atmosphere.ambience_preset,
                self.runtime_atmosphere.ambience_volume,
            );
        }

        self.action_cooldown = 0.0;
        self.progress.clear_anchor();
        self.player
            .reset_for_level_transition(&self.config_data.player);

        self.sync_instances();
        println!(
            "[LEVEL] Loaded '{}' from {} ({} props)",
            self.level_name,
            path,
            self.level_data.props.len()
        );
    }

    pub fn update_lighting(&mut self) {
        let light_pos = [
            self.camera.position.x + 0.5,
            self.camera.position.y + self.config_data.lighting.sun_position_offset,
            self.camera.position.z + 0.3,
        ];
        self.lighting.update_light(
            &self.queue,
            light_pos,
            self.runtime_atmosphere.key_light_color,
            self.runtime_atmosphere.key_light_intensity,
        );

        self.lighting.update_fog(
            &self.queue,
            self.runtime_atmosphere.fog_density,
            self.runtime_atmosphere.fog_color,
        );
    }

    fn build_map_resources(
        device: &wgpu::Device,
        vertices: &[Vertex],
        mesh_parts: Vec<RenderMeshPart>,
        material: &crate::data::world::level::SurfaceMaterialData,
    ) -> MapRenderResources {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let parts = mesh_parts
            .into_iter()
            .map(|part| {
                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Index Buffer"),
                    contents: bytemuck::cast_slice(&part.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                RenderAssetMeshPart {
                    index_buffer,
                    num_indices: part.indices.len() as u32,
                    texture_name: part.texture_name,
                }
            })
            .collect();

        let map_instance = [InstanceRaw {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, BASE_MAP_Y_OFFSET, 0.0, 1.0],
            ],
            tint: [
                0.72 * material.tint[0],
                0.74 * material.tint[1],
                0.78 * material.tint[2],
                1.0,
            ],
            material: [material.uv_scale, material.emissive, 0.0, 0.0],
        }];
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Instance Buffer"),
            contents: bytemuck::cast_slice(&map_instance),
            usage: wgpu::BufferUsages::VERTEX,
        });

        MapRenderResources {
            vertex_buffer,
            parts,
            instance_buffer,
            texture_override: material.texture.clone(),
        }
    }

    fn build_physics_for_level(
        level_data: &LevelData,
        config: &PhysicsConfig,
        phys_points: Vec<Vec3>,
        phys_indices: Vec<[u32; 3]>,
    ) -> PhysicsEngine {
        let mut physics =
            PhysicsEngine::new(level_data.player_spawn, phys_points, phys_indices, config);

        for prop in &level_data.props {
            if let Some((prop_points, prop_indices)) = Self::brush_physics_mesh(prop) {
                physics.add_prop(prop, &prop_points, &prop_indices);
                continue;
            }

            let asset_path = format!("assets/{}", prop.asset_id);
            match try_load_model(&asset_path) {
                Ok((_vertices, _parts, prop_points, prop_indices)) => {
                    physics.add_prop(prop, &prop_points, &prop_indices);
                }
                Err(error) => {
                    eprintln!(
                        "[ERROR] Failed to load prop model '{}': {}",
                        asset_path, error
                    );
                    physics.add_prop(prop, &[], &[]);
                }
            }
        }

        physics
    }

    fn brush_physics_mesh(prop: &PropData) -> Option<(Vec<Vec3>, Vec<[u32; 3]>)> {
        let geometry = prop.brush_geometry.as_ref()?;
        let points = geometry
            .vertices
            .iter()
            .map(|vertex| Vec3::from_array(*vertex))
            .collect();
        Some((points, geometry.faces.clone()))
    }

    fn camera_for_spawn(spawn: [f32; 3], width: u32, height: u32, draw_distance: f32) -> Camera {
        Camera {
            position: Self::camera_position_for_spawn(spawn),
            yaw: -1.5,
            pitch: 0.0,
            visual_yaw_offset: 0.0,
            visual_pitch_offset: 0.0,
            aspect: width as f32 / height as f32,
            fovy: BASE_FOVY,
            znear: 0.1,
            zfar: draw_distance,
        }
    }

    fn camera_position_for_spawn(spawn: [f32; 3]) -> Vec3 {
        Vec3::new(spawn[0], spawn[1] + BASE_MAP_Y_OFFSET + 1.0, spawn[2])
    }

    fn create_depth_resources(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        (depth_texture, depth_view)
    }

    fn log_map_model(vertices: &[Vertex], part_count: usize) {
        println!(
            "[DEBUG] Render: {} map vertices, {} mesh parts",
            vertices.len(),
            part_count
        );
    }

    fn prepare_level(
        level_name: &str,
        enemy_registry: &EnemyRegistry,
        relic_registry: &RelicRegistry,
    ) -> Result<PreparedLevel, String> {
        validate_level_id(level_name)?;
        let path = format!("levels/{}.json", level_name);
        let mut data = LevelData::try_load(&path)?;
        Self::apply_enemy_definitions(&mut data, enemy_registry, &path)?;

        let mut validation_errors = data.validation_errors();
        validation_errors.extend(Self::relic_reference_errors(&data, relic_registry));
        if !validation_errors.is_empty() {
            return Err(format!(
                "level '{}' failed validation: {}",
                path,
                validation_errors.join("; ")
            ));
        }

        let model = try_load_model(&data.base_map).map_err(|error| {
            format!(
                "failed to load base map '{}' for level '{}': {}",
                data.base_map, level_name, error
            )
        })?;
        let model_errors = validate_model_geometry(&model);
        if !model_errors.is_empty() {
            return Err(format!(
                "base map '{}' failed geometry validation: {}",
                data.base_map,
                model_errors.join("; ")
            ));
        }

        Ok(PreparedLevel {
            name: level_name.to_string(),
            path,
            data,
            model,
        })
    }

    fn apply_enemy_definitions(
        level_data: &mut LevelData,
        enemy_registry: &EnemyRegistry,
        level_path: &str,
    ) -> Result<(), String> {
        for (index, prop) in level_data.props.iter_mut().enumerate() {
            let Some(enemy_type) = prop.enemy_type.as_deref() else {
                continue;
            };

            let Some(enemy) = enemy_registry.get(enemy_type) else {
                return Err(format!(
                    "level '{}' prop {} references unknown enemy_type '{}'",
                    level_path, index, enemy_type
                ));
            };

            prop.asset_id = enemy.model_asset.clone();
            prop.collider_type = enemy.collider_type;
            if prop.enemy_health <= 0.0 {
                prop.enemy_health = enemy.health;
            }
        }
        Ok(())
    }

    fn relic_reference_errors(
        level_data: &LevelData,
        relic_registry: &RelicRegistry,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        for (index, prop) in level_data.props.iter().enumerate() {
            if let Some(item_id) = prop.item_id.as_deref() {
                if relic_registry.get(item_id).is_none() {
                    errors.push(format!(
                        "prop {} references unknown item_id '{}'",
                        index, item_id
                    ));
                }
            }
        }
        for (table_index, table) in level_data.loot_tables.iter().enumerate() {
            for (entry_index, entry) in table.entries.iter().enumerate() {
                if let Some(item_id) = entry.item_id.as_deref() {
                    if relic_registry.get(item_id).is_none() {
                        errors.push(format!(
                            "loot table {} entry {} references unknown item_id '{}'",
                            table_index, entry_index, item_id
                        ));
                    }
                }
            }
        }
        errors
    }

    fn enemy_runtime_for_level(level_data: &LevelData) -> Vec<EnemyRuntimeState> {
        level_data
            .props
            .iter()
            .map(|prop| {
                EnemyRuntimeState::for_max_health(
                    prop.enemy_type.as_ref().map_or(0.0, |_| prop.enemy_health),
                )
            })
            .collect()
    }

    fn level_event_runtime_for_level(level_data: &LevelData) -> Vec<bool> {
        vec![false; level_data.events.len()]
    }

    pub(super) fn manual_level_event_status(&self, event_id: &str) -> ManualLevelEventStatus {
        Self::manual_level_event_status_for(
            &self.level_data.events,
            &self.level_event_fired,
            &self.level_flags,
            event_id,
        )
    }

    fn manual_level_event_status_for(
        events: &[LevelEventData],
        fired: &[bool],
        flags: &HashSet<String>,
        event_id: &str,
    ) -> ManualLevelEventStatus {
        let Some((index, event)) = events
            .iter()
            .enumerate()
            .find(|(_, event)| event.id == event_id)
        else {
            return ManualLevelEventStatus::MissingEvent;
        };
        if event.trigger.kind != LevelEventTriggerKind::Manual {
            return ManualLevelEventStatus::WrongTrigger(event.trigger.kind);
        }
        if let Some(flag_id) = event.trigger.flag_id.as_deref() {
            if !flags.contains(flag_id) {
                return ManualLevelEventStatus::MissingFlag(flag_id.to_string());
            }
        }
        if event.once && fired.get(index).copied().unwrap_or(false) {
            return ManualLevelEventStatus::AlreadyFired;
        }
        ManualLevelEventStatus::Ready
    }

    fn level_event_runtime_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (Vec<bool>, bool) {
        let Some(save) = save else {
            return (Self::level_event_runtime_for_level(level_data), false);
        };

        let fired: HashSet<&str> = save.fired_level_events.iter().map(String::as_str).collect();
        let runtime = level_data
            .events
            .iter()
            .map(|event| event.once && fired.contains(event.id.as_str()))
            .collect::<Vec<_>>();
        let restored_count = runtime.iter().filter(|fired| **fired).count();
        (runtime, restored_count != save.fired_level_events.len())
    }

    fn level_flags_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (HashSet<String>, bool) {
        let Some(save) = save else {
            return (HashSet::new(), false);
        };
        let known_ids = level_data
            .events
            .iter()
            .flat_map(|event| {
                event
                    .trigger
                    .flag_id
                    .iter()
                    .chain(event.actions.iter().filter_map(|action| {
                        (action.kind == LevelEventActionKind::SetFlag)
                            .then_some(action.flag_id.as_ref())
                            .flatten()
                    }))
            })
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let flags = save
            .level_flags
            .iter()
            .filter(|flag_id| known_ids.contains(flag_id.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let cleanup_needed = flags.len() != save.level_flags.len();
        (flags, cleanup_needed)
    }

    fn progress_for_saved_level(level_data: &LevelData, save: &SaveData) -> (RunProgress, bool) {
        let mut progress = save.to_progress();
        match (
            save.active_anchor_id.as_deref(),
            save.respawn_position.as_ref(),
        ) {
            (None, None) => (progress, false),
            (Some(anchor_id), Some(saved_position)) => {
                let current_position = level_data
                    .props
                    .iter()
                    .find(|prop| prop.anchor_id.as_deref() == Some(anchor_id))
                    .map(|prop| prop.position);
                if let Some(current_position) = current_position {
                    let cleanup_needed = current_position != *saved_position;
                    progress.respawn_position = Some(current_position);
                    (progress, cleanup_needed)
                } else {
                    eprintln!("[SAVE] Ignoring obsolete active anchor '{}'", anchor_id);
                    progress.clear_anchor();
                    (progress, true)
                }
            }
            _ => {
                eprintln!("[SAVE] Ignoring incomplete active anchor state");
                progress.clear_anchor();
                (progress, true)
            }
        }
    }

    fn mountain_reactions_for_saved_level(
        level_data: &LevelData,
        save: Option<&SaveData>,
    ) -> (VecDeque<String>, bool) {
        let Some(save) = save else {
            return (VecDeque::new(), false);
        };
        let known_ids = level_data
            .mountain_reactions
            .iter()
            .map(|reaction| reaction.id.as_str())
            .collect::<HashSet<_>>();
        let queue = save
            .pending_mountain_reactions
            .iter()
            .filter_map(|reaction_id| {
                if known_ids.contains(reaction_id.as_str()) {
                    Some(reaction_id.clone())
                } else {
                    eprintln!(
                        "[SAVE] Ignoring obsolete mountain reaction '{}'",
                        reaction_id
                    );
                    None
                }
            })
            .collect::<VecDeque<_>>();
        let cleanup_needed = queue.len() != save.pending_mountain_reactions.len();
        (queue, cleanup_needed)
    }

    fn apply_saved_world_state(
        level_data: &mut LevelData,
        save: &SaveData,
        relic_registry: &RelicRegistry,
    ) -> (HashSet<String>, bool) {
        let removable_authored_ids = level_data
            .props
            .iter()
            .filter(|prop| {
                prop.enemy_type.is_some() || prop.item_id.is_some() || prop.resource_value > 0
            })
            .filter_map(|prop| prop.id.as_deref())
            .collect::<HashSet<_>>();
        let removed_prop_ids = save
            .removed_prop_ids
            .iter()
            .filter(|prop_id| removable_authored_ids.contains(prop_id.as_str()))
            .cloned()
            .collect::<HashSet<_>>();
        let mut cleanup_needed = removed_prop_ids.len() != save.removed_prop_ids.len();
        level_data.props.retain(|prop| {
            prop.id
                .as_deref()
                .is_none_or(|prop_id| !removed_prop_ids.contains(prop_id))
        });

        for loot in &save.runtime_loot {
            let already_present = level_data
                .props
                .iter()
                .any(|prop| prop.id.as_deref() == Some(loot.id.as_str()));
            if already_present {
                cleanup_needed = true;
                continue;
            }

            let mut prop = loot.to_prop();
            if let Some(item_id) = loot.item_id.as_deref() {
                let Some(relic) = relic_registry.get(item_id) else {
                    eprintln!(
                        "[SAVE] Ignoring runtime loot '{}' for removed relic '{}'",
                        loot.id, item_id
                    );
                    cleanup_needed = true;
                    continue;
                };
                cleanup_needed |= prop.asset_id != relic.pickup_asset;
                prop.asset_id = relic.pickup_asset.clone();
            } else if !Path::new("assets").join(&loot.asset_id).is_file() {
                eprintln!(
                    "[SAVE] Ignoring runtime resource '{}' with missing asset 'assets/{}'",
                    loot.id, loot.asset_id
                );
                cleanup_needed = true;
                continue;
            }
            level_data.props.push(prop);
        }
        (removed_prop_ids, cleanup_needed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::enemy::EnemyDefinition;
    use crate::data::world::level::{
        ColliderType, LevelEventData, LevelEventTriggerData, LevelEventTriggerKind, PropData,
    };

    #[test]
    fn default_runtime_content_prepares_as_one_valid_bundle() {
        let enemies = EnemyRegistry::try_load_dir("data/enemies").unwrap();
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();

        let prepared = EngineState::prepare_level("movement_test", &enemies, &relics).unwrap();

        assert_eq!(prepared.name, "movement_test");
        assert!(!prepared.data.props.is_empty());
        assert!(!prepared.model.0.is_empty());
    }

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

    #[test]
    fn first_ascent_prepares_with_materialized_enemies() {
        let enemies = EnemyRegistry::try_load_dir("data/enemies").unwrap();
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();

        let prepared = EngineState::prepare_level("ashwalk_01", &enemies, &relics).unwrap();

        assert!(prepared.data.props.len() <= 14);
        assert!(!prepared.model.0.is_empty());
        assert!(prepared
            .data
            .props
            .iter()
            .filter(|prop| prop.enemy_type.is_some())
            .all(|prop| prop.enemy_health > 0.0));
        let ashwarden = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("ashwarden_elite"))
            .unwrap();
        assert_eq!(ashwarden.enemy_type.as_deref(), Some("ashwarden"));
        assert_eq!(ashwarden.enemy_health, 220.0);
        assert_eq!(ashwarden.event_id.as_deref(), Some("ashwarden_fall"));
        let activation_range = enemies.get("ashwarden").unwrap().activation_range;
        let warden_position = Vec3::from_array(ashwarden.position);
        let final_trail_position = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("trail_resource_03"))
            .map(|prop| Vec3::from_array(prop.position))
            .unwrap();
        let anchor_position = prepared
            .data
            .props
            .iter()
            .find(|prop| prop.anchor_id.as_deref() == Some("ashwalk_summit"))
            .map(|prop| Vec3::from_array(prop.position))
            .unwrap();
        assert!(
            Vec3::from_array(prepared.data.player_spawn).distance(warden_position)
                > activation_range
        );
        assert!(final_trail_position.distance(warden_position) < activation_range);
        assert!(anchor_position.distance(warden_position) < activation_range);
        assert_eq!(prepared.data.mountain_reactions.len(), 2);
        assert!(relics.get("debt_of_the_last_keeper").is_some());
    }

    #[test]
    fn enemy_definitions_materialize_level_props() {
        let enemy_registry = EnemyRegistry::from_definitions(vec![EnemyDefinition {
            id: "burdened".to_string(),
            display_name: "Burdened".to_string(),
            role: "tank".to_string(),
            behavior_tag: "slow_chase_melee".to_string(),
            model_asset: "enemies/burdened.obj".to_string(),
            collider_type: ColliderType::Sphere,
            visual_tell: "wide tank silhouette".to_string(),
            health: 120.0,
            damage: 18.0,
            move_speed: 1.4,
            activation_range: 14.0,
            attack_range: 1.8,
            attack_windup: 0.75,
            attack_cooldown: 1.8,
        }])
        .unwrap();
        let mut level = LevelData {
            version: crate::data::world::level::CURRENT_LEVEL_VERSION,
            name: "Materialize Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: Default::default(),
            base_material: Default::default(),
            mountain_reactions: Vec::new(),
            props: vec![PropData {
                id: None,
                display_name: None,
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::None,
                surface_material: None,
                brush_geometry: None,
                is_climbable: false,
                is_hurtbox: false,
                item_id: None,
                resource_value: 0,
                anchor_id: None,
                enemy_type: Some("Burdened".to_string()),
                enemy_health: 0.0,
                light_color: None,
                light_intensity: 0.0,
                ambient_sound_id: None,
                trigger_level_id: None,
                loot_table_id: None,
                path_id: None,
                dialogue_id: None,
                event_id: None,
            }],
            asset_imports: Vec::new(),
            loot_tables: Vec::new(),
            paths: Vec::new(),
            events: Vec::new(),
            dialogues: Vec::new(),
        };
        let mut authored_elite = level.props[0].clone();
        authored_elite.id = Some("named_elite".to_string());
        authored_elite.enemy_health = 220.0;
        level.props.push(authored_elite);

        EngineState::apply_enemy_definitions(&mut level, &enemy_registry, "test").unwrap();

        let prop = &level.props[0];
        assert_eq!(prop.asset_id, "enemies/burdened.obj");
        assert_eq!(prop.collider_type, ColliderType::Sphere);
        assert_eq!(prop.enemy_health, 120.0);
        assert_eq!(level.props[1].enemy_health, 220.0);
    }

    #[test]
    fn enemy_runtime_state_matches_prop_slots() {
        let level = LevelData {
            version: crate::data::world::level::CURRENT_LEVEL_VERSION,
            name: "Cooldown Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            atmosphere: Default::default(),
            base_material: Default::default(),
            mountain_reactions: Vec::new(),
            props: vec![
                PropData {
                    id: None,
                    display_name: None,
                    asset_id: "Cube.obj".to_string(),
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    collider_type: ColliderType::None,
                    surface_material: None,
                    brush_geometry: None,
                    is_climbable: false,
                    is_hurtbox: false,
                    item_id: None,
                    resource_value: 0,
                    anchor_id: None,
                    enemy_type: None,
                    enemy_health: 0.0,
                    light_color: None,
                    light_intensity: 0.0,
                    ambient_sound_id: None,
                    trigger_level_id: None,
                    loot_table_id: None,
                    path_id: None,
                    dialogue_id: None,
                    event_id: None,
                },
                PropData {
                    id: None,
                    display_name: None,
                    asset_id: "Cube.obj".to_string(),
                    position: [1.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    collider_type: ColliderType::Sphere,
                    surface_material: None,
                    brush_geometry: None,
                    is_climbable: false,
                    is_hurtbox: false,
                    item_id: None,
                    resource_value: 0,
                    anchor_id: None,
                    enemy_type: Some("Ashbound".to_string()),
                    enemy_health: 40.0,
                    light_color: None,
                    light_intensity: 0.0,
                    ambient_sound_id: None,
                    trigger_level_id: None,
                    loot_table_id: None,
                    path_id: None,
                    dialogue_id: None,
                    event_id: None,
                },
            ],
            asset_imports: Vec::new(),
            loot_tables: Vec::new(),
            paths: Vec::new(),
            events: Vec::new(),
            dialogues: Vec::new(),
        };

        assert_eq!(
            EngineState::enemy_runtime_for_level(&level),
            vec![
                EnemyRuntimeState::default(),
                EnemyRuntimeState::for_max_health(40.0)
            ]
        );
    }

    #[test]
    fn saved_level_event_state_restores_against_current_level_ids() {
        let mut level = LevelData::default_level();
        let mut boss_door = test_level_event("boss_door");
        boss_door.trigger.flag_id = Some("gate_open".to_string());
        let mut repeatable_rite = test_level_event("repeatable_rite");
        repeatable_rite.once = false;
        repeatable_rite.trigger.kind = LevelEventTriggerKind::Interact;
        level.events = vec![
            test_level_event("intro"),
            test_level_event("gate_open_reward"),
            boss_door,
            repeatable_rite,
        ];
        let save = crate::game::save::SaveData {
            version: crate::game::save::SAVE_VERSION,
            level_name: "foundation_test".to_string(),
            cycle_number: 1,
            unsecured_resource: 0,
            banked_resource: 0,
            active_anchor_id: None,
            respawn_position: None,
            relic_inventory: Vec::new(),
            equipped_relic_id: None,
            fired_level_events: vec![
                "missing_old_event".to_string(),
                "gate_open_reward".to_string(),
                "repeatable_rite".to_string(),
            ],
            level_flags: vec!["gate_open".to_string(), "obsolete_flag".to_string()],
            removed_prop_ids: Vec::new(),
            runtime_loot: Vec::new(),
            pending_mountain_reactions: Vec::new(),
        };

        let (fired, event_cleanup_needed) =
            EngineState::level_event_runtime_for_saved_level(&level, Some(&save));
        let (flags, flag_cleanup_needed) =
            EngineState::level_flags_for_saved_level(&level, Some(&save));

        assert_eq!(fired, vec![false, true, false, false]);
        assert!(event_cleanup_needed);
        assert!(flags.contains("gate_open"));
        assert_eq!(flags.len(), 1);
        assert!(flag_cleanup_needed);
    }

    #[test]
    fn manual_event_readiness_distinguishes_requirements_from_consumed_events() {
        let mut event = test_level_event("anchor_claim");
        event.trigger.kind = LevelEventTriggerKind::Manual;
        event.trigger.flag_id = Some("keeper_fallen".to_string());
        let events = vec![event];
        let mut flags = HashSet::new();

        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[false], &flags, "anchor_claim"),
            ManualLevelEventStatus::MissingFlag("keeper_fallen".to_string())
        );

        flags.insert("keeper_fallen".to_string());
        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[false], &flags, "anchor_claim"),
            ManualLevelEventStatus::Ready
        );
        assert_eq!(
            EngineState::manual_level_event_status_for(&events, &[true], &flags, "anchor_claim"),
            ManualLevelEventStatus::AlreadyFired
        );
    }

    #[test]
    fn saved_world_state_removes_authored_props_and_restores_loose_loot() {
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();
        let mut level = LevelData::default_level();
        let mut defeated: PropData =
            serde_json::from_str(r#"{ "id": "warden", "asset_id": "Cube.obj" }"#).unwrap();
        defeated.enemy_type = Some("ashbound".to_string());
        defeated.enemy_health = 50.0;
        let remaining: PropData = serde_json::from_str(
            r#"{ "id": "remaining_pickup", "asset_id": "Cube.obj", "resource_value": 5 }"#,
        )
        .unwrap();
        level.props = vec![defeated, remaining];

        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.removed_prop_ids = vec!["warden".to_string()];
        save.runtime_loot = vec![
            crate::game::save::SavedRuntimeLoot {
                id: "runtime_loot_abcd_0".to_string(),
                asset_id: "pickups/resource_shard.obj".to_string(),
                position: [3.0, 4.0, 5.0],
                scale: [0.35, 0.35, 0.35],
                item_id: None,
                resource_value: 10,
            },
            crate::game::save::SavedRuntimeLoot {
                id: "runtime_loot_abcd_1".to_string(),
                asset_id: "obsolete/relic.obj".to_string(),
                position: [4.0, 4.0, 5.0],
                scale: [0.35, 0.35, 0.35],
                item_id: Some("debt_of_the_last_keeper".to_string()),
                resource_value: 0,
            },
        ];

        let (removed_ids, cleanup_needed) =
            EngineState::apply_saved_world_state(&mut level, &save, &relics);

        assert_eq!(removed_ids, HashSet::from(["warden".to_string()]));
        assert!(cleanup_needed);
        assert!(level
            .props
            .iter()
            .all(|prop| prop.id.as_deref() != Some("warden")));
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("remaining_pickup")));
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("runtime_loot_abcd_0")));
        let restored_relic = level
            .props
            .iter()
            .find(|prop| prop.id.as_deref() == Some("runtime_loot_abcd_1"))
            .unwrap();
        assert_eq!(restored_relic.asset_id, "pickups/relic_chain_sigil.obj");
    }

    #[test]
    fn saved_mountain_queue_ignores_profiles_removed_by_content_updates() {
        let level = LevelData::try_load("levels/ashwalk_01.json").unwrap();
        let mut save = SaveData::from_runtime(
            "ashwalk_01",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.pending_mountain_reactions = vec![
            "obsolete_answer".to_string(),
            "first_claim_bound".to_string(),
        ];

        assert_eq!(
            EngineState::mountain_reactions_for_saved_level(&level, Some(&save)),
            (VecDeque::from(["first_claim_bound".to_string()]), true)
        );
    }

    #[test]
    fn saved_anchor_uses_current_authored_position_and_rejects_removed_anchors() {
        let mut level = LevelData::default_level();
        let mut anchor: PropData = serde_json::from_str(
            r#"{ "id": "first_anchor_prop", "asset_id": "Cube.obj", "position": [5.0, 6.0, 7.0], "anchor_id": "first_anchor" }"#,
        )
        .unwrap();
        anchor.display_name = Some("The First Anchor".to_string());
        level.props = vec![anchor];
        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.active_anchor_id = Some("first_anchor".to_string());
        save.respawn_position = Some([1.0, 2.0, 3.0]);

        let (progress, cleanup_needed) = EngineState::progress_for_saved_level(&level, &save);

        assert_eq!(progress.active_anchor_id.as_deref(), Some("first_anchor"));
        assert_eq!(progress.respawn_position, Some([5.0, 6.0, 7.0]));
        assert!(cleanup_needed);

        level.props.clear();
        let (progress, cleanup_needed) = EngineState::progress_for_saved_level(&level, &save);
        assert!(progress.active_anchor_id.is_none());
        assert!(progress.respawn_position.is_none());
        assert!(cleanup_needed);
    }

    #[test]
    fn saved_removals_cannot_delete_non_consumable_world_props() {
        let relics = RelicRegistry::try_load_dir("data/relics").unwrap();
        let mut level = LevelData::default_level();
        let anchor: PropData = serde_json::from_str(
            r#"{ "id": "anchor_prop", "asset_id": "Cube.obj", "anchor_id": "anchor" }"#,
        )
        .unwrap();
        level.props = vec![anchor];
        let mut save = SaveData::from_runtime(
            "foundation_test",
            &RunProgress::new(),
            &EquippedRelic::new(),
            &CycleState::new(1),
        );
        save.removed_prop_ids = vec!["anchor_prop".to_string()];

        let (removed_ids, cleanup_needed) =
            EngineState::apply_saved_world_state(&mut level, &save, &relics);

        assert!(removed_ids.is_empty());
        assert!(cleanup_needed);
        assert!(level
            .props
            .iter()
            .any(|prop| prop.id.as_deref() == Some("anchor_prop")));
    }

    fn test_level_event(id: &str) -> LevelEventData {
        LevelEventData {
            id: id.to_string(),
            once: true,
            trigger: LevelEventTriggerData {
                kind: LevelEventTriggerKind::OnEnter,
                position: [0.0, 0.0, 0.0],
                radius: 2.5,
                prop_id: None,
                flag_id: None,
            },
            actions: Vec::new(),
        }
    }
}

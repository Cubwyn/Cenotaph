// src/engine/state.rs
// EngineState owns every runtime subsystem and the GPU surface.
//
// Responsibilities of THIS file:
//   - Struct definition (all fields)
//   - Construction (new)
//   - Resize
//   - Render (GPU draw calls)
//
// Per-frame logic lives in:
//   engine/update.rs  — update_physics / update_visuals / gameplay
//   engine/sync.rs    — sync_instances
//   engine/loader.rs  — load_textures_from_disk / load_prop_assets

use std::sync::Arc;

use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;
use winit::window::Window;

use crate::core::engine::loader::{load_prop_assets, load_textures_from_disk};
use crate::data::config::gameplay::GameConfig;
use crate::data::enemy::EnemyRegistry;
use crate::data::world::level::LevelData;
use crate::game::enemy::EnemyRuntimeState;
use crate::game::player::PlayerState;
use crate::game::progression::RunProgress;
use crate::systems::audio::AudioSystem;
use crate::systems::physics::engine::PhysicsEngine;
use crate::systems::render::assets::{AssetManager, DrawGroup, RenderAssetMeshPart};
use crate::systems::render::camera::{Camera, CameraController, CameraUniform};
use crate::systems::render::hud::HudSystem;
use crate::systems::render::instance::InstanceRaw;
use crate::systems::render::lighting::LightingSystem;
use crate::systems::render::mesh::{empty_model, try_load_model, ModelData};
use crate::systems::render::pipeline::RenderPipeline;
use crate::systems::render::texture::TextureManager;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    Playing,
    Paused,
}

// ── EngineState ───────────────────────────────────────────────────────────────

pub struct EngineState {
    // ── Window / GPU surface ──────────────────────────────────────────────────
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,

    // ── Render pipeline ───────────────────────────────────────────────────────
    pub render_pipeline: wgpu::RenderPipeline,
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    // ── Map geometry ──────────────────────────────────────────────────────────
    pub map_vertex_buffer: wgpu::Buffer,
    pub map_parts: Vec<RenderAssetMeshPart>,
    pub map_instance_buffer: wgpu::Buffer,

    // ── Asset / texture registries ────────────────────────────────────────────
    pub assets: AssetManager,
    pub texture_manager: TextureManager,
    pub active_draw_groups: Vec<DrawGroup>,

    // ── Camera ────────────────────────────────────────────────────────────────
    pub camera: Camera,
    pub camera_controller: CameraController,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,

    // ── Lighting system ───────────────────────────────────────────────────────
    pub lighting: LightingSystem,

    // ── HUD system ────────────────────────────────────────────────────────────
    pub hud: HudSystem,
    pub audio: Option<AudioSystem>,
    pub game_mode: GameMode,

    // ── Subsystems ────────────────────────────────────────────────────────────
    pub physics: PhysicsEngine,

    // ── Game data ─────────────────────────────────────────────────────────────
    pub config_data: GameConfig,
    pub enemy_registry: EnemyRegistry,
    pub level_data: LevelData,
    pub level_name: String,
    pub player: PlayerState,
    pub progress: RunProgress,

    // ── Shared frame cooldown (used by update.rs) ─────────────────────────────
    /// Time remaining (seconds) before the next action is allowed.
    pub action_cooldown: f32,
    /// Per-prop enemy runtime state, aligned with `level_data.props`.
    pub enemy_runtime: Vec<EnemyRuntimeState>,
    /// Accumulator for debug logging (seconds since last print).
    pub debug_timer: f32,

    // ── Level transitions ──────────────────────────────────────────────────────
    /// If Some, the engine should load this level on the next frame.
    pub pending_transition: Option<String>,
}

impl EngineState {
    // ── Construction ──────────────────────────────────────────────────────────

    pub async fn new(window: Arc<Window>, level_name: String) -> Self {
        // Load configuration from config/bindings.toml and config/tuning.toml
        let config_data = GameConfig::load();
        let enemy_registry = EnemyRegistry::load_dir("data/enemies");
        let size = window.inner_size();

        // ── wgpu device setup ─────────────────────────────────────────────────
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: Default::default(),
                ..Default::default()
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // ── Bind group layouts ────────────────────────────────────────────────
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
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // ── Texture manager ───────────────────────────────────────────────────
        let mut texture_manager = TextureManager::new(&device, &queue, &texture_bind_group_layout);
        load_textures_from_disk(
            &device,
            &queue,
            &texture_bind_group_layout,
            &mut texture_manager,
        );

        // ── Level data ────────────────────────────────────────────────────────
        let level_path = format!("levels/{}.json", level_name);
        let mut level_data = LevelData::load(&level_path);
        Self::apply_enemy_definitions(&mut level_data, &enemy_registry, &level_path);
        Self::report_level_validation(&level_data, &level_path);
        println!(
            "[DEBUG] Level loaded: {}, props: {}",
            level_path,
            level_data.props.len()
        );

        // ── Base map ──────────────────────────────────────────────────────────
        let (map_vertices, map_mesh_parts, phys_points, phys_indices) =
            Self::load_map_model(&level_data);

        println!(
            "[DEBUG] Render: {} map vertices, {} mesh parts",
            map_vertices.len(),
            map_mesh_parts.len()
        );
        if !map_vertices.is_empty() {
            println!(
                "[DEBUG] Render: First vertex pos: {:?}",
                map_vertices[0].position
            );
        }

        let map_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Vertex Buffer"),
            contents: bytemuck::cast_slice(&map_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let mut map_parts = Vec::new();
        for part in map_mesh_parts {
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Map Index Buffer"),
                contents: bytemuck::cast_slice(&part.indices),
                usage: wgpu::BufferUsages::INDEX,
            });
            map_parts.push(RenderAssetMeshPart {
                index_buffer,
                num_indices: part.indices.len() as u32,
                texture_name: part.texture_name,
            });
        }

        // Offset the map rendering to align with physics ground (Y=124.5)
        // Map vertices start at Y=0, so we offset by +124.5 to match ground
        let map_y_offset = 124.5;
        let map_instance_data = [InstanceRaw {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, map_y_offset, 0.0, 1.0],
            ],
        }];
        println!("[DEBUG] Map rendered at Y offset: {}", map_y_offset);
        let map_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Map Instance Buffer"),
            contents: bytemuck::cast_slice(&map_instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        // ── Prop assets ───────────────────────────────────────────────────────
        let mut assets = AssetManager::new();
        load_prop_assets(&device, &mut assets);

        // ── Generate asset catalog ────────────────────────────────────────────
        let catalog = crate::core::engine::asset_catalog::AssetCatalog::scan("assets");
        let _ = catalog.save("assets/props.json");
        println!("[DEBUG] Assets loaded: {}", assets.len());

        // ── Camera ────────────────────────────────────────────────────────────
        // Camera starts slightly above the player spawn height (Y offset + player spawn Y)
        let map_y_offset_f = 124.5;
        let camera = Camera {
            position: Vec3::new(
                level_data.player_spawn[0],
                level_data.player_spawn[1] + map_y_offset_f + 1.0,
                level_data.player_spawn[2],
            ),
            yaw: -1.5,
            pitch: 0.0,
            aspect: config.width as f32 / config.height as f32,
            fovy: 0.78,
            znear: 0.1,
            zfar: 500.0,
        };
        let camera_controller = CameraController::new(config_data.camera.sensitivity);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

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

        // ── Lighting system ───────────────────────────────────────────────────
        let lighting = LightingSystem::new(&device, &queue);

        // ── Shader + pipeline ─────────────────────────────────────────────────
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

        // ── Depth buffer ──────────────────────────────────────────────────────
        let depth_size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: depth_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ── Physics ───────────────────────────────────────────────────────────
        let mut physics = PhysicsEngine::new(
            level_data.player_spawn,
            phys_points,
            phys_indices,
            &config_data.physics,
        );
        for prop in &level_data.props {
            let asset_path = format!("assets/{}", prop.asset_id);
            match try_load_model(&asset_path) {
                Ok((_v, _p, pp, pi)) => {
                    physics.add_prop(prop, &pp, &pi);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to load prop model '{}': {}", asset_path, e);
                    physics.add_prop(prop, &[], &[]);
                }
            }
        }

        let hud = HudSystem::new(&device, config.format);
        let mut audio = AudioSystem::new();
        if let Some(audio_system) = audio.as_mut() {
            audio_system.start_ambient();
        }

        let player = PlayerState::new(&config_data.player);
        let progress = RunProgress::new();
        let enemy_runtime = Self::enemy_runtime_for_level(&level_data);
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
            map_vertex_buffer,
            map_parts,
            map_instance_buffer,
            assets,
            texture_manager,
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
            physics,
            config_data,
            enemy_registry,
            level_data,
            level_name,
            player,
            progress,
            action_cooldown: 0.0,
            enemy_runtime,
            debug_timer: 0.0,
            pending_transition: None,
        };

        state.sync_instances();
        state
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);

        let depth_size = wgpu::Extent3d {
            width: self.config.width,
            height: self.config.height,
            depth_or_array_layers: 1,
        };
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: depth_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        self.depth_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.camera.aspect = self.config.width as f32 / self.config.height as f32;
    }

    // ── Render ────────────────────────────────────────────────────────────────

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

        // ── 3D scene pass ───────────────────────────────────────────────────
        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("3D Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.04,
                            b: 0.04,
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

            // Draw base map
            rp.set_vertex_buffer(0, self.map_vertex_buffer.slice(..));
            rp.set_vertex_buffer(1, self.map_instance_buffer.slice(..));
            for part in &self.map_parts {
                let bg = self.texture_manager.get(&part.texture_name);
                rp.set_bind_group(1, bg, &[]);
                rp.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                rp.draw_indexed(0..part.num_indices, 0, 0..1);
            }

            // Draw prop instances
            for group in &self.active_draw_groups {
                if let Some(asset) = self.assets.get(&group.asset_id) {
                    rp.set_vertex_buffer(0, asset.vertex_buffer.slice(..));
                    rp.set_vertex_buffer(1, group.instance_buffer.slice(..));
                    for part in &asset.parts {
                        let bg = self.texture_manager.get(&part.texture_name);
                        rp.set_bind_group(1, bg, &[]);
                        rp.set_index_buffer(part.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                        rp.draw_indexed(0..part.num_indices, 0, 0..group.num_instances);
                    }
                }
            }
        }

        // ── HUD overlay pass (no depth test, alpha blending) ────────────────
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
            let stamina_ratio = self.player.stamina.display_ratio();

            self.hud.draw(
                &mut rp,
                &self.queue,
                health_ratio,
                stamina_ratio,
                self.player.hit_flash_timer,
                self.game_mode == GameMode::Paused,
            );
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    // ── Level loading (runtime) ───────────────────────────────────────────────

    /// Replaces current level data with a new level, rebuilding all map geometry,
    /// physics colliders, prop instances, and resetting camera position.
    pub fn load_level(&mut self, new_level_name: &str) {
        let level_path = format!("levels/{}.json", new_level_name);
        let mut level_data = LevelData::load(&level_path);
        Self::apply_enemy_definitions(&mut level_data, &self.enemy_registry, &level_path);
        Self::report_level_validation(&level_data, &level_path);
        println!(
            "[LEVEL] Loaded '{}': {} props",
            new_level_name,
            level_data.props.len()
        );

        // ── Load base map ─────────────────────────────────────────────────────
        let (map_vertices, map_mesh_parts, phys_points, phys_indices) =
            Self::load_map_model(&level_data);

        let map_vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Map Vertex Buffer"),
                contents: bytemuck::cast_slice(&map_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut map_parts = Vec::new();
        for part in map_mesh_parts {
            let index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Index Buffer"),
                    contents: bytemuck::cast_slice(&part.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
            map_parts.push(RenderAssetMeshPart {
                index_buffer,
                num_indices: part.indices.len() as u32,
                texture_name: part.texture_name,
            });
        }

        let map_y_offset = 124.5;
        let map_instance_data = [InstanceRaw {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, map_y_offset, 0.0, 1.0],
            ],
        }];
        let map_instance_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Map Instance Buffer"),
                    contents: bytemuck::cast_slice(&map_instance_data),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        // ── Rebuild physics ───────────────────────────────────────────────────
        self.physics = PhysicsEngine::new(
            level_data.player_spawn,
            phys_points,
            phys_indices,
            &self.config_data.physics,
        );
        for prop in &level_data.props {
            let asset_path = format!("assets/{}", prop.asset_id);
            match try_load_model(&asset_path) {
                Ok((_v, _p, pp, pi)) => {
                    self.physics.add_prop(prop, &pp, &pi);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to load prop model '{}': {}", asset_path, e);
                    self.physics.add_prop(prop, &[], &[]);
                }
            }
        }

        // ── Reset camera to new spawn ─────────────────────────────────────────
        let map_y = 124.5;
        self.camera.position = Vec3::new(
            level_data.player_spawn[0],
            level_data.player_spawn[1] + map_y + 1.0,
            level_data.player_spawn[2],
        );

        // ── Swap state ────────────────────────────────────────────────────────
        let enemy_runtime = Self::enemy_runtime_for_level(&level_data);
        self.map_vertex_buffer = map_vertex_buffer;
        self.map_parts = map_parts;
        self.map_instance_buffer = map_instance_buffer;
        self.level_data = level_data;
        self.level_name = new_level_name.to_string();
        self.enemy_runtime = enemy_runtime;

        // Reset runtime state
        self.action_cooldown = 0.0;
        self.progress.clear_anchor();
        self.player
            .reset_for_level_transition(&self.config_data.player);

        self.sync_instances();
        println!("[LEVEL] Transition complete: {}", new_level_name);
    }

    // ── Lighting updates ──────────────────────────────────────────────────────

    pub fn update_lighting(&mut self) {
        // Light follows camera with slight offset for more natural feel
        let light_pos = [
            self.camera.position.x + 0.5,
            self.camera.position.y + self.config_data.lighting.sun_position_offset,
            self.camera.position.z + 0.3,
        ];
        // Warm torch-like light color with moderate intensity
        self.lighting.update_light(
            &self.queue,
            light_pos,
            self.config_data.lighting.sun_color,
            self.config_data.lighting.sun_intensity,
        );

        // Atmospheric fog — density tuned for underground cavern feel
        self.lighting.update_fog(
            &self.queue,
            self.config_data.world.fog_density,
            self.config_data.lighting.ambient_color,
        );
    }

    fn report_level_validation(level_data: &LevelData, level_path: &str) {
        if let Err(errors) = level_data.validate() {
            eprintln!(
                "[LEVEL VALIDATION] {} has {} issue(s):",
                level_path,
                errors.len()
            );
            for error in errors {
                eprintln!("  - {}", error);
            }
        }
    }

    fn apply_enemy_definitions(
        level_data: &mut LevelData,
        enemy_registry: &EnemyRegistry,
        level_path: &str,
    ) {
        for prop in &mut level_data.props {
            let Some(enemy_type) = prop.enemy_type.as_deref() else {
                continue;
            };

            let Some(enemy) = enemy_registry.get(enemy_type) else {
                eprintln!(
                    "[ENEMY DATA] Level '{}' references unknown enemy_type '{}'",
                    level_path, enemy_type
                );
                continue;
            };

            prop.asset_id = enemy.model_asset.clone();
            prop.collider_type = enemy.collider_type.clone();
            prop.enemy_health = enemy.health;
        }
    }

    fn enemy_runtime_for_level(level_data: &LevelData) -> Vec<EnemyRuntimeState> {
        vec![EnemyRuntimeState::default(); level_data.props.len()]
    }

    fn load_map_model(level_data: &LevelData) -> ModelData {
        match try_load_model(&level_data.base_map) {
            Ok(model) => model,
            Err(error) => {
                eprintln!(
                    "[ERROR] Failed to load base map '{}': {}",
                    level_data.base_map, error
                );
                let fallback = LevelData::default_level();
                match try_load_model(&fallback.base_map) {
                    Ok(model) => {
                        eprintln!(
                            "[LEVEL] Falling back to default base map '{}'",
                            fallback.base_map
                        );
                        model
                    }
                    Err(fallback_error) => {
                        eprintln!(
                            "[ERROR] Failed to load fallback base map '{}': {}",
                            fallback.base_map, fallback_error
                        );
                        eprintln!("[LEVEL] Using empty render model and physics fallback floor.");
                        empty_model()
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::enemy::EnemyDefinition;
    use crate::data::world::level::{ColliderType, PropData};

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
            name: "Materialize Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            props: vec![PropData {
                asset_id: "Cube.obj".to_string(),
                position: [0.0, 0.0, 0.0],
                rotation: [0.0, 0.0, 0.0],
                scale: [1.0, 1.0, 1.0],
                collider_type: ColliderType::None,
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
            }],
        };

        EngineState::apply_enemy_definitions(&mut level, &enemy_registry, "test");

        let prop = &level.props[0];
        assert_eq!(prop.asset_id, "enemies/burdened.obj");
        assert_eq!(prop.collider_type, ColliderType::Sphere);
        assert_eq!(prop.enemy_health, 120.0);
    }

    #[test]
    fn enemy_runtime_state_matches_prop_slots() {
        let level = LevelData {
            name: "Cooldown Test".to_string(),
            base_map: "assets/Cube.obj".to_string(),
            player_spawn: [0.0, 0.0, 0.0],
            props: vec![
                PropData {
                    asset_id: "Cube.obj".to_string(),
                    position: [0.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    collider_type: ColliderType::None,
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
                },
                PropData {
                    asset_id: "Cube.obj".to_string(),
                    position: [1.0, 0.0, 0.0],
                    rotation: [0.0, 0.0, 0.0],
                    scale: [1.0, 1.0, 1.0],
                    collider_type: ColliderType::Sphere,
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
                },
            ],
        };

        assert_eq!(
            EngineState::enemy_runtime_for_level(&level),
            vec![EnemyRuntimeState::default(), EnemyRuntimeState::default()]
        );
    }
}

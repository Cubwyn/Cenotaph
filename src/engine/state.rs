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
//   engine/update.rs  — update_physics / update_visuals / gameplay / editor
//   engine/sync.rs    — sync_instances
//   engine/loader.rs  — load_textures_from_disk / load_prop_assets

use std::sync::Arc;

use glam::Vec3;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use wgpu::util::DeviceExt;

use crate::config::gameplay::GameConfig;
use crate::engine::debug::DebugManager;
use crate::engine::loader::{load_prop_assets, load_textures_from_disk};
use crate::physics::PhysicsEngine;
use crate::render::assets::{AssetManager, DrawGroup, RenderAssetMeshPart};
use crate::render::camera::{Camera, CameraController, CameraUniform};
use crate::render::instance::InstanceRaw;
use crate::render::lighting::LightingSystem;
use crate::render::mesh::load_model;
use crate::render::pipeline::RenderPipeline;
use crate::render::texture::TextureManager;
use crate::world::level::LevelData;

// Editor subsystem — compiled out in ship builds
#[cfg(feature = "editor")]
use crate::editor::EditorState;

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

    // ── Subsystems ────────────────────────────────────────────────────────────
    pub physics: PhysicsEngine,
    pub debug: DebugManager,

    // ── Editor (dev builds only) ──────────────────────────────────────────────
    #[cfg(feature = "editor")]
    pub editor: EditorState,

    // ── Game data ─────────────────────────────────────────────────────────────
    pub config_data: GameConfig,
    pub level_data: LevelData,
    pub level_name: String,

    // ── Shared frame cooldown (used by update.rs) ─────────────────────────────
    pub action_cooldown: u32,
}

impl EngineState {
    // ── Construction ──────────────────────────────────────────────────────────

    pub async fn new(window: Arc<Window>, level_name: String) -> Self {
        let mut config_data = GameConfig::default();
        config_data.keys = GameConfig::load_keys();
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
        let mut texture_manager =
            TextureManager::new(&device, &queue, &texture_bind_group_layout);
        load_textures_from_disk(&device, &queue, &texture_bind_group_layout, &mut texture_manager);

        // ── Level data ────────────────────────────────────────────────────────
        let level_path = format!("levels/{}.json", level_name);
        let level_data = LevelData::load(&level_path);

        // ── Base map ──────────────────────────────────────────────────────────
        let (map_vertices, map_mesh_parts, phys_points, phys_indices) =
            load_model(&level_data.base_map);

        let map_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Map Vertex Buffer"),
                contents: bytemuck::cast_slice(&map_vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut map_parts = Vec::new();
        for part in map_mesh_parts {
            let index_buffer =
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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

        let map_instance_data = [InstanceRaw {
            model: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }];
        let map_instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Map Instance Buffer"),
                contents: bytemuck::cast_slice(&map_instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            });

        // ── Prop assets ───────────────────────────────────────────────────────
        let mut assets = AssetManager::new();
        load_prop_assets(&device, &mut assets);

        // ── Generate asset catalog ────────────────────────────────────────────
        let catalog = crate::engine::asset_catalog::AssetCatalog::scan("assets");
        let _ = catalog.save("assets/props.json");

        // ── Camera ────────────────────────────────────────────────────────────
        let camera = Camera {
            position: Vec3::new(-10.0, 2.0, -10.0),
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

        let camera_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
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
            source: wgpu::ShaderSource::Wgsl(include_str!("../render/shader.wgsl").into()),
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_view =
            depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // ── Physics ───────────────────────────────────────────────────────────
        let mut physics = PhysicsEngine::new(
            level_data.player_spawn,
            phys_points,
            phys_indices,
            &config_data.physics,
        );
        for prop in &level_data.props {
            let asset_path = format!("assets/{}", prop.asset_id);
            if let Ok((_v, _p, pp, pi)) = std::panic::catch_unwind(
                std::panic::AssertUnwindSafe(|| load_model(&asset_path)),
            ) {
                physics.add_prop(prop, &pp, &pi);
            }
        }

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
            lighting,
            physics,
            debug: DebugManager::new(),
            #[cfg(feature = "editor")]
            editor: EditorState::new(),
            config_data,
            level_data,
            level_name,
            action_cooldown: 0,
        };

        state.debug.log("EngineState initialised.");
        state.sync_instances();
        state
    }

    // ── Resize ────────────────────────────────────────────────────────────────

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 { return; }
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
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
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

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.05,
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
                        rp.set_index_buffer(
                            part.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        rp.draw_indexed(0..part.num_indices, 0, 0..group.num_instances);
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    // ── Lighting updates ──────────────────────────────────────────────────────

    pub fn update_lighting(&mut self) {
        // Update light position to follow camera
        let light_pos = [self.camera.position.x, self.camera.position.y + 5.0, self.camera.position.z];
        self.lighting.update_light(&self.queue, light_pos, [1.0, 0.8, 0.5], 2.0);
        
        // Update fog based on environment
        self.lighting.update_fog(&self.queue, 0.01, [0.1, 0.1, 0.15]);
    }
}

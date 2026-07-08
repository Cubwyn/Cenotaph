// src/render/hud.rs
// Simple HUD overlay using a dedicated 2D pipeline.
// Draws health bar, stamina bar, and crosshair.

use wgpu::util::DeviceExt;

/// A colored vertex in screen-space (NDC: -1..1).
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct HudVertex {
    position: [f32; 2],
    color: [f32; 4],
}

const BAR_WIDTH: f32 = 0.3;
const BAR_HEIGHT: f32 = 0.04;
const BAR_Y: f32 = -0.85;
const BAR_PADDING: f32 = 0.05;
const CROSSHAIR_SIZE: f32 = 0.015;

const HUD_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
    wgpu::VertexAttribute {
        offset: 0,
        shader_location: 0,
        format: wgpu::VertexFormat::Float32x2,
    },
    wgpu::VertexAttribute {
        offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
        shader_location: 1,
        format: wgpu::VertexFormat::Float32x4,
    },
];

pub struct HudSystem {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl HudSystem {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("HUD Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("hud.wgsl").into()),
        });
        let empty_bind_group_layouts: &[&wgpu::BindGroupLayout] = &[];
        let vertex_buffers: &[wgpu::VertexBufferLayout<'static>] = &[wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<HudVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &HUD_VERTEX_ATTRIBUTES,
        }];
        let fragment_targets: &[Option<wgpu::ColorTargetState>] = &[Some(wgpu::ColorTargetState {
            format: surface_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("HUD Pipeline Layout"),
            bind_group_layouts: empty_bind_group_layouts,
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("HUD Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: fragment_targets,
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        let max_verts = 40;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HUD Vertex Buffer"),
            size: (max_verts * std::mem::size_of::<HudVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices: Vec<u16> = Vec::with_capacity(60);
        for quad in 0..10u16 {
            let base = quad * 4;
            indices.extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 1, base + 3]);
        }
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("HUD Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            num_indices: 0,
        }
    }

    fn rect_verts(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> [HudVertex; 4] {
        [
            HudVertex {
                position: [x, y],
                color,
            },
            HudVertex {
                position: [x + w, y],
                color,
            },
            HudVertex {
                position: [x, y + h],
                color,
            },
            HudVertex {
                position: [x + w, y + h],
                color,
            },
        ]
    }

    pub fn draw(
        &mut self,
        render_pass: &mut wgpu::RenderPass,
        queue: &wgpu::Queue,
        health_ratio: f32,
        stamina_ratio: f32,
        hit_flash: f32,
        paused: bool,
    ) {
        let mut verts: Vec<HudVertex> = Vec::with_capacity(40);

        let health_color = if health_ratio > 0.5 {
            [0.2, 0.8, 0.2, 0.9]
        } else if health_ratio > 0.25 {
            [0.9, 0.7, 0.1, 0.9]
        } else {
            [0.9, 0.2, 0.1, 0.9]
        };

        let health_bg = [0.1, 0.1, 0.1, 0.6];
        verts.extend(Self::rect_verts(
            -BAR_WIDTH - 0.002,
            BAR_Y - 0.002,
            BAR_WIDTH * 2.0 + 0.004,
            BAR_HEIGHT + 0.004,
            health_bg,
        ));
        let hw = BAR_WIDTH * 2.0 * health_ratio;
        verts.extend(Self::rect_verts(
            -BAR_WIDTH,
            BAR_Y,
            hw,
            BAR_HEIGHT,
            health_color,
        ));

        let stamina_y = BAR_Y - BAR_HEIGHT - BAR_PADDING;
        verts.extend(Self::rect_verts(
            -BAR_WIDTH - 0.002,
            stamina_y - 0.002,
            BAR_WIDTH * 2.0 + 0.004,
            BAR_HEIGHT + 0.004,
            health_bg,
        ));
        let sw = BAR_WIDTH * 2.0 * stamina_ratio;
        verts.extend(Self::rect_verts(
            -BAR_WIDTH,
            stamina_y,
            sw,
            BAR_HEIGHT,
            [0.9, 0.8, 0.3, 0.85],
        ));

        if paused {
            // Dim the world and show a simple pause emblem without requiring text rendering.
            verts.extend(Self::rect_verts(
                -1.0,
                -1.0,
                2.0,
                2.0,
                [0.0, 0.0, 0.0, 0.45],
            ));
            let pause_color = [0.95, 0.95, 0.95, 0.90];
            verts.extend(Self::rect_verts(-0.085, -0.14, 0.045, 0.28, pause_color));
            verts.extend(Self::rect_verts(0.040, -0.14, 0.045, 0.28, pause_color));
            verts.extend(Self::rect_verts(
                -0.13,
                0.18,
                0.26,
                0.02,
                [0.95, 0.95, 0.95, 0.55],
            ));
        } else {
            let ch = CROSSHAIR_SIZE;
            let crosshair_color = if hit_flash > 0.0 {
                [1.0, 0.3, 0.2, 0.9]
            } else {
                [1.0, 1.0, 1.0, 0.7]
            };
            verts.extend(Self::rect_verts(
                -ch,
                -ch,
                ch * 2.0,
                ch * 2.0,
                crosshair_color,
            ));
        }

        self.num_indices = (verts.len() / 4 * 6) as u32;
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

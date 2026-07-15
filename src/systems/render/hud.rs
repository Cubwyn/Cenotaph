// src/systems/render/hud.rs
// Immediate-mode HUD overlay using a dedicated 2D pipeline. The theme and
// layout modules keep presentation tokens and screen regions independent from
// gameplay state so new widgets can join the HUD without another coordinate
// pile-up.

mod anchor_rite;
mod ascent;
mod dialogue;
mod encounter;
mod layout;
mod markers;
mod notifications;
mod overlays;
mod player_status;
mod state;
mod theme;

use wgpu::util::DeviceExt;

use crate::data::config::ui::UiConfig;

use self::layout::{HudLayout, HudRect};
use self::theme::{with_alpha, HudColor, HudTheme};

pub use self::state::{
    AnchorRiteHudState, AscentHudState, DebugHudState, DialogueHudState, HudFeedEvent, HudFeedback,
    HudFrameState, HudMarkerKind, HudMarkerState, HudWorldMarker, NamedEncounterHudState,
    NamedNoticeHudState, PlayerHudState,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct HudVertex {
    position: [f32; 2],
    color: [f32; 4],
}

const CROSSHAIR_SIZE: f32 = 0.015;
const MAX_HUD_QUADS: usize = 16000;
const MAX_WORLD_MARKERS: usize = 64;
const MAX_EVENT_FEED_ITEMS: usize = 4;
const HUD_TEXT_X_SCALE: f32 = 0.70;
const HUD_MIN_TEXT_SCALE: f32 = 0.32;

#[derive(Debug, Clone, Copy)]
enum HudIcon {
    Vital,
    Stamina,
    Dash,
    Ash,
    Bank,
    Relic,
    Interact,
    Bell,
    Death,
    Pause,
}

#[derive(Debug, Clone, Copy)]
struct HudTextFit {
    scale: f32,
    max_width: f32,
    color: HudColor,
}

#[derive(Debug, Clone, Copy)]
struct HudMeterStyle {
    fill: HudColor,
    trail: HudColor,
    segments: usize,
}

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
    vertices: Vec<HudVertex>,
    num_indices: u32,
    theme: HudTheme,
}

impl HudSystem {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        ui_config: &UiConfig,
    ) -> Self {
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

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("HUD Vertex Buffer"),
            size: (MAX_HUD_QUADS * 4 * std::mem::size_of::<HudVertex>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut indices: Vec<u16> = Vec::with_capacity(MAX_HUD_QUADS * 6);
        for quad in 0..MAX_HUD_QUADS as u16 {
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
            vertices: Vec::with_capacity(MAX_HUD_QUADS * 4),
            num_indices: 0,
            theme: HudTheme::from_config(&ui_config.hud),
        }
    }

    pub fn set_ui_config(&mut self, ui_config: &UiConfig) {
        self.theme = HudTheme::from_config(&ui_config.hud);
    }

    fn push_rect(verts: &mut Vec<HudVertex>, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
        if verts.len() / 4 >= MAX_HUD_QUADS {
            return;
        }
        verts.extend(Self::rect_verts(x, y, w, h, color));
    }

    fn push_centered_rect(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: [f32; 4],
    ) {
        Self::push_rect(verts, x - w * 0.5, y - h * 0.5, w, h, color);
    }

    fn push_outline_rect(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: [f32; 4],
    ) {
        Self::push_rect(verts, x, y, w, thickness, color);
        Self::push_rect(verts, x, y + h - thickness, w, thickness, color);
        Self::push_rect(verts, x, y, thickness, h, color);
        Self::push_rect(verts, x + w - thickness, y, thickness, h, color);
    }

    fn push_quad(verts: &mut Vec<HudVertex>, points: [[f32; 2]; 4], color: HudColor) {
        if verts.len() / 4 >= MAX_HUD_QUADS {
            return;
        }
        verts.extend(points.map(|position| HudVertex { position, color }));
    }

    fn push_triangle(
        verts: &mut Vec<HudVertex>,
        a: [f32; 2],
        b: [f32; 2],
        c: [f32; 2],
        color: HudColor,
    ) {
        Self::push_quad(verts, [a, b, c, c], color);
    }

    fn push_line(
        verts: &mut Vec<HudVertex>,
        from: [f32; 2],
        to: [f32; 2],
        thickness: f32,
        color: HudColor,
    ) {
        let dx = to[0] - from[0];
        let dy = to[1] - from[1];
        let length = (dx * dx + dy * dy).sqrt();
        if length <= f32::EPSILON {
            return;
        }
        let nx = -dy / length * thickness * 0.5;
        let ny = dx / length * thickness * 0.5;
        Self::push_quad(
            verts,
            [
                [from[0] + nx, from[1] + ny],
                [to[0] + nx, to[1] + ny],
                [from[0] - nx, from[1] - ny],
                [to[0] - nx, to[1] - ny],
            ],
            color,
        );
    }

    fn push_diamond(verts: &mut Vec<HudVertex>, center: [f32; 2], size: [f32; 2], color: HudColor) {
        let hx = size[0] * 0.5;
        let hy = size[1] * 0.5;
        Self::push_quad(
            verts,
            [
                [center[0] - hx, center[1]],
                [center[0], center[1] - hy],
                [center[0], center[1] + hy],
                [center[0] + hx, center[1]],
            ],
            color,
        );
    }

    fn push_diamond_outline(
        verts: &mut Vec<HudVertex>,
        center: [f32; 2],
        size: [f32; 2],
        thickness: f32,
        color: HudColor,
    ) {
        let left = [center[0] - size[0] * 0.5, center[1]];
        let bottom = [center[0], center[1] - size[1] * 0.5];
        let top = [center[0], center[1] + size[1] * 0.5];
        let right = [center[0] + size[0] * 0.5, center[1]];
        Self::push_line(verts, left, top, thickness, color);
        Self::push_line(verts, top, right, thickness, color);
        Self::push_line(verts, right, bottom, thickness, color);
        Self::push_line(verts, bottom, left, thickness, color);
    }

    fn push_cut_panel(
        verts: &mut Vec<HudVertex>,
        rect: HudRect,
        corner: [f32; 2],
        fill: HudColor,
        outline: HudColor,
    ) {
        let cx = corner[0].min(rect.w * 0.25);
        let cy = corner[1].min(rect.h * 0.25);
        Self::push_rect(verts, rect.x + cx, rect.y, rect.w - cx * 2.0, rect.h, fill);
        Self::push_rect(verts, rect.x, rect.y + cy, cx, rect.h - cy * 2.0, fill);
        Self::push_rect(
            verts,
            rect.right() - cx,
            rect.y + cy,
            cx,
            rect.h - cy * 2.0,
            fill,
        );
        Self::push_triangle(
            verts,
            [rect.x, rect.y + cy],
            [rect.x + cx, rect.y],
            [rect.x + cx, rect.y + cy],
            fill,
        );
        Self::push_triangle(
            verts,
            [rect.right() - cx, rect.y],
            [rect.right(), rect.y + cy],
            [rect.right() - cx, rect.y + cy],
            fill,
        );
        Self::push_triangle(
            verts,
            [rect.x, rect.top() - cy],
            [rect.x + cx, rect.top()],
            [rect.x + cx, rect.top() - cy],
            fill,
        );
        Self::push_triangle(
            verts,
            [rect.right() - cx, rect.top()],
            [rect.right(), rect.top() - cy],
            [rect.right() - cx, rect.top() - cy],
            fill,
        );

        let thickness = (cy * 0.12).clamp(0.0015, 0.004);
        let bottom_left = [rect.x + cx, rect.y];
        let bottom_right = [rect.right() - cx, rect.y];
        let right_bottom = [rect.right(), rect.y + cy];
        let right_top = [rect.right(), rect.top() - cy];
        let top_right = [rect.right() - cx, rect.top()];
        let top_left = [rect.x + cx, rect.top()];
        let left_top = [rect.x, rect.top() - cy];
        let left_bottom = [rect.x, rect.y + cy];
        for (from, to) in [
            (bottom_left, bottom_right),
            (bottom_right, right_bottom),
            (right_bottom, right_top),
            (right_top, top_right),
            (top_right, top_left),
            (top_left, left_top),
            (left_top, left_bottom),
            (left_bottom, bottom_left),
        ] {
            Self::push_line(verts, from, to, thickness, outline);
        }
    }

    fn push_icon(
        verts: &mut Vec<HudVertex>,
        icon: HudIcon,
        center: [f32; 2],
        size: [f32; 2],
        color: HudColor,
    ) {
        let hx = size[0] * 0.5;
        let hy = size[1] * 0.5;
        let thickness = (size[1] * 0.07).clamp(0.0018, 0.0045);
        match icon {
            HudIcon::Vital => {
                Self::push_diamond_outline(verts, center, size, thickness, with_alpha(color, 0.65));
                Self::push_line(
                    verts,
                    [center[0], center[1] - hy * 0.58],
                    [center[0], center[1] + hy * 0.58],
                    thickness,
                    color,
                );
                Self::push_diamond(verts, center, [hx * 0.52, hy * 0.52], color);
            }
            HudIcon::Stamina => {
                for offset in [-0.52_f32, 0.0, 0.52] {
                    let x = center[0] + hx * offset;
                    Self::push_line(
                        verts,
                        [x - hx * 0.30, center[1] - hy * 0.42],
                        [x + hx * 0.08, center[1]],
                        thickness,
                        color,
                    );
                    Self::push_line(
                        verts,
                        [x + hx * 0.08, center[1]],
                        [x - hx * 0.30, center[1] + hy * 0.42],
                        thickness,
                        color,
                    );
                }
            }
            HudIcon::Dash => {
                Self::push_triangle(
                    verts,
                    [center[0] - hx, center[1] - hy * 0.55],
                    [center[0] + hx, center[1]],
                    [center[0] - hx, center[1] + hy * 0.55],
                    with_alpha(color, 0.78),
                );
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.85, center[1]],
                    [center[0] + hx * 0.55, center[1]],
                    thickness,
                    color,
                );
            }
            HudIcon::Ash => {
                Self::push_diamond(
                    verts,
                    [center[0], center[1] - hy * 0.35],
                    [hx * 0.72, hy * 0.70],
                    with_alpha(color, 0.82),
                );
                Self::push_triangle(
                    verts,
                    [center[0] - hx * 0.35, center[1]],
                    [center[0] + hx * 0.16, center[1] + hy],
                    [center[0] + hx * 0.35, center[1] - hy * 0.04],
                    color,
                );
            }
            HudIcon::Bank => {
                Self::push_line(
                    verts,
                    [center[0], center[1] + hy],
                    [center[0], center[1] - hy * 0.50],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.72, center[1] - hy * 0.10],
                    [center[0], center[1] - hy * 0.72],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0], center[1] - hy * 0.72],
                    [center[0] + hx * 0.72, center[1] - hy * 0.10],
                    thickness,
                    color,
                );
                Self::push_diamond(
                    verts,
                    [center[0], center[1] + hy * 0.60],
                    [hx * 0.45, hy * 0.34],
                    color,
                );
            }
            HudIcon::Relic => {
                Self::push_diamond_outline(verts, center, size, thickness, with_alpha(color, 0.72));
                Self::push_diamond(verts, center, [hx * 0.70, hy * 0.70], color);
                Self::push_line(
                    verts,
                    [center[0], center[1] - hy],
                    [center[0], center[1] + hy],
                    thickness * 0.72,
                    with_alpha(color, 0.82),
                );
            }
            HudIcon::Interact => {
                Self::push_diamond_outline(verts, center, size, thickness, color);
                Self::push_diamond(
                    verts,
                    center,
                    [hx * 0.35, hy * 0.35],
                    with_alpha(color, 0.72),
                );
            }
            HudIcon::Bell => {
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.70, center[1] - hy * 0.35],
                    [center[0] - hx * 0.40, center[1] + hy * 0.55],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.40, center[1] + hy * 0.55],
                    [center[0] + hx * 0.40, center[1] + hy * 0.55],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0] + hx * 0.40, center[1] + hy * 0.55],
                    [center[0] + hx * 0.70, center[1] - hy * 0.35],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0] - hx, center[1] - hy * 0.35],
                    [center[0] + hx, center[1] - hy * 0.35],
                    thickness,
                    color,
                );
                Self::push_diamond(
                    verts,
                    [center[0], center[1] - hy * 0.72],
                    [hx * 0.28, hy * 0.28],
                    color,
                );
            }
            HudIcon::Death => {
                Self::push_diamond_outline(verts, center, size, thickness, with_alpha(color, 0.62));
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.46, center[1] - hy * 0.46],
                    [center[0] + hx * 0.46, center[1] + hy * 0.46],
                    thickness,
                    color,
                );
                Self::push_line(
                    verts,
                    [center[0] - hx * 0.46, center[1] + hy * 0.46],
                    [center[0] + hx * 0.46, center[1] - hy * 0.46],
                    thickness,
                    color,
                );
            }
            HudIcon::Pause => {
                Self::push_rect(
                    verts,
                    center[0] - hx * 0.52,
                    center[1] - hy,
                    hx * 0.34,
                    size[1],
                    color,
                );
                Self::push_rect(
                    verts,
                    center[0] + hx * 0.18,
                    center[1] - hy,
                    hx * 0.34,
                    size[1],
                    color,
                );
            }
        }
    }

    fn pulse(value: f32, duration: f32) -> f32 {
        if duration <= 0.0 {
            0.0
        } else {
            (value / duration).clamp(0.0, 1.0)
        }
    }

    fn add_center_particles(
        verts: &mut Vec<HudVertex>,
        timer: f32,
        duration: f32,
        count: usize,
        spread: f32,
        size: f32,
        color: [f32; 4],
    ) {
        let remaining = Self::pulse(timer, duration);
        if remaining <= 0.0 {
            return;
        }

        let progress = 1.0 - remaining;
        let alpha = color[3] * remaining;
        for i in 0..count {
            let angle = i as f32 / count as f32 * std::f32::consts::TAU;
            let wobble = ((i as f32 * 12.9898 + progress * 4.7).sin()) * 0.018;
            let distance = spread * progress + wobble;
            let x = angle.cos() * distance;
            let y = angle.sin() * distance;
            Self::push_rect(
                verts,
                x - size * 0.5,
                y - size * 0.5,
                size,
                size,
                [color[0], color[1], color[2], alpha],
            );
        }
    }

    fn push_digit(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        digit: u32,
        scale: f32,
        color: [f32; 4],
    ) {
        const SEGMENTS: [[bool; 7]; 10] = [
            [true, true, true, true, true, true, false],
            [false, true, true, false, false, false, false],
            [true, true, false, true, true, false, true],
            [true, true, true, true, false, false, true],
            [false, true, true, false, false, true, true],
            [true, false, true, true, false, true, true],
            [true, false, true, true, true, true, true],
            [true, true, true, false, false, false, false],
            [true, true, true, true, true, true, true],
            [true, true, true, true, false, true, true],
        ];

        let Some(active) = SEGMENTS.get(digit as usize) else {
            return;
        };
        let scale = scale.max(HUD_MIN_TEXT_SCALE);
        let w = 0.026 * scale * HUD_TEXT_X_SCALE;
        let h = 0.048 * scale;
        let tx = 0.0055 * scale * HUD_TEXT_X_SCALE;
        let ty = 0.0055 * scale;
        let half = h * 0.5;

        if active[0] {
            Self::push_rect(verts, x + tx, y + h - ty, w - tx * 2.0, ty, color);
        }
        if active[1] {
            Self::push_rect(verts, x + w - tx, y + half, tx, half - ty, color);
        }
        if active[2] {
            Self::push_rect(verts, x + w - tx, y + ty, tx, half - ty, color);
        }
        if active[3] {
            Self::push_rect(verts, x + tx, y, w - tx * 2.0, ty, color);
        }
        if active[4] {
            Self::push_rect(verts, x, y + ty, tx, half - ty, color);
        }
        if active[5] {
            Self::push_rect(verts, x, y + half, tx, half - ty, color);
        }
        if active[6] {
            Self::push_rect(verts, x + tx, y + half - ty * 0.5, w - tx * 2.0, ty, color);
        }
    }

    fn push_number(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        value: u32,
        scale: f32,
        color: [f32; 4],
    ) {
        let text = value.min(9999).to_string();
        let scale = scale.max(HUD_MIN_TEXT_SCALE);
        let advance = 0.034 * scale * HUD_TEXT_X_SCALE;
        for (index, ch) in text.chars().enumerate() {
            if let Some(digit) = ch.to_digit(10) {
                Self::push_digit(verts, x + index as f32 * advance, y, digit, scale, color);
            }
        }
    }

    fn push_text(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        color: [f32; 4],
    ) {
        Self::push_text_with_x_scale(verts, x, y, text, scale, HUD_TEXT_X_SCALE, color);
    }

    fn push_text_with_x_scale(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        x_scale: f32,
        color: HudColor,
    ) {
        let scale = scale.max(HUD_MIN_TEXT_SCALE);
        let pixel_y = 0.007 * scale;
        let pixel_x = pixel_y * x_scale;
        let shadow = [0.0, 0.0, 0.0, (color[3] * 0.62).min(0.76)];
        Self::push_text_layer(
            verts,
            x + pixel_x * 0.32,
            y - pixel_y * 0.30,
            text,
            pixel_x,
            pixel_y,
            shadow,
        );
        Self::push_text_layer(verts, x, y, text, pixel_x, pixel_y, color);
    }

    fn push_text_layer(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        text: &str,
        pixel_x: f32,
        pixel_y: f32,
        color: HudColor,
    ) {
        let advance = pixel_x * 6.25;

        for (index, ch) in text.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let Some(rows) = Self::glyph_rows(ch.to_ascii_uppercase()) else {
                continue;
            };
            let glyph_x = x + index as f32 * advance;
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) == 0 {
                        continue;
                    }
                    Self::push_rect(
                        verts,
                        glyph_x + col as f32 * pixel_x,
                        y + (6 - row) as f32 * pixel_y,
                        pixel_x * 1.02,
                        pixel_y * 1.02,
                        color,
                    );
                }
            }
        }
    }

    fn push_ui_text(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        color: HudColor,
    ) {
        Self::push_text_with_x_scale(
            verts,
            x,
            y,
            text,
            layout.text_scale(scale),
            layout.glyph_x_scale,
            color,
        );
    }

    fn push_ui_fit_text(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        x: f32,
        y: f32,
        text: &str,
        fit: HudTextFit,
    ) {
        let width = Self::ui_text_width(layout, text, fit.scale);
        let fitted_scale = if width > fit.max_width && width > 0.0 {
            fit.scale * fit.max_width / width
        } else {
            fit.scale
        };
        Self::push_ui_text(verts, layout, x, y, text, fitted_scale, fit.color);
    }

    fn push_ui_centered_text(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        color: HudColor,
    ) {
        let width = Self::ui_text_width(layout, text, scale);
        Self::push_ui_text(verts, layout, x - width * 0.5, y, text, scale, color);
    }

    fn push_centered_text(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        text: &str,
        scale: f32,
        color: [f32; 4],
    ) {
        let width = Self::text_width(text, scale);
        Self::push_text(verts, x - width * 0.5, y, text, scale, color);
    }

    fn text_width(text: &str, scale: f32) -> f32 {
        Self::text_width_with_x_scale(text, scale, HUD_TEXT_X_SCALE)
    }

    fn ui_text_width(layout: HudLayout, text: &str, scale: f32) -> f32 {
        Self::text_width_with_x_scale(text, layout.text_scale(scale), layout.glyph_x_scale)
    }

    fn text_width_with_x_scale(text: &str, scale: f32, x_scale: f32) -> f32 {
        let count = text.chars().count() as f32;
        if count <= 0.0 {
            0.0
        } else {
            let scale = scale.max(HUD_MIN_TEXT_SCALE);
            let pixel_x = 0.007 * scale * x_scale;
            count * pixel_x * 6.25 - pixel_x
        }
    }

    fn glyph_rows(ch: char) -> Option<[u8; 7]> {
        let rows = match ch {
            'A' => [
                0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'B' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
            ],
            'C' => [
                0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
            ],
            'D' => [
                0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
            ],
            'E' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
            ],
            'F' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'G' => [
                0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
            ],
            'H' => [
                0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
            ],
            'I' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
            ],
            'J' => [
                0b00001, 0b00001, 0b00001, 0b00001, 0b10001, 0b10001, 0b01110,
            ],
            'K' => [
                0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
            ],
            'L' => [
                0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
            ],
            'M' => [
                0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
            ],
            'N' => [
                0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
            ],
            'O' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'P' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
            ],
            'Q' => [
                0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
            ],
            'R' => [
                0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
            ],
            'S' => [
                0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
            'T' => [
                0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'U' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
            ],
            'V' => [
                0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
            ],
            'W' => [
                0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
            ],
            'X' => [
                0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
            ],
            'Y' => [
                0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
            ],
            'Z' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
            ],
            '0' => [
                0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
            ],
            '1' => [
                0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
            ],
            '2' => [
                0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
            ],
            '3' => [
                0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
            ],
            '4' => [
                0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
            ],
            '5' => [
                0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
            ],
            '6' => [
                0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
            ],
            '7' => [
                0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
            ],
            '8' => [
                0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
            ],
            '9' => [
                0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
            ],
            '!' => [
                0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
            ],
            '-' => [
                0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
            ],
            '/' => [
                0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
            ],
            _ => return None,
        };
        Some(rows)
    }

    fn push_themed_meter(
        verts: &mut Vec<HudVertex>,
        rect: HudRect,
        ratios: [f32; 2],
        style: HudMeterStyle,
        theme: HudTheme,
    ) {
        let ratio = ratios[0].clamp(0.0, 1.0);
        let trail_ratio = ratios[1].clamp(ratio, 1.0);
        Self::push_rect(verts, rect.x, rect.y, rect.w, rect.h, theme.void);
        Self::push_rect(
            verts,
            rect.x,
            rect.y,
            rect.w * trail_ratio,
            rect.h,
            style.trail,
        );
        Self::push_rect(verts, rect.x, rect.y, rect.w * ratio, rect.h, style.fill);
        Self::push_rect(
            verts,
            rect.x,
            rect.y + rect.h * 0.72,
            rect.w * ratio,
            rect.h * 0.12,
            with_alpha(theme.bone, 0.24),
        );
        Self::push_outline_rect(
            verts,
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            (rect.h * 0.10).clamp(0.0012, 0.003),
            theme.line,
        );
        if style.segments > 1 {
            let tick_w = (rect.h * 0.10).clamp(0.0012, 0.0025);
            for segment in 1..style.segments {
                let x = rect.x + rect.w * segment as f32 / style.segments as f32;
                Self::push_rect(
                    verts,
                    x - tick_w * 0.5,
                    rect.y,
                    tick_w,
                    rect.h,
                    with_alpha(theme.void, 0.80),
                );
            }
        }
    }

    fn push_keycap(
        verts: &mut Vec<HudVertex>,
        layout: HudLayout,
        rect: HudRect,
        label: &str,
        accent: HudColor,
        theme: HudTheme,
    ) {
        Self::push_cut_panel(
            verts,
            rect,
            [layout.sx(0.006), layout.sy(0.006)],
            theme.surface_raised,
            with_alpha(accent, 0.72),
        );
        Self::push_ui_centered_text(
            verts,
            layout,
            rect.center_x(),
            rect.y + rect.h * 0.30,
            label,
            0.42,
            theme.bone,
        );
    }

    fn wrap_ui_lines(
        layout: HudLayout,
        text: &str,
        scale: f32,
        max_width: f32,
        max_lines: usize,
    ) -> Vec<String> {
        if max_lines == 0 || text.is_empty() {
            return Vec::new();
        }
        let mut lines = Vec::with_capacity(max_lines);
        let mut current = String::new();
        let mut truncated = false;

        for word in text.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            if Self::ui_text_width(layout, &candidate, scale) <= max_width || current.is_empty() {
                current = candidate;
                continue;
            }
            if lines.len() + 1 >= max_lines {
                truncated = true;
                break;
            }
            lines.push(std::mem::take(&mut current));
            current = word.to_owned();
        }

        if lines.len() < max_lines && !current.is_empty() {
            lines.push(current);
        }
        if lines.is_empty() {
            lines.push(text.to_owned());
        }

        if let Some(last) = lines.last_mut() {
            while Self::ui_text_width(layout, last, scale) > max_width && last.len() > 3 {
                last.pop();
                truncated = true;
            }
            if truncated {
                while Self::ui_text_width(layout, &format!("{last}..."), scale) > max_width
                    && !last.is_empty()
                {
                    last.pop();
                }
                last.push_str("...");
            }
        }
        lines
    }

    fn push_labeled_value(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        label: &str,
        color: [f32; 4],
        value: u32,
    ) {
        Self::push_text(verts, x, y + 0.007, label, 0.54, [0.85, 0.9, 0.95, 0.62]);
        Self::push_rect(verts, x + 0.105, y + 0.014, 0.018, 0.018, color);
        Self::push_number(verts, x + 0.135, y - 0.001, value, 0.70, color);
    }

    fn push_event_tag(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        label: &str,
        color: [f32; 4],
        pulse: f32,
    ) {
        if pulse <= 0.0 {
            return;
        }
        let text_scale = 0.86;
        let width = Self::text_width(label, text_scale) + 0.055;
        let height = 0.055;
        Self::push_rect(verts, x, y, width, height, [0.0, 0.0, 0.0, 0.45 * pulse]);
        Self::push_outline_rect(
            verts,
            x,
            y,
            width,
            height,
            0.004,
            [color[0], color[1], color[2], 0.42 * pulse],
        );
        Self::push_text(
            verts,
            x + 0.026,
            y + 0.008,
            label,
            text_scale,
            [color[0], color[1], color[2], 0.9 * pulse],
        );
    }

    fn push_edge_vignette(verts: &mut Vec<HudVertex>, color: [f32; 3], alpha: f32) {
        if alpha <= 0.0 {
            return;
        }
        let edge = 0.085;
        let rgba = [color[0], color[1], color[2], alpha];
        Self::push_rect(verts, -1.0, -1.0, edge, 2.0, rgba);
        Self::push_rect(verts, 1.0 - edge, -1.0, edge, 2.0, rgba);
        Self::push_rect(verts, -1.0, -1.0, 2.0, edge, rgba);
        Self::push_rect(verts, -1.0, 1.0 - edge, 2.0, edge, rgba);
    }

    fn push_event_feedback(verts: &mut Vec<HudVertex>, feedback: HudFeedback) {
        let debug_pulse = Self::pulse(feedback.debug_flash, 0.38);
        if debug_pulse > 0.0 {
            Self::push_outline_rect(
                verts,
                -0.985,
                -0.985,
                1.97,
                1.97,
                0.009,
                [0.2, 0.85, 1.0, 0.35 * debug_pulse],
            );
            Self::push_event_tag(
                verts,
                0.62,
                0.79,
                "DEBUG",
                [0.2, 0.85, 1.0, 1.0],
                debug_pulse,
            );
        }

        let spawn_pulse = Self::pulse(feedback.spawn_flash, 0.5);
        if spawn_pulse > 0.0 {
            for i in 0..5 {
                let y = 0.58 - i as f32 * 0.075;
                Self::push_rect(
                    verts,
                    0.89,
                    y,
                    0.055 + (1.0 - spawn_pulse) * 0.045,
                    0.038,
                    [0.35, 1.0, 0.45, 0.55 * spawn_pulse],
                );
            }
            Self::push_event_tag(
                verts,
                0.60,
                0.72,
                "SPAWN",
                [0.35, 1.0, 0.45, 1.0],
                spawn_pulse,
            );
        }

        let reload_pulse = Self::pulse(feedback.reload_flash, 0.65);
        if reload_pulse > 0.0 {
            Self::push_rect(
                verts,
                -1.0,
                0.88,
                2.0,
                0.045,
                [0.25, 0.55, 1.0, 0.32 * reload_pulse],
            );
            Self::push_rect(
                verts,
                -1.0,
                -0.93,
                2.0,
                0.035,
                [0.25, 0.55, 1.0, 0.25 * reload_pulse],
            );
            Self::push_event_tag(
                verts,
                -0.12,
                0.885,
                "RELOAD",
                [0.45, 0.75, 1.0, 1.0],
                reload_pulse,
            );
        }

        let loot_pulse = Self::pulse(feedback.loot_flash, 0.5);
        if loot_pulse > 0.0 {
            for i in 0..6 {
                let x = -0.18 + i as f32 * 0.07;
                Self::push_centered_rect(
                    verts,
                    x,
                    -0.58,
                    0.035,
                    0.035,
                    [1.0, 0.78, 0.18, 0.62 * loot_pulse],
                );
            }
            Self::push_event_tag(
                verts,
                -0.17,
                -0.66,
                "LOOT",
                [1.0, 0.78, 0.18, 1.0],
                loot_pulse,
            );
        }

        let heal_pulse = Self::pulse(feedback.heal_flash, 0.45);
        if heal_pulse > 0.0 {
            let color = [0.25, 1.0, 0.45, 0.58 * heal_pulse];
            Self::push_rect(verts, -0.94, -0.74, 0.11, 0.025, color);
            Self::push_rect(verts, -0.897, -0.785, 0.025, 0.115, color);
            Self::push_event_tag(
                verts,
                -0.82,
                -0.77,
                "HEAL",
                [0.25, 1.0, 0.45, 1.0],
                heal_pulse,
            );
        }
    }

    fn push_debug_dashboard(verts: &mut Vec<HudVertex>, debug: DebugHudState) {
        if !debug.enabled {
            return;
        }

        let x = -0.965;
        let y = 0.50;
        Self::push_rect(
            verts,
            x - 0.015,
            y - 0.04,
            0.59,
            0.48,
            [0.0, 0.0, 0.0, 0.46],
        );
        Self::push_outline_rect(
            verts,
            x - 0.015,
            y - 0.04,
            0.59,
            0.48,
            0.004,
            [0.2, 0.85, 1.0, 0.22],
        );
        Self::push_text(
            verts,
            x + 0.015,
            y + 0.385,
            "DEBUG",
            0.70,
            [0.2, 0.85, 1.0, 0.65],
        );

        Self::push_labeled_value(verts, x, y + 0.29, "FPS", [0.35, 1.0, 0.55, 0.9], debug.fps);
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.29,
            "MS",
            [0.2, 0.85, 1.0, 0.9],
            debug.frame_ms,
        );
        Self::push_labeled_value(
            verts,
            x,
            y + 0.205,
            "ENEMY",
            [1.0, 0.18, 0.12, 0.9],
            debug.enemies,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.205,
            "LOOT",
            [1.0, 0.78, 0.2, 0.9],
            debug.loot,
        );
        Self::push_labeled_value(
            verts,
            x,
            y + 0.12,
            "RES",
            [0.75, 0.75, 0.95, 0.9],
            debug.unsecured_resource,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.12,
            "BANK",
            [0.35, 0.75, 1.0, 0.9],
            debug.banked_resource,
        );
        Self::push_labeled_value(
            verts,
            x,
            y + 0.035,
            "CYCLE",
            [0.8, 0.45, 1.0, 0.9],
            debug.cycle,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.035,
            "PROPS",
            [0.55, 0.9, 0.9, 0.9],
            debug.props,
        );
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
        state: HudFrameState,
    ) {
        let mut verts = std::mem::take(&mut self.vertices);
        verts.clear();
        let layout = HudLayout::new(state.viewport_size);
        let theme = self.theme;
        let hit_flash = state.hit_flash;
        let paused = state.paused;
        let feedback = state.feedback;

        Self::push_player_status_panel(&mut verts, layout, theme, state.player);
        Self::push_ascent_panel(&mut verts, layout, theme, &state.ascent);

        let damage_pulse = Self::pulse(feedback.damage_flash.max(hit_flash), 0.45);
        if damage_pulse > 0.0 {
            Self::push_edge_vignette(&mut verts, [0.9, 0.05, 0.02], 0.24 * damage_pulse);
        }

        let pickup_pulse = Self::pulse(feedback.pickup_flash, 0.35);
        if pickup_pulse > 0.0 {
            Self::push_edge_vignette(&mut verts, [0.9, 0.75, 0.25], 0.075 * pickup_pulse);
        }
        Self::push_event_feedback(&mut verts, feedback);
        Self::push_event_feed(&mut verts, layout, theme, &state.event_feed);
        Self::push_named_notice(&mut verts, layout, theme, &state.named_notice);
        Self::push_named_encounter(&mut verts, layout, theme, &state.named_encounter);
        if !paused {
            Self::push_level_arrival(
                &mut verts,
                layout,
                theme,
                state.level_arrival_ratio,
                &state.level_title,
                &state.level_subtitle,
            );
        }

        if !paused {
            for marker in state.markers.iter().take(MAX_WORLD_MARKERS).rev() {
                Self::push_world_marker(&mut verts, layout, theme, *marker, state.time);
            }

            let shot_pulse = Self::pulse(feedback.shot_flash, 0.08);
            let hit_pulse = Self::pulse(feedback.hit_marker, 0.18);
            let kill_pulse = Self::pulse(feedback.kill_marker, 0.35);
            let blocked_pulse = Self::pulse(feedback.blocked_flash, 0.32);
            let miss_pulse = Self::pulse(feedback.miss_flash, 0.22);
            let ch = CROSSHAIR_SIZE
                * (1.0
                    + shot_pulse * 0.65
                    + hit_pulse * 0.55
                    + blocked_pulse * 0.45
                    + miss_pulse * 0.25);
            let crosshair_color = if kill_pulse > 0.0 {
                theme.gold_bright
            } else if hit_pulse > 0.0 || hit_flash > 0.0 {
                theme.ember
            } else if blocked_pulse > 0.0 {
                theme.cold
            } else if miss_pulse > 0.0 {
                theme.ash
            } else {
                with_alpha(theme.bone, 0.74)
            };
            let arm = 0.020 + ch * 0.30;
            let gap = ch * 0.82;
            let thickness = 0.0035;
            Self::push_rect(
                &mut verts,
                -gap - arm,
                -thickness * 0.5,
                arm,
                thickness,
                crosshair_color,
            );
            Self::push_rect(
                &mut verts,
                gap,
                -thickness * 0.5,
                arm,
                thickness,
                crosshair_color,
            );
            Self::push_rect(
                &mut verts,
                -thickness * 0.5,
                -gap - arm,
                thickness,
                arm,
                crosshair_color,
            );
            Self::push_rect(
                &mut verts,
                -thickness * 0.5,
                gap,
                thickness,
                arm,
                crosshair_color,
            );
            Self::push_centered_rect(
                &mut verts,
                0.0,
                0.0,
                0.005,
                0.005,
                [
                    crosshair_color[0],
                    crosshair_color[1],
                    crosshair_color[2],
                    0.82,
                ],
            );

            if blocked_pulse > 0.0 {
                let color = [0.45, 0.78, 1.0, 0.78 * blocked_pulse];
                let ring = 0.072 + (1.0 - blocked_pulse) * 0.030;
                Self::push_outline_rect(
                    &mut verts,
                    -ring,
                    -ring,
                    ring * 2.0,
                    ring * 2.0,
                    0.006,
                    color,
                );
                Self::push_centered_text(
                    &mut verts,
                    0.0,
                    0.090,
                    "BLOCK",
                    0.56,
                    [0.58, 0.86, 1.0, 0.78 * blocked_pulse],
                );
            }

            if miss_pulse > 0.0 && blocked_pulse <= 0.0 && hit_pulse <= 0.0 {
                Self::push_centered_text(
                    &mut verts,
                    0.0,
                    0.080,
                    "MISS",
                    0.48,
                    [0.74, 0.78, 0.84, 0.62 * miss_pulse],
                );
            }

            if hit_pulse > 0.0 {
                let color = [1.0, 0.18, 0.1, 0.85 * hit_pulse];
                Self::push_rect(&mut verts, -0.065, -0.004, 0.032, 0.008, color);
                Self::push_rect(&mut verts, 0.033, -0.004, 0.032, 0.008, color);
                Self::push_rect(&mut verts, -0.004, -0.065, 0.008, 0.032, color);
                Self::push_rect(&mut verts, -0.004, 0.033, 0.008, 0.032, color);
                Self::push_centered_text(
                    &mut verts,
                    0.0,
                    0.085,
                    "HIT",
                    0.58,
                    [1.0, 0.28, 0.14, 0.78 * hit_pulse],
                );
            }

            if kill_pulse > 0.0 {
                let color = [1.0, 0.72, 0.18, 0.9 * kill_pulse];
                let size = 0.09 + (1.0 - kill_pulse) * 0.08;
                Self::push_rect(&mut verts, -size, -0.006, size * 2.0, 0.012, color);
                Self::push_rect(&mut verts, -0.006, -size, 0.012, size * 2.0, color);
                Self::push_centered_text(
                    &mut verts,
                    0.0,
                    0.135,
                    "KILL",
                    0.68,
                    [1.0, 0.76, 0.18, 0.88 * kill_pulse],
                );
            }

            Self::add_center_particles(
                &mut verts,
                feedback.hit_marker,
                0.18,
                8,
                0.18,
                0.012,
                [1.0, 0.22, 0.12, 0.75],
            );
            Self::add_center_particles(
                &mut verts,
                feedback.kill_marker,
                0.35,
                12,
                0.28,
                0.014,
                [1.0, 0.72, 0.18, 0.85],
            );
            Self::add_center_particles(
                &mut verts,
                feedback.pickup_flash,
                0.35,
                10,
                0.23,
                0.013,
                [0.9, 0.78, 0.26, 0.7],
            );

            if state.dialogue.line.is_empty() {
                Self::push_interaction_prompt(&mut verts, layout, theme, &state.interaction_prompt);
            } else {
                Self::push_dialogue(&mut verts, layout, theme, &state.dialogue);
            }
        }

        Self::push_anchor_rite(&mut verts, layout, theme, &state.anchor_rite);

        if paused {
            Self::push_pause_overlay(&mut verts, layout, theme);
        }

        if state.dead {
            Self::push_death_overlay(&mut verts, layout, theme, state.respawn_remaining);
        }

        Self::push_debug_dashboard(&mut verts, state.debug);

        self.num_indices = (verts.len() / 4 * 6) as u32;
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        self.vertices = verts;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_wrap_respects_line_and_width_limits() {
        let layout = HudLayout::new([1920, 1080]);
        let max_width = layout.sx(0.34);
        let lines = HudSystem::wrap_ui_lines(
            layout,
            "Pale waystones climb toward the Anchor beyond the ash",
            0.48,
            max_width,
            2,
        );

        assert_eq!(lines.len(), 2);
        assert!(lines
            .iter()
            .all(|line| HudSystem::ui_text_width(layout, line, 0.48) <= max_width));
        assert!(lines[1].ends_with("..."));
    }

    #[test]
    fn dialogue_wrap_accepts_an_explicit_zero_line_budget() {
        let layout = HudLayout::new([800, 600]);
        assert!(HudSystem::wrap_ui_lines(layout, "Hidden", 0.48, 0.4, 0).is_empty());
    }

    #[test]
    fn first_ascent_names_remain_complete_at_supported_viewports() {
        let encounter = "The Ash-Warden, Bearer of the Last Chain";
        let relic = "Debt of the Last Keeper";

        for viewport in [[800, 600], [1280, 720], [1920, 1080], [2560, 1080]] {
            let layout = HudLayout::new(viewport);
            assert_eq!(
                HudSystem::wrap_ui_lines(layout, encounter, 0.34, layout.boss.w, 1),
                vec![encounter.to_string()],
                "{viewport:?}"
            );
            assert_eq!(
                HudSystem::wrap_ui_lines(layout, relic, 0.52, layout.objective.w, 1),
                vec![relic.to_string()],
                "{viewport:?}"
            );
        }
    }
}

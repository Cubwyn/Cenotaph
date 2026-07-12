// src/systems/render/hud.rs
// Rectangle-only HUD overlay using a dedicated 2D pipeline.
// Draws health/stamina, crosshair feedback, projected debug markers, and a
// compact numeric debug dashboard without requiring a font renderer.

use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct HudVertex {
    position: [f32; 2],
    color: [f32; 4],
}

const CROSSHAIR_SIZE: f32 = 0.015;
const MAX_HUD_QUADS: usize = 8192;
const MAX_WORLD_MARKERS: usize = 64;
const MAX_EVENT_FEED_ITEMS: usize = 5;
const HUD_TEXT_X_SCALE: f32 = 0.58;
const HUD_MIN_TEXT_SCALE: f32 = 0.46;

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
    pub pickup_flash: f32,
    pub damage_flash: f32,
    pub debug_flash: f32,
    pub spawn_flash: f32,
    pub reload_flash: f32,
    pub loot_flash: f32,
    pub heal_flash: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DebugHudState {
    pub enabled: bool,
    pub health_current: u32,
    pub health_max: u32,
    pub stamina_current: u32,
    pub stamina_max: u32,
    pub enemies: u32,
    pub loot: u32,
    pub unsecured_resource: u32,
    pub banked_resource: u32,
    pub cycle: u32,
    pub props: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AscentHudState {
    pub cycle: u32,
    pub cycle_modifier: String,
    pub relic_name: String,
    pub relic_family: String,
    pub relic_rarity: String,
    pub owned_relics: u32,
    pub unsecured_resource: u32,
    pub banked_resource: u32,
}

#[derive(Debug, Clone, Default)]
pub struct HudFrameState {
    pub health_ratio: f32,
    pub stamina_ratio: f32,
    pub dash_cooldown_ratio: f32,
    pub hit_flash: f32,
    pub paused: bool,
    pub dead: bool,
    pub respawn_remaining: f32,
    pub time: f32,
    pub feedback: HudFeedback,
    pub debug: DebugHudState,
    pub ascent: AscentHudState,
    pub markers: Vec<HudWorldMarker>,
    pub event_feed: Vec<HudFeedEvent>,
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
            num_indices: 0,
        }
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
        let scale = scale.max(HUD_MIN_TEXT_SCALE);
        let pixel_y = 0.006 * scale;
        let pixel_x = pixel_y * HUD_TEXT_X_SCALE;
        let advance = pixel_x * 6.35;

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
                        pixel_x * 0.92,
                        pixel_y * 0.92,
                        color,
                    );
                }
            }
        }
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
        let count = text.chars().count() as f32;
        if count <= 0.0 {
            0.0
        } else {
            let scale = scale.max(HUD_MIN_TEXT_SCALE);
            let pixel_x = 0.006 * scale * HUD_TEXT_X_SCALE;
            count * pixel_x * 6.35 - pixel_x
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

    fn push_meter_row(
        verts: &mut Vec<HudVertex>,
        x: f32,
        y: f32,
        color: [f32; 4],
        ratio: f32,
        value: u32,
        max_value: u32,
    ) {
        let ratio = ratio.clamp(0.0, 1.0);
        Self::push_rect(verts, x, y + 0.012, 0.02, 0.02, color);
        Self::push_rect(
            verts,
            x + 0.035,
            y + 0.014,
            0.19,
            0.016,
            [0.02, 0.02, 0.02, 0.72],
        );
        Self::push_rect(verts, x + 0.035, y + 0.014, 0.19 * ratio, 0.016, color);
        Self::push_outline_rect(
            verts,
            x + 0.035,
            y + 0.014,
            0.19,
            0.016,
            0.002,
            [1.0, 1.0, 1.0, 0.22],
        );
        Self::push_number(verts, x + 0.25, y - 0.001, value, 0.78, color);
        Self::push_number(
            verts,
            x + 0.345,
            y + 0.004,
            max_value,
            0.52,
            [color[0], color[1], color[2], 0.58],
        );
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

    fn push_event_feed(verts: &mut Vec<HudVertex>, events: &[HudFeedEvent]) {
        let visible_count = events.len().min(MAX_EVENT_FEED_ITEMS);
        if visible_count == 0 {
            return;
        }

        let x = 0.545;
        let top = 0.36;
        let row_h = 0.058;
        let panel_h = 0.082 + visible_count as f32 * row_h;
        let panel_y = top - panel_h;

        Self::push_rect(verts, x, panel_y, 0.43, panel_h, [0.0, 0.0, 0.0, 0.42]);
        Self::push_outline_rect(
            verts,
            x,
            panel_y,
            0.43,
            panel_h,
            0.003,
            [0.85, 0.9, 0.95, 0.16],
        );
        Self::push_text(
            verts,
            x + 0.020,
            top - 0.050,
            "EVENTS",
            0.48,
            [0.85, 0.9, 0.95, 0.48],
        );

        for (index, event) in events.iter().take(MAX_EVENT_FEED_ITEMS).enumerate() {
            let y = top - 0.105 - index as f32 * row_h;
            let ratio = event.ratio.clamp(0.0, 1.0);
            let alpha = 0.25 + ratio * 0.65;
            let color = [
                event.color[0],
                event.color[1],
                event.color[2],
                event.color[3] * alpha,
            ];

            Self::push_rect(verts, x + 0.019, y + 0.013, 0.016, 0.016, color);
            Self::push_text(
                verts,
                x + 0.048,
                y,
                event.label,
                0.52,
                [color[0], color[1], color[2], color[3].max(0.30)],
            );

            if event.has_value {
                Self::push_number(
                    verts,
                    x + 0.300,
                    y - 0.001,
                    event.value,
                    0.54,
                    [color[0], color[1], color[2], color[3].max(0.34)],
                );
            }
        }
    }

    fn push_world_marker(verts: &mut Vec<HudVertex>, marker: HudWorldMarker, time: f32) {
        let [x, y] = marker.screen_pos;
        let pulse = 0.5 + 0.5 * (time * 4.2).sin();
        let ratio = marker.ratio.clamp(0.0, 1.0);

        match marker.kind {
            HudMarkerKind::Enemy => {
                let size = 0.026 + (1.0 - ratio) * 0.009;
                let state_ratio = marker.state_ratio.clamp(0.0, 1.0);
                let (main_color, outline_color, state_label) = match marker.state {
                    HudMarkerState::Neutral => {
                        ([0.82, 0.16, 0.12, 0.68], [1.0, 0.22, 0.16, 0.18], "")
                    }
                    HudMarkerState::Aggro => (
                        [1.0, 0.18, 0.10, 0.84],
                        [1.0, 0.10, 0.06, 0.48 + state_ratio * 0.22],
                        "AGGRO",
                    ),
                    HudMarkerState::Windup => (
                        [1.0, 0.48, 0.08, 0.90],
                        [1.0, 0.72, 0.12, 0.58 + pulse * 0.25],
                        "ATK",
                    ),
                    HudMarkerState::Staggered => (
                        [0.24, 0.72, 1.0, 0.86],
                        [0.38, 0.92, 1.0, 0.52 + state_ratio * 0.18],
                        "STUN",
                    ),
                };

                let outline_size = size + 0.020 + pulse * 0.006;
                if marker.state != HudMarkerState::Neutral {
                    Self::push_outline_rect(
                        verts,
                        x - outline_size * 0.5,
                        y - outline_size * 0.5,
                        outline_size,
                        outline_size,
                        0.0045,
                        outline_color,
                    );
                }

                Self::push_centered_rect(verts, x, y, size, size, main_color);
                Self::push_centered_text(verts, x, y + 0.018, "E", 0.48, [1.0, 0.88, 0.82, 0.82]);

                match marker.state {
                    HudMarkerState::Aggro => {}
                    HudMarkerState::Windup => {
                        Self::push_rect(
                            verts,
                            x - 0.046,
                            y + 0.042,
                            0.092,
                            0.008,
                            [0.02, 0.02, 0.02, 0.72],
                        );
                        Self::push_rect(
                            verts,
                            x - 0.046,
                            y + 0.042,
                            0.092 * state_ratio,
                            0.008,
                            [1.0, 0.76, 0.12, 0.92],
                        );
                    }
                    HudMarkerState::Staggered => {
                        let stun_alpha = 0.46 + state_ratio * 0.26;
                        Self::push_centered_rect(
                            verts,
                            x - 0.020,
                            y + 0.042,
                            0.025,
                            0.006,
                            [0.38, 0.92, 1.0, stun_alpha],
                        );
                        Self::push_centered_rect(
                            verts,
                            x + 0.020,
                            y + 0.042,
                            0.025,
                            0.006,
                            [0.38, 0.92, 1.0, stun_alpha],
                        );
                    }
                    HudMarkerState::Neutral => {}
                }

                if !state_label.is_empty() {
                    Self::push_centered_text(
                        verts,
                        x,
                        y + 0.058,
                        state_label,
                        0.30,
                        [outline_color[0], outline_color[1], outline_color[2], 0.70],
                    );
                }

                Self::push_rect(
                    verts,
                    x - 0.038,
                    y - 0.045,
                    0.076,
                    0.008,
                    [0.02, 0.02, 0.02, 0.70],
                );
                Self::push_rect(
                    verts,
                    x - 0.038,
                    y - 0.045,
                    0.076 * ratio,
                    0.008,
                    [1.0, 0.2 + ratio * 0.45, 0.12, 0.85],
                );
                Self::push_number(
                    verts,
                    x - 0.022,
                    y - 0.085,
                    marker.distance_m,
                    0.42,
                    [1.0, 0.72, 0.60, 0.66],
                );
            }
            HudMarkerKind::Loot => {
                let size = 0.024 + pulse * 0.006;
                Self::push_centered_rect(verts, x, y, size, size, [1.0, 0.78, 0.2, 0.78]);
                Self::push_centered_text(verts, x, y - 0.014, "L", 0.45, [0.18, 0.12, 0.0, 0.85]);
                Self::push_centered_rect(verts, x, y + 0.036, 0.012, 0.03, [1.0, 0.9, 0.35, 0.55]);
                Self::push_number(
                    verts,
                    x - 0.018,
                    y - 0.065,
                    marker.distance_m,
                    0.38,
                    [1.0, 0.86, 0.35, 0.58],
                );
            }
            HudMarkerKind::Anchor => {
                Self::push_outline_rect(
                    verts,
                    x - 0.031,
                    y - 0.031,
                    0.062,
                    0.062,
                    0.006,
                    [0.35, 0.75, 1.0, 0.70],
                );
                Self::push_centered_rect(verts, x, y, 0.018, 0.018, [0.55, 0.95, 1.0, 0.65]);
                Self::push_centered_text(verts, x, y - 0.012, "A", 0.42, [0.8, 1.0, 1.0, 0.82]);
                Self::push_number(
                    verts,
                    x - 0.018,
                    y - 0.068,
                    marker.distance_m,
                    0.38,
                    [0.65, 0.9, 1.0, 0.58],
                );
            }
            HudMarkerKind::Hazard => {
                Self::push_centered_rect(
                    verts,
                    x,
                    y + 0.012,
                    0.016,
                    0.052,
                    [1.0, 0.08, 0.02, 0.76],
                );
                Self::push_centered_rect(
                    verts,
                    x,
                    y - 0.033,
                    0.018,
                    0.018,
                    [1.0, 0.08, 0.02, 0.76],
                );
                Self::push_centered_text(verts, x, y + 0.006, "!", 0.48, [1.0, 0.9, 0.8, 0.9]);
                Self::push_number(
                    verts,
                    x - 0.018,
                    y - 0.082,
                    marker.distance_m,
                    0.38,
                    [1.0, 0.35, 0.22, 0.62],
                );
            }
        }
    }

    fn push_debug_dashboard(
        verts: &mut Vec<HudVertex>,
        debug: DebugHudState,
        health_ratio: f32,
        stamina_ratio: f32,
    ) {
        if !debug.enabled {
            return;
        }

        let x = -0.965;
        let y = 0.50;
        let meter_x = x + 0.105;
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

        Self::push_text(verts, x, y + 0.304, "HP", 0.58, [0.85, 0.9, 0.95, 0.62]);
        Self::push_meter_row(
            verts,
            meter_x,
            y + 0.29,
            [0.3, 0.9, 0.3, 0.92],
            health_ratio,
            debug.health_current,
            debug.health_max,
        );
        Self::push_text(verts, x, y + 0.229, "STA", 0.58, [0.85, 0.9, 0.95, 0.62]);
        Self::push_meter_row(
            verts,
            meter_x,
            y + 0.215,
            [0.95, 0.82, 0.24, 0.90],
            stamina_ratio,
            debug.stamina_current,
            debug.stamina_max,
        );
        Self::push_labeled_value(
            verts,
            x,
            y + 0.13,
            "ENEMY",
            [1.0, 0.18, 0.12, 0.9],
            debug.enemies,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.13,
            "LOOT",
            [1.0, 0.78, 0.2, 0.9],
            debug.loot,
        );
        Self::push_labeled_value(
            verts,
            x,
            y + 0.055,
            "RES",
            [0.75, 0.75, 0.95, 0.9],
            debug.unsecured_resource,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y + 0.055,
            "BANK",
            [0.35, 0.75, 1.0, 0.9],
            debug.banked_resource,
        );
        Self::push_labeled_value(
            verts,
            x,
            y - 0.02,
            "CYCLE",
            [0.8, 0.45, 1.0, 0.9],
            debug.cycle,
        );
        Self::push_labeled_value(
            verts,
            x + 0.29,
            y - 0.02,
            "PROPS",
            [0.55, 0.9, 0.9, 0.9],
            debug.props,
        );
    }

    fn push_ascent_panel(verts: &mut Vec<HudVertex>, ascent: &AscentHudState) {
        let panel_x = -0.34;
        let panel_y = 0.765;
        let panel_w = 0.68;
        let panel_h = 0.205;

        Self::push_rect(
            verts,
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            [0.0, 0.0, 0.0, 0.42],
        );
        Self::push_outline_rect(
            verts,
            panel_x,
            panel_y,
            panel_w,
            panel_h,
            0.0035,
            [0.75, 0.82, 1.0, 0.18],
        );

        Self::push_text(
            verts,
            panel_x + 0.024,
            panel_y + 0.154,
            "CYCLE",
            0.55,
            [0.86, 0.90, 1.0, 0.55],
        );
        Self::push_number(
            verts,
            panel_x + 0.130,
            panel_y + 0.145,
            ascent.cycle,
            0.70,
            [0.82, 0.50, 1.0, 0.88],
        );
        Self::push_text(
            verts,
            panel_x + 0.225,
            panel_y + 0.154,
            &ascent.cycle_modifier,
            0.55,
            [0.82, 0.50, 1.0, 0.76],
        );

        Self::push_text(
            verts,
            panel_x + 0.024,
            panel_y + 0.090,
            "RELIC",
            0.55,
            [0.86, 0.90, 1.0, 0.55],
        );
        Self::push_text(
            verts,
            panel_x + 0.130,
            panel_y + 0.090,
            &ascent.relic_name,
            0.60,
            [1.0, 0.78, 0.24, 0.88],
        );
        Self::push_text(
            verts,
            panel_x + 0.130,
            panel_y + 0.040,
            &ascent.relic_family,
            0.44,
            [0.80, 0.88, 1.0, 0.56],
        );
        Self::push_text(
            verts,
            panel_x + 0.320,
            panel_y + 0.040,
            &ascent.relic_rarity,
            0.44,
            [1.0, 0.80, 0.38, 0.58],
        );

        Self::push_text(
            verts,
            panel_x + 0.485,
            panel_y + 0.090,
            "OWN",
            0.48,
            [0.86, 0.90, 1.0, 0.50],
        );
        Self::push_number(
            verts,
            panel_x + 0.565,
            panel_y + 0.080,
            ascent.owned_relics,
            0.62,
            [1.0, 0.78, 0.24, 0.78],
        );

        Self::push_text(
            verts,
            panel_x + 0.485,
            panel_y + 0.040,
            "RES",
            0.44,
            [0.86, 0.90, 1.0, 0.50],
        );
        Self::push_number(
            verts,
            panel_x + 0.555,
            panel_y + 0.032,
            ascent.unsecured_resource,
            0.50,
            [0.78, 0.78, 1.0, 0.72],
        );
        Self::push_text(
            verts,
            panel_x + 0.485,
            panel_y + 0.002,
            "BANK",
            0.40,
            [0.86, 0.90, 1.0, 0.44],
        );
        Self::push_number(
            verts,
            panel_x + 0.565,
            panel_y - 0.005,
            ascent.banked_resource,
            0.46,
            [0.35, 0.78, 1.0, 0.70],
        );
    }

    fn push_player_status_panel(
        verts: &mut Vec<HudVertex>,
        debug: DebugHudState,
        health_ratio: f32,
        stamina_ratio: f32,
        dash_cooldown_ratio: f32,
        health_color: [f32; 4],
    ) {
        let panel_x = -0.42;
        let panel_y = -0.965;
        Self::push_rect(verts, panel_x, panel_y, 0.84, 0.205, [0.0, 0.0, 0.0, 0.48]);
        Self::push_outline_rect(
            verts,
            panel_x,
            panel_y,
            0.84,
            0.205,
            0.004,
            [0.95, 0.95, 0.95, 0.16],
        );
        Self::push_text(
            verts,
            panel_x + 0.025,
            panel_y + 0.158,
            "STATUS",
            0.58,
            [0.9, 0.93, 0.95, 0.62],
        );

        let label_x = panel_x + 0.035;
        let meter_x = panel_x + 0.11;
        Self::push_text(
            verts,
            label_x,
            panel_y + 0.092,
            "HP",
            0.60,
            [0.9, 0.93, 0.95, 0.70],
        );
        Self::push_meter_row(
            verts,
            meter_x,
            panel_y + 0.078,
            health_color,
            health_ratio,
            debug.health_current,
            debug.health_max,
        );
        Self::push_text(
            verts,
            label_x,
            panel_y + 0.027,
            "STA",
            0.60,
            [0.9, 0.93, 0.95, 0.70],
        );
        Self::push_meter_row(
            verts,
            meter_x,
            panel_y + 0.013,
            [0.95, 0.82, 0.24, 0.90],
            stamina_ratio,
            debug.stamina_current,
            debug.stamina_max,
        );

        let dash_x = panel_x + 0.58;
        let dash_y = panel_y + 0.035;
        let dash_ready = dash_cooldown_ratio <= 0.01;
        let dash_color = if dash_ready {
            [0.35, 1.0, 0.55, 0.88]
        } else {
            [1.0, 0.48, 0.18, 0.88]
        };
        Self::push_text(
            verts,
            dash_x,
            dash_y + 0.096,
            "DASH",
            0.58,
            [0.9, 0.93, 0.95, 0.62],
        );
        Self::push_rect(
            verts,
            dash_x,
            dash_y + 0.055,
            0.20,
            0.022,
            [0.03, 0.03, 0.03, 0.68],
        );
        let dash_fill = if dash_ready {
            0.20
        } else {
            0.20 * (1.0 - dash_cooldown_ratio.clamp(0.0, 1.0))
        };
        Self::push_rect(verts, dash_x, dash_y + 0.055, dash_fill, 0.022, dash_color);
        Self::push_outline_rect(
            verts,
            dash_x,
            dash_y + 0.055,
            0.20,
            0.022,
            0.003,
            [1.0, 1.0, 1.0, 0.18],
        );
        Self::push_text(
            verts,
            dash_x,
            dash_y,
            if dash_ready { "READY" } else { "WAIT" },
            0.54,
            dash_color,
        );
    }

    fn push_death_overlay(verts: &mut Vec<HudVertex>, respawn_remaining: f32) {
        Self::push_rect(verts, -1.0, -1.0, 2.0, 2.0, [0.08, 0.0, 0.0, 0.50]);
        Self::push_centered_text(verts, 0.0, 0.14, "DEFEATED", 1.10, [1.0, 0.18, 0.10, 0.88]);
        Self::push_centered_text(verts, 0.0, 0.04, "RESPAWN", 0.72, [1.0, 0.72, 0.56, 0.76]);
        Self::push_number(
            verts,
            -0.018,
            -0.055,
            respawn_remaining.ceil().max(0.0) as u32,
            1.15,
            [1.0, 0.72, 0.56, 0.85],
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
        let mut verts: Vec<HudVertex> = Vec::with_capacity(MAX_HUD_QUADS * 4);
        let health_ratio = state.health_ratio;
        let stamina_ratio = state.stamina_ratio;
        let hit_flash = state.hit_flash;
        let paused = state.paused;
        let feedback = state.feedback;

        let health_color = if health_ratio > 0.5 {
            [0.2, 0.8, 0.2, 0.9]
        } else if health_ratio > 0.25 {
            [0.9, 0.7, 0.1, 0.9]
        } else {
            [0.9, 0.2, 0.1, 0.9]
        };

        Self::push_player_status_panel(
            &mut verts,
            state.debug,
            health_ratio,
            stamina_ratio,
            state.dash_cooldown_ratio,
            health_color,
        );
        Self::push_ascent_panel(&mut verts, &state.ascent);

        let damage_pulse = Self::pulse(feedback.damage_flash.max(hit_flash), 0.45);
        if damage_pulse > 0.0 {
            Self::push_rect(
                &mut verts,
                -1.0,
                -1.0,
                2.0,
                2.0,
                [0.9, 0.05, 0.02, 0.22 * damage_pulse],
            );
            Self::push_event_tag(
                &mut verts,
                -0.15,
                -0.78,
                "DAMAGE",
                [1.0, 0.16, 0.08, 1.0],
                damage_pulse,
            );
        }

        let pickup_pulse = Self::pulse(feedback.pickup_flash, 0.35);
        if pickup_pulse > 0.0 {
            Self::push_rect(
                &mut verts,
                -1.0,
                -1.0,
                2.0,
                2.0,
                [0.9, 0.75, 0.25, 0.10 * pickup_pulse],
            );
            Self::push_event_tag(
                &mut verts,
                -0.14,
                -0.70,
                "PICKUP",
                [1.0, 0.78, 0.18, 1.0],
                pickup_pulse,
            );
        }
        Self::push_event_feedback(&mut verts, feedback);
        Self::push_event_feed(&mut verts, &state.event_feed);

        if paused {
            // Dim the world and show a simple pause emblem without requiring text rendering.
            Self::push_rect(&mut verts, -1.0, -1.0, 2.0, 2.0, [0.0, 0.0, 0.0, 0.45]);
            let pause_color = [0.95, 0.95, 0.95, 0.90];
            Self::push_rect(&mut verts, -0.085, -0.14, 0.045, 0.28, pause_color);
            Self::push_rect(&mut verts, 0.040, -0.14, 0.045, 0.28, pause_color);
            Self::push_rect(
                &mut verts,
                -0.13,
                0.18,
                0.26,
                0.02,
                [0.95, 0.95, 0.95, 0.55],
            );
            Self::push_centered_text(
                &mut verts,
                0.0,
                -0.25,
                "PAUSED",
                0.82,
                [0.95, 0.95, 0.95, 0.72],
            );
        } else {
            for marker in state.markers.iter().take(MAX_WORLD_MARKERS).rev() {
                Self::push_world_marker(&mut verts, *marker, state.time);
            }

            let shot_pulse = Self::pulse(feedback.shot_flash, 0.08);
            let hit_pulse = Self::pulse(feedback.hit_marker, 0.18);
            let kill_pulse = Self::pulse(feedback.kill_marker, 0.35);
            let ch = CROSSHAIR_SIZE * (1.0 + shot_pulse * 0.65 + hit_pulse * 0.55);
            let crosshair_color = if kill_pulse > 0.0 {
                [1.0, 0.65, 0.15, 0.95]
            } else if hit_pulse > 0.0 || hit_flash > 0.0 {
                [1.0, 0.3, 0.2, 0.9]
            } else {
                [1.0, 1.0, 1.0, 0.7]
            };
            Self::push_rect(&mut verts, -ch, -ch, ch * 2.0, ch * 2.0, crosshair_color);

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
        }

        if state.dead {
            Self::push_death_overlay(&mut verts, state.respawn_remaining);
        }

        Self::push_debug_dashboard(&mut verts, state.debug, health_ratio, stamina_ratio);

        self.num_indices = (verts.len() / 4 * 6) as u32;
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&verts));
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

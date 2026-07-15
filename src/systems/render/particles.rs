use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::data::world::level::{AtmosphereData, ParticlePreset};

pub const MAX_AMBIENT_PARTICLES: usize = 512;
const MAX_EFFECT_PARTICLES: usize = 256;
const MAX_RENDER_PARTICLES: usize = MAX_AMBIENT_PARTICLES + MAX_EFFECT_PARTICLES;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleVertex {
    position: [f32; 3],
}

impl ParticleVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ParticleInstanceRaw {
    position_size: [f32; 4],
    color: [f32; 4],
    shape: [f32; 4],
}

impl ParticleInstanceRaw {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

#[derive(Clone, Copy)]
struct Particle {
    offset: Vec3,
    variation: f32,
    phase: f32,
    brightness: f32,
}

#[derive(Clone, Copy)]
struct EffectParticle {
    position: Vec3,
    velocity: Vec3,
    color: [f32; 4],
    shape: [f32; 3],
    size: f32,
    age: f32,
    lifetime: f32,
    gravity: f32,
    drag: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleBurst {
    Muzzle,
    Hit,
    Kill,
    Blocked,
    Pickup,
    Damage,
    Dash,
    Land,
}

pub struct ParticleSystem {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    particles: Vec<Particle>,
    effects: Vec<EffectParticle>,
    instances: Vec<ParticleInstanceRaw>,
    active_count: u32,
    vertex_count: u32,
    elapsed: f32,
    burst_sequence: u64,
}

impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Atmosphere Particle Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("particle.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Atmosphere Particle Pipeline Layout"),
            bind_group_layouts: &[camera_layout],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Atmosphere Particle Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[ParticleVertex::desc(), ParticleInstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertices = octahedron_vertices();
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Atmosphere Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Atmosphere Particle Instance Buffer"),
            size: (MAX_RENDER_PARTICLES * std::mem::size_of::<ParticleInstanceRaw>())
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            instance_buffer,
            particles: Vec::new(),
            effects: Vec::with_capacity(MAX_EFFECT_PARTICLES),
            instances: Vec::with_capacity(MAX_RENDER_PARTICLES),
            active_count: 0,
            vertex_count: vertices.len() as u32,
            elapsed: 0.0,
            burst_sequence: 0,
        }
    }

    pub fn configure(&mut self, queue: &wgpu::Queue, atmosphere: &AtmosphereData, center: Vec3) {
        let count = atmosphere.particle_count.min(MAX_AMBIENT_PARTICLES as u32) as usize;
        self.particles.clear();
        self.particles.reserve(count);
        self.elapsed = 0.0;
        for index in 0..count {
            self.particles.push(Particle {
                offset: Vec3::new(
                    signed_seed(index, 0) * atmosphere.particle_radius,
                    signed_seed(index, 1) * atmosphere.particle_height * 0.5,
                    signed_seed(index, 2) * atmosphere.particle_radius,
                ),
                variation: 0.65 + seed_unit(index, 3) * 0.7,
                phase: seed_unit(index, 4) * std::f32::consts::TAU,
                brightness: 0.78 + seed_unit(index, 5) * 0.42,
            });
        }
        self.update(queue, atmosphere, center, 0.0);
    }

    pub fn update(
        &mut self,
        queue: &wgpu::Queue,
        atmosphere: &AtmosphereData,
        center: Vec3,
        dt: f32,
    ) {
        let dt = dt.clamp(0.0, 1.0 / 20.0);
        self.elapsed += dt;
        self.instances.clear();

        if atmosphere.particle_preset != ParticlePreset::None && atmosphere.particle_count > 0 {
            let target_count = atmosphere.particle_count.min(MAX_AMBIENT_PARTICLES as u32) as usize;
            if self.particles.len() != target_count {
                self.configure(queue, atmosphere, center);
                return;
            }

            let wind = Vec3::from_array(atmosphere.wind);
            let radius = atmosphere.particle_radius.max(2.0);
            let half_height = (atmosphere.particle_height * 0.5).max(1.0);

            for particle in &mut self.particles {
                let vertical = match atmosphere.particle_preset {
                    ParticlePreset::Ashfall => -atmosphere.particle_speed,
                    ParticlePreset::Embers => atmosphere.particle_speed,
                    ParticlePreset::Dust => {
                        (self.elapsed * 0.7 + particle.phase).sin()
                            * atmosphere.particle_speed
                            * 0.18
                    }
                    ParticlePreset::None => 0.0,
                };
                let gust = 0.72 + 0.28 * (self.elapsed * 0.23 + particle.phase).sin();
                let swirl = Vec3::new(
                    (self.elapsed * 0.45 + particle.phase).sin(),
                    0.0,
                    (self.elapsed * 0.37 + particle.phase).cos(),
                ) * atmosphere.particle_speed
                    * 0.15;
                particle.offset +=
                    (wind * gust + swirl + Vec3::Y * vertical) * dt * particle.variation;
                particle.offset.x = wrap_axis(particle.offset.x, radius);
                particle.offset.y = wrap_axis(particle.offset.y, half_height);
                particle.offset.z = wrap_axis(particle.offset.z, radius);

                let flicker = match atmosphere.particle_preset {
                    ParticlePreset::Embers => {
                        0.68 + 0.32 * (self.elapsed * 7.0 + particle.phase).sin().abs()
                    }
                    _ => 1.0,
                };
                let horizontal_edge = particle.offset.x.abs().max(particle.offset.z.abs());
                let edge_fade =
                    ((radius - horizontal_edge) / (radius * 0.14).max(0.1)).clamp(0.0, 1.0);
                let near_fade = ((particle.offset.length() - 0.55) / 1.25).clamp(0.0, 1.0);
                let alpha = atmosphere.particle_opacity
                    * flicker
                    * particle.brightness
                    * edge_fade
                    * near_fade;
                if alpha <= 0.002 {
                    continue;
                }
                let size = atmosphere.particle_size * particle.variation * flicker;
                let shape = match atmosphere.particle_preset {
                    ParticlePreset::Ashfall => [0.34, 1.15, 0.34, flicker],
                    ParticlePreset::Embers => [0.28, 1.55, 0.28, flicker],
                    ParticlePreset::Dust => [1.35, 0.30, 1.35, flicker],
                    ParticlePreset::None => [1.0; 4],
                };
                self.instances.push(ParticleInstanceRaw {
                    position_size: [
                        center.x + particle.offset.x,
                        center.y + particle.offset.y,
                        center.z + particle.offset.z,
                        size,
                    ],
                    color: [
                        atmosphere.particle_color[0] * particle.brightness,
                        atmosphere.particle_color[1] * particle.brightness,
                        atmosphere.particle_color[2] * particle.brightness,
                        alpha,
                    ],
                    shape,
                });
            }
        }

        for effect in &mut self.effects {
            effect.age += dt;
            effect.velocity.y += effect.gravity * dt;
            effect.velocity *= (-effect.drag * dt).exp();
            effect.position += effect.velocity * dt;

            let progress = (effect.age / effect.lifetime.max(0.001)).clamp(0.0, 1.0);
            let remaining = 1.0 - progress;
            self.instances.push(ParticleInstanceRaw {
                position_size: [
                    effect.position.x,
                    effect.position.y,
                    effect.position.z,
                    effect.size * (0.82 + progress * 0.48),
                ],
                color: [
                    effect.color[0],
                    effect.color[1],
                    effect.color[2],
                    effect.color[3] * remaining * remaining,
                ],
                shape: [effect.shape[0], effect.shape[1], effect.shape[2], remaining],
            });
        }
        self.effects.retain(|effect| effect.age < effect.lifetime);

        self.active_count = self.instances.len().min(MAX_RENDER_PARTICLES) as u32;
        if !self.instances.is_empty() {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&self.instances[..self.active_count as usize]),
            );
        }
    }

    pub fn spawn_burst(&mut self, kind: ParticleBurst, origin: Vec3, direction: Vec3) {
        let (count, color, speed, spread, size, lifetime, gravity, drag, shape) = match kind {
            ParticleBurst::Muzzle => (
                8,
                [1.35, 0.58, 0.16, 0.86],
                4.8,
                1.4,
                0.035,
                0.18,
                -1.2,
                5.0,
                [0.22, 1.9, 0.22],
            ),
            ParticleBurst::Hit => (
                15,
                [1.15, 0.20, 0.08, 0.88],
                3.6,
                2.4,
                0.045,
                0.34,
                -5.5,
                2.8,
                [0.26, 1.55, 0.26],
            ),
            ParticleBurst::Kill => (
                30,
                [1.20, 0.68, 0.18, 0.92],
                4.4,
                3.4,
                0.055,
                0.68,
                -4.2,
                1.8,
                [0.34, 1.35, 0.34],
            ),
            ParticleBurst::Blocked => (
                12,
                [0.72, 0.88, 1.20, 0.90],
                4.2,
                1.8,
                0.032,
                0.24,
                -6.0,
                3.5,
                [0.18, 2.0, 0.18],
            ),
            ParticleBurst::Pickup => (
                22,
                [0.58, 0.92, 1.18, 0.82],
                2.2,
                1.8,
                0.050,
                0.72,
                1.2,
                1.4,
                [0.42, 1.25, 0.42],
            ),
            ParticleBurst::Damage => (
                18,
                [1.10, 0.10, 0.04, 0.72],
                2.8,
                2.6,
                0.045,
                0.42,
                -2.8,
                2.6,
                [0.30, 1.25, 0.30],
            ),
            ParticleBurst::Dash => (
                22,
                [0.68, 0.62, 0.58, 0.56],
                3.2,
                1.4,
                0.055,
                0.45,
                -0.8,
                2.4,
                [0.38, 1.8, 0.38],
            ),
            ParticleBurst::Land => (
                18,
                [0.58, 0.52, 0.47, 0.48],
                2.4,
                2.0,
                0.060,
                0.52,
                -2.2,
                2.2,
                [1.65, 0.28, 1.65],
            ),
        };

        let overflow = self
            .effects
            .len()
            .saturating_add(count)
            .saturating_sub(MAX_EFFECT_PARTICLES);
        if overflow > 0 {
            self.effects.drain(..overflow.min(self.effects.len()));
        }

        self.burst_sequence = self.burst_sequence.wrapping_add(1);
        let axis = if direction.length_squared() > 0.0001 {
            direction.normalize()
        } else {
            Vec3::Y
        };
        for index in 0..count {
            let seed = index + self.burst_sequence as usize * 37;
            let random = Vec3::new(
                signed_seed(seed, 11),
                signed_seed(seed, 12),
                signed_seed(seed, 13),
            )
            .normalize_or_zero();
            let variation = 0.72 + seed_unit(seed, 14) * 0.58;
            let velocity = match kind {
                ParticleBurst::Pickup => random * spread + Vec3::Y * speed * variation,
                ParticleBurst::Dash => -axis * speed * variation + random * spread,
                ParticleBurst::Land => {
                    let flat = Vec3::new(random.x, 0.08 + random.y.abs() * 0.24, random.z)
                        .normalize_or_zero();
                    flat * speed * variation
                }
                ParticleBurst::Kill | ParticleBurst::Damage => {
                    random * speed * variation + Vec3::Y * spread * 0.35
                }
                _ => axis * speed * variation + random * spread,
            };
            let brightness = 0.82 + seed_unit(seed, 15) * 0.38;
            self.effects.push(EffectParticle {
                position: origin + random * 0.045,
                velocity,
                color: [
                    color[0] * brightness,
                    color[1] * brightness,
                    color[2] * brightness,
                    color[3],
                ],
                shape,
                size: size * variation,
                age: 0.0,
                lifetime: lifetime * (0.82 + seed_unit(seed, 16) * 0.36),
                gravity,
                drag,
            });
        }
    }

    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        camera_bind_group: &'pass wgpu::BindGroup,
    ) {
        if self.active_count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, camera_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..self.active_count);
    }
}

fn octahedron_vertices() -> [ParticleVertex; 24] {
    let top = [0.0, 1.4, 0.0];
    let bottom = [0.0, -1.4, 0.0];
    let east = [1.0, 0.0, 0.0];
    let west = [-1.0, 0.0, 0.0];
    let north = [0.0, 0.0, 1.0];
    let south = [0.0, 0.0, -1.0];
    let vertex = |position| ParticleVertex { position };
    [
        vertex(top),
        vertex(east),
        vertex(north),
        vertex(top),
        vertex(north),
        vertex(west),
        vertex(top),
        vertex(west),
        vertex(south),
        vertex(top),
        vertex(south),
        vertex(east),
        vertex(bottom),
        vertex(north),
        vertex(east),
        vertex(bottom),
        vertex(west),
        vertex(north),
        vertex(bottom),
        vertex(south),
        vertex(west),
        vertex(bottom),
        vertex(east),
        vertex(south),
    ]
}

fn seed_unit(index: usize, channel: u64) -> f32 {
    let mut value = (index as u64)
        .wrapping_add(channel.wrapping_mul(0x9e37_79b9_7f4a_7c15))
        .wrapping_add(0x632b_e59b_d9b4_e019);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^= value >> 31;
    (value as u32) as f32 / u32::MAX as f32
}

fn signed_seed(index: usize, channel: u64) -> f32 {
    seed_unit(index, channel) * 2.0 - 1.0
}

fn wrap_axis(value: f32, half_extent: f32) -> f32 {
    (value + half_extent).rem_euclid(half_extent * 2.0) - half_extent
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_particle_seeds_stay_normalized() {
        let first = (0..32).map(|index| seed_unit(index, 7)).collect::<Vec<_>>();
        let second = (0..32).map(|index| seed_unit(index, 7)).collect::<Vec<_>>();
        assert_eq!(first, second);
        assert!(first.iter().all(|value| (0.0..=1.0).contains(value)));
    }

    #[test]
    fn particle_wrapping_keeps_offsets_inside_the_field() {
        assert!((wrap_axis(12.5, 10.0) + 7.5).abs() < 0.0001);
        assert!((wrap_axis(-13.0, 10.0) - 7.0).abs() < 0.0001);
    }
}

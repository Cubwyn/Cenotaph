struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct LightUniform {
    position: vec3<f32>,
    color: vec3<f32>,
    intensity: f32,
};
@group(2) @binding(0)
var<uniform> light: LightUniform;

struct FogUniform {
    density: f32,
    color: vec3<f32>,
};
@group(3) @binding(0)
var<uniform> fog: FogUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
    @location(4) view_dir: vec3<f32>,
};

fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn hash3(p: vec3<f32>) -> f32 {
    return hash(p.xy + p.z * 31.7);
}

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = vec3<f32>(1.0, 1.0, 1.0);
    let world_pos = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_pos = world_pos.xyz;
    out.world_normal = normalize((model_matrix * vec4<f32>(model.normal, 0.0)).xyz);
    out.clip_position = camera.view_proj * world_pos;
    // View direction from world pos toward camera (approximate via inverse of view_proj)
    // For forward rendering, we approximate: camera position is (0,0,0) in view space
    // We'll compute it in fragment shader using distance
    out.view_dir = vec3<f32>(0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ── 1. Basic vectors ─────────────────────────────────────────────────────
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.position - in.world_pos);
    let distance = length(light.position - in.world_pos);

    // ── 2. Improved lighting model ───────────────────────────────────────────
    // Softer attenuation for more even illumination
    let atten_linear = 0.09;
    let atten_quad = 0.032;
    let attenuation = 1.0 / (1.0 + atten_linear * distance + atten_quad * distance * distance);

    // Diffuse with soft wrap lighting (slight light wrap for moody feel)
    let NdotL = dot(normal, light_dir);
    let wrap = 0.35;
    let diffuse = max((NdotL + wrap) / (1.0 + wrap), 0.0);

    // Subtle rim/edge highlight for depth perception
    let view_dir = normalize(-in.world_pos); // approximate: camera at origin in view space
    let rim = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0) * 0.15;

    // Ambient: warm base with slight bounce from below
    let ambient = vec3<f32>(0.12, 0.10, 0.09);
    let bounce = vec3<f32>(0.04, 0.035, 0.03) * max(-normal.y, 0.0);

    let light_contrib = light.intensity * attenuation;
    let lighting = diffuse * light_contrib + ambient + bounce + rim;

    // ── 3. Texture sampling ──────────────────────────────────────────────────
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // ── 4. Combine with light color ──────────────────────────────────────────
    var base_color = tex_color.rgb * lighting * light.color;

    // ── 5. Subtle procedural detail (reduced for performance) ────────────────
    let noise_seed = in.tex_coords * 80.0;
    let grime_noise = hash(noise_seed);
    let vignette = 1.0 - smoothstep(0.3, 1.4, length(in.tex_coords - vec2<f32>(0.5)));
    let detail_factor = 0.85 + grime_noise * 0.1 - vignette * 0.12;
    base_color *= detail_factor;

    // ── 6. Soft color grading ────────────────────────────────────────────────
    // Slight desaturation in shadows, warm tint in highlights
    let luminance = dot(base_color, vec3<f32>(0.299, 0.587, 0.114));
    let shadow_factor = smoothstep(0.0, 0.3, luminance);
    let desat_amount = mix(0.35, 0.1, shadow_factor);
    let graded = mix(base_color, vec3<f32>(luminance), desat_amount);

    // Warm muddy tint — less aggressive than before
    let warm_tint = vec3<f32>(0.95, 0.90, 0.82);
    let final_base = graded * warm_tint;

    // ── 7. Atmospheric fog ───────────────────────────────────────────────────
    // Height-based fog density for underground feel
    let height_fog = exp(-max(in.world_pos.y * 0.005, 0.0));
    let combined_density = fog.density * (1.0 + height_fog * 0.5);
    let fog_factor = 1.0 - exp(-combined_density * distance * distance * 0.01);

    // Fog color: darker at distance, warmer near light
    let fog_tint = mix(vec3<f32>(0.5, 0.45, 0.4), light.color * 0.3, attenuation);
    let effective_fog_color = fog.color * fog_tint;

    let with_fog = mix(final_base, effective_fog_color, saturate(fog_factor));

    // ── 8. Subtle film grain (very light) ────────────────────────────────────
    let grain = (hash(in.tex_coords * 300.0) - 0.5) * 0.015;

    return vec4<f32>(with_fog + grain, 1.0);
}
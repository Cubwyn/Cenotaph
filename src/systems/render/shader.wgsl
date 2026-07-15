struct CameraUniform {
    view_proj: mat4x4<f32>,
    position_time: vec4<f32>,
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
    @location(9) tint: vec4<f32>,
    @location(10) material: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
    @location(3) world_normal: vec3<f32>,
    @location(4) material: vec4<f32>,
};

fn rotate_y(value: vec3<f32>, angle: f32) -> vec3<f32> {
    let sine = sin(angle);
    let cosine = cos(angle);
    return vec3<f32>(
        value.x * cosine - value.z * sine,
        value.y,
        value.x * sine + value.z * cosine,
    );
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
    let role = instance.material.z;
    let phase = instance.material.w;
    let time = camera.position_time.w;
    var animated_position = model.position;
    var animated_normal = model.normal;

    // Small shader-side motions make static prototype meshes feel intentional
    // without adding animation components or per-frame instance uploads.
    if (role > 0.5 && role < 1.5) {
        let angle = time * 0.75 + phase;
        animated_position = rotate_y(animated_position, angle);
        animated_normal = rotate_y(animated_normal, angle);
        animated_position.y += sin(time * 1.65 + phase) * 0.075;
    } else if (role > 1.5 && role < 2.5) {
        let height_weight = clamp(animated_position.y * 0.55, 0.0, 1.0);
        let breath = sin(time * 1.8 + phase);
        animated_position.x += breath * height_weight * 0.018;
        animated_position.y *= 1.0 + breath * 0.006;
    } else if (role > 2.5 && role < 3.5) {
        let pulse = 1.0 + sin(time * 1.15 + phase) * 0.018;
        animated_position.x *= pulse;
        animated_position.z *= pulse;
        animated_position.y += sin(time * 0.8 + phase) * 0.018;
    } else if (role > 3.5 && role < 4.5) {
        let warning_pulse = 1.0 + max(sin(time * 4.2 + phase), 0.0) * 0.035;
        animated_position *= warning_pulse;
    }

    out.tex_coords = model.tex_coords;
    out.color = instance.tint.rgb;
    out.material = instance.material;
    let world_pos = model_matrix * vec4<f32>(animated_position, 1.0);
    out.world_pos = world_pos.xyz;
    let normal_matrix = mat3x3<f32>(
        normalize(model_matrix[0].xyz),
        normalize(model_matrix[1].xyz),
        normalize(model_matrix[2].xyz),
    );
    out.world_normal = normalize(normal_matrix * animated_normal);
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ── 1. Basic vectors ─────────────────────────────────────────────────────
    let normal = normalize(in.world_normal);
    let light_dir = normalize(light.position - in.world_pos);
    let light_distance = length(light.position - in.world_pos);
    let camera_distance = length(camera.position_time.xyz - in.world_pos);

    // ── 2. Improved lighting model ───────────────────────────────────────────
    // Softer attenuation for more even illumination
    let atten_linear = 0.09;
    let atten_quad = 0.032;
    let attenuation = 1.0
        / (1.0 + atten_linear * light_distance + atten_quad * light_distance * light_distance);

    // Diffuse with soft wrap lighting (slight light wrap for moody feel)
    let NdotL = dot(normal, light_dir);
    let wrap = 0.35;
    let diffuse = max((NdotL + wrap) / (1.0 + wrap), 0.0);

    // Subtle rim/edge highlight for depth perception
    let view_dir = normalize(camera.position_time.xyz - in.world_pos);
    var rim_strength = 0.14;
    if (in.material.z > 1.5 && in.material.z < 2.5) {
        rim_strength = 0.20;
    }
    let rim = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0) * rim_strength;

    // Ambient: warm base with slight bounce from below
    let sky_ambient = mix(fog.color * 0.42, vec3<f32>(0.20, 0.18, 0.16), 0.60);
    let ambient = sky_ambient * (0.78 + max(normal.y, 0.0) * 0.22);
    let bounce = fog.color * 0.12 * max(-normal.y, 0.0);

    let light_contrib = light.intensity * attenuation;
    let lighting = diffuse * light_contrib + ambient + bounce + rim;

    // ── 3. Texture sampling ──────────────────────────────────────────────────
    let scaled_uv = in.tex_coords * max(in.material.x, 0.05);
    let tex_color = textureSample(t_diffuse, s_diffuse, scaled_uv);

    // ── 4. Combine with light color ──────────────────────────────────────────
    var emissive_strength = max(in.material.y, 0.0);
    if (in.material.z > 0.5 && in.material.z < 1.5) {
        emissive_strength += 0.08 + 0.06 * (sin(camera.position_time.w * 2.8 + in.material.w) * 0.5 + 0.5);
    } else if (in.material.z > 1.5 && in.material.z < 2.5) {
        emissive_strength += 0.012;
    } else if (in.material.z > 2.5 && in.material.z < 4.5) {
        emissive_strength += 0.035 * (sin(camera.position_time.w * 2.0 + in.material.w) * 0.5 + 0.5);
    }
    let emissive = vec3<f32>(emissive_strength);
    var base_color = tex_color.rgb * in.color * (lighting * light.color + emissive);

    // ── 5. Cheap geometric variation ────────────────────────────────────────
    // Silhouettes and role tints carry the image. Full-screen trigonometric
    // noise was expensive and made untextured blockout assets look muddier.
    base_color *= 0.92 + abs(normal.y) * 0.08;

    // ── 6. Soft color grading ────────────────────────────────────────────────
    // Slight desaturation in shadows, warm tint in highlights
    let luminance = dot(base_color, vec3<f32>(0.299, 0.587, 0.114));
    let shadow_factor = smoothstep(0.0, 0.3, luminance);
    let desat_amount = mix(0.35, 0.1, shadow_factor);
    let graded = mix(base_color, vec3<f32>(luminance), desat_amount);

    let final_base = graded * vec3<f32>(0.98, 0.98, 0.96);

    // ── 7. Atmospheric fog ───────────────────────────────────────────────────
    // Camera-relative distance keeps fog stable in levels authored far from Y=0.
    let height_delta = in.world_pos.y - camera.position_time.y;
    let low_fog = 1.0 + clamp(-height_delta * 0.035, 0.0, 0.45);
    let combined_density = fog.density * low_fog;
    let fog_factor = 1.0 - exp(-combined_density * camera_distance * camera_distance * 0.012);

    // Fog color: darker at distance, warmer near light
    let fog_tint = mix(vec3<f32>(0.72, 0.68, 0.64), light.color * 0.48, attenuation);
    let effective_fog_color = fog.color * fog_tint;

    let with_fog = mix(final_base, effective_fog_color, saturate(fog_factor));

    return vec4<f32>(with_fog, 1.0);
}

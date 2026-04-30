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
};

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
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple lighting calculation
    let light_dir = normalize(light.position - in.world_pos);
    let distance = length(light.position - in.world_pos);
    let attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
    let diffuse = max(dot(vec3<f32>(0.0, 1.0, 0.0), light_dir), 0.0);
    let lighting = (diffuse * light.intensity * attenuation) + 0.1; // Add ambient
    
    let final_color = in.color * lighting * light.color;
    
    // Fog calculation
    let fog_factor = 1.0 - exp(-fog.density * fog.density * distance * distance);
    let result = mix(final_color, fog.color, fog_factor);
    
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return tex_color * vec4<f32>(result, 1.0);
}

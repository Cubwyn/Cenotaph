struct CameraUniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

// We will use the light uniform for color/intensity, but we'll override the "clean" feel in the shader
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

// Simple pseudo-random noise function for grime/dust
fn hash(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
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
    out.clip_position = camera.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Distance Calculation
    let light_dir = normalize(light.position - in.world_pos);
    let distance = length(light.position - in.world_pos);
    
    // 2. Grimy Lighting (Darker, murkier)
    // Increased quadratic falloff for tighter, dirtier pools of light
    let attenuation = 1.0 / (1.0 + 0.15 * distance + 0.08 * distance * distance);
    
    // Use a flat normal approximation or pass real normals if available
    // Using a slightly tilted normal to break up perfect symmetry
    let normal = vec3<f32>(0.0, 1.0, 0.0); 
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Bright ambient to see the level
    let ambient = vec3<f32>(0.5, 0.5, 0.5); 
    let lighting = (diffuse * light.intensity * attenuation) + ambient;
    
    let base_lighting = in.color * lighting * light.color;
    
    // 3. Texture Sampling
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    
    // 4. Procedural Grime Overlay
    // Generate noise based on UVs to simulate dust/dirt accumulation
    let noise_seed = in.tex_coords * 150.0;
    let grime_noise = hash(noise_seed);
    // Make corners/dark areas darker (simple vignette-ish grime)
    let vignette = 1.0 - smoothstep(0.2, 1.2, length(in.tex_coords - vec2<f32>(0.5)));
    let grime_factor = 0.8 + (grime_noise * 0.15) - (vignette * 0.2);
    
    let dirty_color = tex_color.rgb * base_lighting * grime_factor;
    
    // 5. Color Grading (Desaturation & Muddy Tint)
    // Desaturate to kill vibrancy
    let luminance = dot(dirty_color, vec3<f32>(0.299, 0.587, 0.114));
    let desaturated = mix(dirty_color, vec3<f32>(luminance), 0.45); // 45% desaturation
    
    // Apply a sickly/muddy tint (Green-Brown shift)
    let muddy_tint = vec3<f32>(0.9, 0.85, 0.75); 
    let grimy_base = desaturated * muddy_tint;
    
    // 6. Grimy Fog (Thicker, darker, mixes differently)
    // Increase density effect for a "soup" feel
    let fog_density = fog.density * 1.5; 
    let fog_factor = 1.0 - exp(-fog_density * distance);
    
    // Ensure fog color is dark and murky if not set externally
    let effective_fog_color = fog.color * vec3<f32>(0.6, 0.55, 0.5); 
    
    let final_color = mix(grimy_base, effective_fog_color, fog_factor);
    
    // 7. Film Grain / Dust Particles
    // Add high-frequency noise to the final image
    let grain = (hash(in.tex_coords * 400.0) - 0.5) * 0.04;
    
    return vec4<f32>(final_color + grain, 1.0);
}
struct CameraUniform {
    view_proj: mat4x4<f32>,
    position_time: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) instance_position_size: vec4<f32>,
    @location(2) instance_color: vec4<f32>,
    @location(3) instance_shape: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let local_shape = input.position * input.instance_shape.xyz;
    let world_position = input.instance_position_size.xyz
        + local_shape * input.instance_position_size.w;
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(world_position, 1.0);
    out.color = input.instance_color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}

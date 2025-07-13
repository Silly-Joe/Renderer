struct Uniforms {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    let world_pos = vec4(position, 1.0);
    return uniforms.view_proj * world_pos;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(1.0, 0.0, 0.0, 1.0); // rot
}
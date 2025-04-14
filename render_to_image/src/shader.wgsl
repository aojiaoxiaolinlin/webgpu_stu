struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}
const view_matrix: mat4x4<f32> = mat4x4<f32>(
    vec4<f32>(1.0 / 1280.0, 0.0, 0.0, 0.0),
    vec4<f32>(0.0, -1.0 / 720.0, 0.0, 0.0),
    vec4<f32>(0.0, 0.0, 1.0, 0.0),
    vec4<f32>(0.0, 0.0, 0.0, 1.0)
);

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    let pos = vec4<f32>(in.position.x, in.position.y, in.position.z, 1.0);
    let color = vec4<f32>(in.color, 1.0);
    return VertexOutput(pos, color);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
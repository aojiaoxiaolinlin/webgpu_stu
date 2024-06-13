struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(model.position, 1.0);
    output.tex_coords = model.tex_coords;
    return output;
}
// 变量 t_diffuse 和 s_diffuse 就是所谓的 uniforms，它们是在渲染时由应用程序传递给着色器的。
// 通过 @group 和 @binding 修饰符，我们可以指定这两个变量的绑定点。
// @group(0) 对应于 set_bind_group() 中的第一个参数，@binding(x) 与我们创建绑定组布局和绑定组时指定的 binding 值对应。
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
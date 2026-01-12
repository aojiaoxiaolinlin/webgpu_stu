struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec4<f32>,
    life: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    // 为了画圆
    @location(1) uv: vec2<f32>,
};

// 这里是只读的，因为渲染时不需要修改数据
@group(0) @binding(0) var<storage, read> particles: array<Particle>;

@vertex
fn vs_main(
    @builtin(vertex_index) v_index: u32,
    @builtin(instance_index) i_index: u32
) -> VertexOutput {
    var output: VertexOutput;
    // 1. 获取对应的粒子
    // output.clip_position = vec4<f32>(particles[i_index].pos, 0.0, 1.0);
    // output.color = particles[i_index].color;
    //
    let particle = particles[i_index];

    // 2. 凭空定义一个正方形的 4 个角 (偏移量)
    // 顺序是：左下, 右下, 左上, 右上
    // 假设我们要画 6 个点组成的两个三角形：
    // Triangle 1: 0, 1, 2
    // Triangle 2: 2, 1, 3 (注意绕序)

    // 定义 4 个角的坐标 (相对于粒子中心)
    var corners = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0), // 左下
        vec2<f32>( 1.0, -1.0), // 右下
        vec2<f32>(-1.0,  1.0), // 左上
        vec2<f32>( 1.0,  1.0)  // 右上
    );
    // 把 0~5 的vertex_index 映射到 0~3 的角上
    // 0 -> 0, 1 -> 1, 2 -> 2, 3 -> 2, 4 -> 1, 5 -> 3
    let corner_indices = array<u32, 6>(
        0, 1, 2, 2, 1, 3
    );
    // 根据 vertex_index 获取对应的角的坐标
    let offset = corners[corner_indices[v_index]];
    // 3. 设定粒子大小（半径）
    let radius = 0.05; // 可以调节，uniform传递

    // 4. 计算最终位置
    // 粒子中心 + 偏移量 * 半径（大小）
    // 注意：这里没有处理长宽比，如果窗口不是正方形，粒子会变扁。
    // 如果要完美圆形，需要传入窗口长宽比进行修正。
    let final_pos = particle.pos + offset * radius;
    // 计算顶点位置
    output.clip_position = vec4<f32>(final_pos, 0.0, 1.0);
    output.color = particle.color;

    // 5. 传递uv（把 -1.0~1.0 的 offset 当作UV用，方便画圆）
    output.uv = offset;

    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return in.color;

    // in.uv 的范围是[-1.0, 1.0]
    // 计算当前像素距离中心的距离
    let dist_sq = dot(in.uv, in.uv); // 点积 = x^2 + y^2
    // 如果距离大于 1.0（在圆外面），丢弃这个像素。
    if (dist_sq > 1.0) {
        discard;
    }

    // 增加一点边缘柔化 (Anti-aliasing)
    // smoothstep(min, max, x): “平滑阶梯”
    // Min (下限): 这里是 0.8
    // Max (上限): 这里是 1.0
    // X (输入值): 这里是 sqrt(dist_sq)，也就是当前像素距离圆心的距离。

    // 如果 X 小于 Min (比 0.8 小)：
    // "还没到坡道呢！" -> 输出 0.0 (纯黑)。
    // 如果 X 大于 Max (比 1.0 大)：
    // "已经过了坡道啦！" -> 输出 1.0 (纯白)。
    // 如果 X 夹在中间 (0.8 到 1.0 之间)：
    // "正在爬坡！" -> 输出 0.0 到 1.0 之间的平滑过渡值。
    // 而且这个过渡不是直线的，是 S形曲线（两头慢，中间快），非常自然顺滑。
    // smoothstep 就是一个 “智能钳子”。它把 0.8 以下的砍成 0，把 1.0 以上的砍成 1，中间的拉成一条丝滑的曲线。

    // sqrt(dist_sq): 这是 距离。
    // 圆心是 0.0，边缘是 1.0。
    // smoothstep(0.8, 1.0, 距离):
    // 圆心到 0.8 的区域：输入小于 0.8，函数输出 0.0。
    // 0.8 到 1.0 的边缘区域：输入在变大，函数输出从 0.0 慢慢变成 1.0。
    // 1.0 - ... (反转一下)：
    // 圆心区域：1.0 - 0.0 = 1.0 (完全不透明，实心的)。
    // 边缘区域：1.0 - (0.0~1.0) = 1.0 慢慢变到 0.0 (慢慢变透明)。
    let alpha = 1.0 - smoothstep(0.8, 1.0, sqrt(dist_sq));
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

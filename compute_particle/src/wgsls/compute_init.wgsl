struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec4<f32>,
    life: f32,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> delta_time: f32;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if index >= arrayLength(&particles) {
        return;
    }

    // 用粒子的 index 作为随机数种子
    let rand_x = pcg_hash(index + 1u);
    let rand_y = pcg_hash(index + 9999u);
    let rand_vx = pcg_hash(index + 12345u);
    let rand_vy = pcg_hash(index + 54321u);

    // 随机位置： 映射到[-1.0, 1.0]
    particles[index].pos = vec2(rand_x * 2., rand_y) * 2.0 - 1.0;
    // 随机速度： 向四周炸开
    particles[index].vel = (vec2(rand_vx * 2., rand_vy) * 2.0 - 1.0) * 0.5;
    // 随机颜色
    particles[index].color = vec4<f32>(rand_x, 0.5, rand_y, 1.0);
}

// GPU 用的伪随机数生成器 (输入一个种子，输出一个 0.0 到 1.0 的随机数)
fn pcg_hash(seed: u32) -> f32 {
    var state = seed * 747796405u + 2891336453u;
    let word = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    return f32((word >> 22u) ^ word) / f32(0xFFFFFFFFu);
}

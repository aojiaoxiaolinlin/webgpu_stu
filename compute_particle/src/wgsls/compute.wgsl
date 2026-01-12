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

    // 读取当前粒子
    var particle = particles[index];
    // 物理更新
    particle.pos += particle.vel * delta_time;
    // 碰撞检测,触碰到墙壁反弹
    if (particle.pos.x > 1.0 || particle.pos.x < -1.0) {
        particle.vel.x = -particle.vel.x;
    }
    if (particle.pos.y > 1.0 || particle.pos.y < -1.0) {
        particle.vel.y = -particle.vel.y;
    }

    // 寿命管理
    particle.life -= delta_time;

    particles[index] = particle;
}

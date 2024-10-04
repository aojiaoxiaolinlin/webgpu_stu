use glam::{vec3, Vec2, Vec3};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Mesh {
    position: Vec3,
    color: Vec3,
    uv: Vec2,
}

impl Mesh {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2=>Float32x2];
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Mesh>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub const MESH: &[Mesh] = &[
    Mesh {
        position: Vec3::new(0.0, 0.5, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Mesh {
        position: Vec3::new(-0.5, -0.0, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Mesh {
        position: Vec3::new(0.0, -0.5, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
        uv: Vec2::new(0.0, 1.0),
    },
    Mesh {
        position: Vec3::new(-0.5, 0.5, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        uv: Vec2::new(1.0, 0.0),
    },
    Mesh {
        position: Vec3::new(0.0, -0.5, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        uv: Vec2::new(0.0, 0.0),
    },
    Mesh {
        position: Vec3::new(0.5, 0.0, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
        uv: Vec2::new(0.0, 1.0),
    },
];

pub fn calc_bundle() -> (Vec2, Vec2) {
    let (mut min_x, mut max_x) = (MESH[0].position.x, MESH[0].position.y);
    let (mut min_y, mut max_y) = (MESH[0].position.x, MESH[0].position.y);
    MESH.iter().for_each(|mesh| {
        if mesh.position.x < min_x {
            min_x = mesh.position.x;
        } else if mesh.position.x > max_x {
            max_x = mesh.position.x;
        } else if mesh.position.y < min_y {
            min_y = mesh.position.y;
        } else if mesh.position.y > max_y {
            max_y = mesh.position.y;
        }
    });
    (Vec2::new(min_x, min_y), Vec2::new(max_x, max_y))
}

pub fn mesh_size(min: &Vec2, max: &Vec2) -> (f32, f32) {
    (max.x - min.x, max.y - min.y)
}
pub const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];
pub const RECTANGLE: &[Mesh] = &[
    Mesh {
        position: vec3(1.0, 0.5, 0.0),
        color: Vec3::INFINITY,
        uv: Vec2::new(1.0, 0.0),
    },
    Mesh {
        position: vec3(0.5, 0.5, 0.0),
        color: Vec3::INFINITY,
        uv: Vec2::new(0.0, 0.0),
    },
    Mesh {
        position: vec3(0.5, -0.5, 0.0),
        color: Vec3::INFINITY,
        uv: Vec2::new(0.0, 1.0),
    },
    Mesh {
        position: vec3(1.0, -0.5, 0.0),
        color: Vec3::INFINITY,
        uv: Vec2::new(1.0, 1.0),
    },
];

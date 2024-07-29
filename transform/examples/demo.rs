use glam::{Quat, Vec3, Vec4};

fn main() {
    let v4 = Vec4::new(1.0, 2.0, 3.0, 0.0);

    // 创建一个平移矩阵
    let translation = glam::Mat4::from_translation(glam::Vec3::new(0.5, 0.0, 0.0));

    println!("translation: {:?}", translation);

    // 创建一个缩放矩阵
    let scale = glam::Mat4::from_scale(glam::Vec3::new(2.0, 2.0, 2.0));

    println!("scale: {:?}", scale);

    let from_scale_rotation_translation = glam::Mat4::from_scale_rotation_translation(
        Vec3::new(2.0, 2.0, 2.0),
        Quat::from_rotation_z(30.0),
        Vec3::new(0.5, 0.0, 0.0),
    );

    println!(
        "from_scale_rotation_translation: {:?}",
        from_scale_rotation_translation
    );

    let transform_point3 = scale.transform_point3(Vec3::new(1.0, 1.0, 1.0));
    println!("transform_point3: {:?}", transform_point3);
}

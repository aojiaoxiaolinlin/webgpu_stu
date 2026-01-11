use image::{GenericImageView, ImageBuffer, Rgba};
use rust_embed::RustEmbed;
// 指定要嵌入的文件夹路径（相对于 Cargo.toml）
//
#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn load_image_from_file(path: &str) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, (u32, u32)) {
    // 这是一个运行时查找，但数据是编译时嵌入的
    let image_data = Assets::get(path).map(|file| file.data.into_owned());

    match image_data {
        Some(data) => {
            // 处理图像数据
            let image = image::load_from_memory(&data).unwrap();
            let rgba = image.to_rgba8();
            let dimensions = image.dimensions();
            (rgba, dimensions)
        }
        None => panic!("文件路径不存在!,{}", path),
    }
}

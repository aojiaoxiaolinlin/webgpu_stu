use image::{GenericImageView, ImageBuffer, Rgba};
use rust_embed::RustEmbed;
use wgpu::Extent3d;
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

pub fn copy_buffer_save_image(
    file_name: &str,
    texture: &wgpu::Texture,
    size: Extent3d,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    // 将纹理数据复制到CPU内存
    // 纹理宽度
    // 乘以 4 是因为每个 RGBA 像素需要4字节。
    let unpadded_byte_per_row = size.width as u32 * 4;
    // GPU 中数据访问需要内存对齐，通常是256字节
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u32;
    // upadded_byte_per_row_padding % align: 当前字节数除以对齐值的余数。
    // align - 余数: 需要补充的字节数。
    // 外层的 % align: 计算需要补充的字节数 如果为余数为0，则不需要填充。
    let padded_byte_per_row_padding = (align - unpadded_byte_per_row % align) % align;
    // 计算填充后的字节数，这个值满足 padded_byte_per_row % align == 0
    let padded_byte_per_row = unpadded_byte_per_row + padded_byte_per_row_padding;

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: (padded_byte_per_row * size.height) as u64,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_byte_per_row),
                rows_per_image: None,
            },
        },
        size,
    );

    let index = queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    // map_async 异步映射缓冲区到CPU可访问内存。
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).unwrap();
    });
    device
        .poll(wgpu::PollType::Wait {
            // 等待指定的提交所以
            submission_index: Some(index),
            // None 无限期等待，不会超时
            timeout: None,
        })
        .unwrap();
    // 等待从通道接收数据
    // 第一个 `unwrap()`：处理接收失败（如果通道关闭）
    // 第二个 `unwrap()`：处理映射失败（如果缓冲区未准备好）
    receiver.recv().unwrap().unwrap();

    // 获取已经映射到CPU可访问的缓冲区数据。
    let data = buffer_slice.get_mapped_range();
    let mut bytes = Vec::with_capacity((size.width * 4 * size.height) as usize);
    // 将缓冲区数据转换为RGBA格式的像素数据。
    // chunks_exact()方法用于将缓冲区数据按行分割成固定大小的块。
    for row in data.chunks_exact(padded_byte_per_row as usize) {
        // 只读取每行的前 unpadded_byte_per_row 字节，即一行的像素数据，排除对齐字节填充
        bytes.extend_from_slice(&row[..unpadded_byte_per_row as usize]);
    }
    let image_buffer = image::RgbaImage::from_raw(size.width, size.height, bytes)
        .expect("Retrieved texture buffer must be a valid RgbaImage");
    let output_path = format!("render_to_image/output/{file_name}.png");
    image_buffer
        .save(output_path)
        .expect("Failed to save image");
}

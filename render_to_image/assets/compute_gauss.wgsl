// 1. 定义绑定资源
@group(0) @binding(0) var input_texture: texture_2d<f32>;
// 输出图 格式要和Rust中创建的一致，如 rgba8unorm 且 不能是带 srgb 格式
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

// 2.高斯权重（3x3 近似）
// 0.0625 = 1 / 16
const weights: array<f32, 9> = array<f32, 9>(
    0.0625, 0.125, 0.0625,
    0.125, 0.25, 0.125,
    0.0625, 0.125, 0.0625
);

// 1. 定义常数 (也可以通过 uniform 传进来，但这里为了简单直接写死)
// 试着把这里改成 5, 8, 10 看看效果！(不要超过 15 哦，显卡会哭的)
const RADIUS: i32 = 12;

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // 获取当前像素坐标
    let x = global_id.x;
    let y = global_id.y;

    // 获取图片尺寸，防止越界采样
    let dims = textureDimensions(input_texture);
    let width = dims.x;
    let height = dims.y;

    // 如果超出了图片范围，直接返回
    if x < 0 || x >= width || y < 0 || y >= height {
        return;
    }

    // --- 准备高斯参数 ---
    // Sigma 决定了模糊有多“散”。通常设为半径的一半比较自然。
    let sigma = f32(RADIUS) / 2.0;
    // 预计算一下 2 * sigma^2，省点除法运算
    let two_sigma_sq = 2.0 * sigma * sigma;

    // 计算高斯模糊后的颜色值
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);
    var weight_sum = 0.0;
    // 3x3 循环采样
    // 偏移量从 -1 到 1
    // for (var offset_x = -1; offset_x <=1 ; offset_x++) {
    //     for(var offset_y = -1; offset_y <= 1; offset_y++) {

    //         // 计算邻居坐标
    //         let sample_x = i32(x) + offset_x;
    //         let sample_y = i32(y) + offset_y;
    //         // 边界检查
    //         if (sample_x >= 0 && sample_x < i32(width) && sample_y >= 0 && sample_y < i32(height)) {
    //             // 读取邻居颜色 textureLoad 不需要采样器，直接使用整数坐标读
    //             let sample_color = textureLoad(input_texture, vec2<i32>(sample_x, sample_y), 0);

    //             // 简单的权重计算 (这里喵偷懒用简单的逻辑，不用数组查表了)
    //             // 核心是 4，十字邻居是 2，角落是 1
    //             var weight = 1.0;
    //             if (offset_x == 0 && offset_y == 0) { weight = 4.0; }
    //             else if (offset_x == 0 || offset_y == 0) { weight = 2.0; }
    //             // color += sample_color * weights[(offset_x + 1) * 3 + (offset_y + 1)];
    //             // weight_sum += weights[(offset_x + 1) * 3 + (offset_y + 1)];


    //             color += sample_color * weight;
    //             weight_sum += weight;

    //         }
    //     }
    // }


    // --- 双重循环 (暴力版) ---
    // 从 -RADIUS 循环到 +RADIUS
    for (var offset_x = -RADIUS; offset_x <= RADIUS; offset_x++) {
        for (var offset_y = -RADIUS; offset_y <= RADIUS; offset_y++) {

            // 1. 算出采样坐标
            let sample_x = i32(x) + offset_x;
            let sample_y = i32(y) + offset_y;

            // 2. 边界检查 (Clamp 到边缘，防止黑边)
            // 喵小贴士：简单的做法是丢弃，但更好的做法是取边缘像素
            // 这里我们用 clamp 保证不出界
            let coords = vec2<i32>(
                clamp(sample_x, 0, i32(width - 1)),
                clamp(sample_y, 0, i32(height - 1))
            );

            // 3. 读取颜色
            let sample_color = textureLoad(input_texture, coords, 0);

            // 4. 🔥 核心：计算高斯权重 🔥
            // 距离圆心的距离平方 (x^2 + y^2)
            let dist_sq = f32(offset_x * offset_x + offset_y * offset_y);

            // 套用高斯公式: e^(-d^2 / 2sigma^2)
            let weight = exp(-dist_sq / two_sigma_sq);

            // 5. 累加
            color += sample_color * weight;
            weight_sum += weight;
        }
    }


    // 归一化：除以总权重
    color /= weight_sum;

    // 确保 Alpha 通道正确（通常保持 1.0 或者原图 Alpha)
    color.a = 1.0;

    // 将结果写入输出纹理
    textureStore(output_texture, vec2<u32>(x, y), linear_to_srgb(color));
}

fn linear_to_srgb(linear: vec4<f32>) -> vec4<f32> {
    var rgb: vec3<f32> = linear.rgb;
    if (linear.a > 0.0) {
        rgb = rgb / linear.a;
    }
    let a = 12.92 * rgb;
    let b = 1.055 * pow(rgb, vec3<f32>(1.0 / 2.4)) - 0.055;
    let c = step(vec3<f32>(0.0031308), rgb);
    return vec4<f32>(mix(a, b, c) * linear.a, linear.a);
}

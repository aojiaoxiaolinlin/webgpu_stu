use anyhow::anyhow;
use image::GenericImageView;
use wgpu::{util::DeviceExt, SurfaceError};
use winit::window::Window;
pub struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    num_vertices: u32,
    diffuse_bind_group: wgpu::BindGroup,
}

impl State<'_> {
    pub async fn new(window: &Window) -> anyhow::Result<Self> {
        let (instance, _backend) = create_wgpu_instance()?;
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window)?)
        }?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("没有找到可用的适配器"))?;
        dbg!(adapter.get_info());
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("Device"),
                },
                None,
            )
            .await?;
        let caps = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: caps.formats[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // 纹理和绑定组
        // 此处代码从图像文件中读取字节，并将其加载到 image 对象中，然后转换为 rgba 动态数组。我们还保存了图像的尺寸信息以便在创建实际纹理时使用。
        let diffuse_bytes = include_bytes!("../assets/xiongmao.jpg");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.to_rgba8();
        let dimensions = diffuse_image.dimensions();

        // 创建纹理
        let texture_size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
            // 所有纹理都是以 3D 形式存储的，我们通过设置深度 1 来表示 2D 纹理
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // 大多数图像都是使用 sRGB 来存储的，我们需要在这里指定。
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            // TEXTURE_BINDING 表示我们要在着色器中使用这个纹理。
            // COPY_DST 表示我们能将数据复制到这个纹理上。
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("定义纹理"),
            view_formats: &[],
        });
        // 将图像数据上传到纹理
        queue.write_texture(
            // 告诉 wgpu 从何处复制像素数据
            wgpu::ImageCopyTextureBase {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // 实际像素数据
            &diffuse_rgba,
            // 纹理的内存布局
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view =
            diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // 参数指定了如果采样器得到的纹理坐标超出了纹理边界时该如何处理。我们有几个选项可供选择：
        //  - Repeat：纹理坐标会被重复，这样纹理就会重复出现。
        // - MirrorRepeat：纹理坐标会被重复，这样纹理就会重复出现。：纹理坐标会被镜像，这样纹理就会被镜像出现。
        // - ClampToEdge：任何在纹理外的纹理坐标将返回离纹理边缘最近的像素的颜色。
        // mag_filter 与 min_filter 字段描述了当采样足迹小于或大于一个纹素（Texel）时该如何处理。当场景中的贴图远离或靠近 camera 时，这两个字段的设置通常会有效果。
        //  - Linear：在每个维度中选择两个纹素，并在它们的值之间返回线性插值。
        //  - Nearest：返回离纹理坐标最近的纹素的值。这创造了一个从远处看比较清晰但近处有像素的图像。然而，如果你的纹理被设计成像素化的，比如像素艺术游戏，或者像 Minecraft 这样的体素游戏，这可能是符合预期的。
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("采样器"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // 创建绑定组
        // 绑定组（BindGroup）描述了一组资源以及如何通过着色器访问它们。我们先来创建一个绑定组布局（BindGroupLayout）：
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("资源绑定组"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        // 指定片元着色器可见
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        //  这里需要匹配纹理的采样器
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("绑定组"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wgsls/shader.wgsl").into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                compilation_options: Default::default(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // 图元，描述了如何将顶点数据转换为图元
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            num_vertices: VERTICES.len() as u32,
            diffuse_bind_group,
        })
    }
    pub fn render(&mut self) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            // 函数接收两个参数，第一个参数是顶点缓冲区要使用的缓冲槽索引。你可以连续设置多个顶点缓冲区。
            // 第二个参数是要使用的缓冲区的数据片断
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

            // 怎么画，3个顶点，1个实例
            render_pass.draw(0..self.num_vertices, 0..1);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
    pub fn update(&mut self) {}
    pub fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        if physical_size.width > 0 && physical_size.height > 0 {
            self.config.width = physical_size.width;
            self.config.height = physical_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

fn create_wgpu_instance() -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in wgpu::Backends::all() {
        if let Some(instance) = try_wgpu_backend(backend) {
            return Ok((instance, backend));
        }
    }
    Err(anyhow!("没有找到可用渲染后端"))
}
fn try_wgpu_backend(backend: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: backend,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backend).is_empty() {
        None
    } else {
        Some(instance)
    }
}

#[repr(C)] // 保证结构体的内存布局和C语言一致，用于和C语言交互，共享数据
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}
const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        // 纹理坐标的原点在左上角
        tex_coords: [0.5131759, 1.0 - 0.99240386],
    }, // A
    Vertex {
        position: [-0.5, -0.5, 0.0],
        tex_coords: [0.0048659444, 1.0 - 0.56958647],
    }, // B
    Vertex {
        position: [0.5, -0.5, 0.0],
        tex_coords: [0.28081453, 1.0 - 0.05060294],
    }, // C
];

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x3,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 1,
                },
            ],
        }
    }
}

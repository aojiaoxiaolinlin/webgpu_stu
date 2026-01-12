use std::time::Duration;

use anyhow::anyhow;
use glam::{Vec2, Vec4};
use wgpu::{SurfaceError, TextureFormat};
use winit::window::Window;

const PARTICLE_COUNT: u32 = 1024;

pub struct State<'window> {
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    compute_bind_group: wgpu::BindGroup,
    compute_pipeline: wgpu::ComputePipeline,
    time_buffer: wgpu::Buffer,
}

impl State<'_> {
    pub async fn new(window: &Window) -> anyhow::Result<Self> {
        let (instance, _backend) = create_wgpu_instance().await?;
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
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: Some("Device"),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await?;
        let caps = surface.get_capabilities(&adapter);
        let format = TextureFormat::Rgba8Unorm;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: if caps.formats.contains(&format) {
                format
            } else {
                caps.formats[0]
            },
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wgsls/shader.wgsl").into()),
        });

        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    {
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    },
                ],
            });

        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                immediate_size: 0,
            });

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Render Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                compilation_options: Default::default(),
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            // 图元，描述了如何将顶点数据转换为图元
            primitive: wgpu::PrimitiveState {
                // topology: wgpu::PrimitiveTopology::PointList,
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
            multiview_mask: None,
            cache: None,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Init Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wgsls/compute_init.wgsl").into()),
        });

        let init_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Init Compute Pipeline"),
                layout: Some(&compute_pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                cache: None,
            });

        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: std::mem::size_of::<Particle>() as u64 * PARTICLE_COUNT as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Random Number Buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: particle_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: time_buffer.as_entire_binding(),
                },
            ],
        });

        // 绑定同一个buffer
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_buffer.as_entire_binding(),
            }],
        });

        let mut init_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Computer Encoder"),
        });
        {
            let mut compute_pass = init_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Init Computer Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&init_compute_pipeline);
            compute_pass.set_bind_group(0, &compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(PARTICLE_COUNT.div_ceil(64), 1, 1);
        }
        queue.submit(Some(init_encoder.finish()));

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./wgsls/compute.wgsl").into()),
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            render_bind_group,
            render_pipeline,
            compute_bind_group,
            compute_pipeline,
            time_buffer,
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
            // 物理模拟
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(PARTICLE_COUNT.div_ceil(64), 1, 1);
        }

        {
            // 渲染
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.render_pipeline);
            // 复用bind_group 它们绑定了同一个buffer
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            // 画 1024 个点
            // 画 N 个点，每个点一个顶点
            // render_pass.draw(0..PARTICLE_COUNT, 0..1);
            // 0..6 画一个正方形需要的顶点数（两个三角形：0，1，2，和 2，1，3）
            // 0..n: 就是 instance 的 数量
            render_pass.draw(0..6, 0..PARTICLE_COUNT);
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
    pub fn update(&mut self, delta_time: Duration) {
        // 更新 delta_time
        // bytemuck::bytes_of(): 将一个值转换为字节数组&[u8]，必须是一个单一的变量引用（比如 &f32, &MyStruct）。
        // bytemuck::cast_slice(&slice): 将一个切片（一堆）数据转换为字节数组&[u8]，必须是一个切片（Slice），比如 &[f32], &[Particle], 或者 &Vec<T>
        self.queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::bytes_of(&delta_time.as_secs_f32()),
        );
    }
    pub fn resize(&mut self, physical_size: winit::dpi::PhysicalSize<u32>) {
        if physical_size.width > 0 && physical_size.height > 0 {
            self.config.width = physical_size.width;
            self.config.height = physical_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
}

async fn create_wgpu_instance() -> anyhow::Result<(wgpu::Instance, wgpu::Backends)> {
    for backend in wgpu::Backends::all() {
        if let Some(instance) = try_wgpu_backend(backend).await {
            return Ok((instance, backend));
        }
    }
    Err(anyhow!("没有找到可用渲染后端"))
}
async fn try_wgpu_backend(backend: wgpu::Backends) -> Option<wgpu::Instance> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: backend,
        flags: wgpu::InstanceFlags::default().with_env(),
        ..Default::default()
    });
    if instance.enumerate_adapters(backend).await.is_empty() {
        None
    } else {
        Some(instance)
    }
}

#[repr(C)] // 保证结构体的内存布局和C语言一致，用于和C语言交互，共享数据
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Particle {
    pos: Vec2,
    vel: Vec2,
    color: Vec4,
    life: f32,
    _padding: [f32; 3],
}
/// 🎨 标准 sRGB 转 Linear RGB 转换器
///
/// 这是一个纯 Rust 实现，不依赖任何第三方库。
/// 遵循 IEC 61966-2-1 标准 (混合了线性段和指数段)。
pub mod color_utils {

    /// 将单个 sRGB 通道 (0.0 - 1.0) 转换为 Linear 通道 (0.0 - 1.0)
    pub fn srgb_to_linear(s: f64) -> f64 {
        // 1. 确保输入在合理范围内（虽然通常不会越界，但为了安全喵）
        let s = s.clamp(0.0, 1.0);

        // 2. 标准公式判定
        // 如果颜色很暗 (<= 0.04045)，使用线性变换 (除以 12.92)
        // 否则使用 Gamma 2.4 变换 (稍微偏移后取 2.4 次方)
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }

    /// 便捷函数：输入整数 RGB (0-255)，输出线性 RGB 数组 [r, g, b]
    pub fn srgb_u8_to_linear(r: u8, g: u8, b: u8) -> [f64; 3] {
        // 先把 0-255 归一化到 0.0-1.0
        let r_norm = r as f64 / 255.0;
        let g_norm = g as f64 / 255.0;
        let b_norm = b as f64 / 255.0;

        [
            srgb_to_linear(r_norm),
            srgb_to_linear(g_norm),
            srgb_to_linear(b_norm),
        ]
    }
}

use anyhow::Result;
use winit::{dpi::PhysicalSize, window::Window};

use crate::SpecialRenderPipeline;

pub mod mesh;

pub struct RenderRes<'window> {
    pub surface: wgpu::Surface<'window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl RenderRes<'_> {
    pub async fn new(window: &Window) -> Result<Self> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = unsafe {
            instance.create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(window)?)?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("没有找到可用适配器");
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("设备"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
            })
            .await
            .expect("创建设备失败");

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        Ok(Self {
            surface,
            device,
            queue,
            config,
        })
    }
}

pub struct Renderer<'window, T: SpecialRenderPipeline> {
    pub window: Option<Window>,
    pub render_res: Option<RenderRes<'window>>,
    pub render: T,
}

impl<T: SpecialRenderPipeline> Renderer<'_, T> {
    pub fn new(render: T) -> Self {
        Self {
            render,
            render_res: None,
            window: None,
        }
    }

    pub fn resize(&mut self, physical_size: PhysicalSize<u32>) {
        if let Some(render_res) = &mut self.render_res {
            render_res.config.width = physical_size.width;
            render_res.config.height = physical_size.height;
            render_res
                .surface
                .configure(&render_res.device, &render_res.config);
        }
    }

    pub fn render(&mut self) -> Result<()> {
        let Some(render_res) = &mut self.render_res else {
            return Err(anyhow::Error::msg("render_res 不存在"));
        };
        let output = render_res.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            render_res
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            let render_pipeline = self
                .render
                .special_render_pipeline(&render_res.device, render_res.config.format);

            self.render
                .draw(render_pass, &render_pipeline, &render_res.device);
        }
        render_res.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub(crate) fn set_window(&mut self, window: Window) {
        self.render_res = Some(futures::executor::block_on(RenderRes::new(&window)).unwrap());
        self.window = Some(window);
    }
}

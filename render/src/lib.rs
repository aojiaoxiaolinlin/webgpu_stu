use render::Renderer;
use wgpu::{RenderPass, RenderPipeline};
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ControlFlow, window::Window,
};

pub mod render;

pub struct WinitRunner<'window, T: SpecialRenderPipeline> {
    renderer: Option<Renderer<'window, T>>,
}

impl<T: SpecialRenderPipeline> WinitRunner<'_, T> {
    pub fn new(render: T) -> Self {
        Self {
            renderer: Some(Renderer::new(render)),
        }
    }
}

impl<'a, T: SpecialRenderPipeline> ApplicationHandler for WinitRunner<'a, T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();
        if let Some(renderer) = &mut self.renderer {
            renderer.set_window(window);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    match renderer.render() {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if physical_size.width > 0 && physical_size.height > 0 {
                    if let Some(renderer) = &mut self.renderer {
                        renderer.resize(physical_size);
                    }
                }
            }
            _ => {}
        }
    }
}

pub struct App;

impl App {
    pub fn run(render: impl SpecialRenderPipeline) {
        let event_loop = winit::event_loop::EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = WinitRunner::new(render);

        event_loop.run_app(&mut app).expect("运行失败");
    }
}

pub trait SpecialRenderPipeline {
    fn special_render_pipeline(
        &self,
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> RenderPipeline;

    fn draw(&self, render_pass: RenderPass, device: &wgpu::Device);
}

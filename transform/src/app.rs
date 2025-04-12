use std::time::Instant;

use transform::State;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

#[derive(Default)]
pub struct App<'window> {
    window: Option<Window>,
    state: Option<State<'window>>,
    time: Option<Instant>,
}
impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        self.window = Some(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );
        if let Some(window) = &self.window {
            self.state = Some(futures::executor::block_on(State::new(window)).unwrap());
        }
        self.time = Some(Instant::now());
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
                if let Some(state) = &mut self.state {
                    state.update();
                    match state.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => {
                            dbg!("surface lost");
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                        }
                    }
                }
            }
            WindowEvent::Resized(physical_size) => {
                if let Some(state) = &mut self.state {
                    state.resize(physical_size);
                }
            }
            // 最小化
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        // 计时器，一秒中执行60次
        let new_time = Instant::now();
        let duration = new_time.duration_since(self.time.unwrap());
        if duration.as_secs_f32() > 1.0 / 60.0 {
            self.time = Some(new_time);
            if let Some(state) = &mut self.state {
                state.update();
                state.render().unwrap();
            }
        }
    }
}

impl App<'_> {
    pub fn run() {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App::default();
        let _ = event_loop.run_app(&mut app);
    }
}

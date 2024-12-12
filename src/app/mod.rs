use crate::{Engine, EngineCallbackHandler};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

pub mod feature_request;

#[allow(unused_variables)]
pub trait Application: EngineCallbackHandler {
    fn on_window_try_close(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, engine: &mut Engine) -> bool {
        true
    }
    fn on_window_close(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, engine: &mut Engine) {}

    fn on_create_windows(&mut self, event_loop: &ActiveEventLoop, engine: &mut Engine) {
    }

    fn on_done(&mut self, engine: &mut Engine) {}

    fn on_about_to_wait(&mut self, event_loop: &ActiveEventLoop, engine: &mut Engine) {}

    fn on_redraw_window(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, engine: &mut Engine) {}
}

pub struct ApplicationWrapper<A: Application> {
    app: A,
    engine: Option<Engine>,
}

impl<A: Application> ApplicationHandler for ApplicationWrapper<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let Some(mut engine) = self.engine.take() else { return; };

        self.app.on_create_windows(event_loop, &mut engine);

        self.engine = Some(engine);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(mut engine) = self.engine.take() else { return; };

        match event {
            WindowEvent::ActivationTokenDone { .. } => {}
            WindowEvent::Resized(_) => {}
            WindowEvent::Moved(_) => {}
            WindowEvent::CloseRequested => {
                if self.app.on_window_try_close(event_loop, window_id, &mut engine) {
                    engine.close_window(window_id);
                    self.app.on_window_close(event_loop, window_id, &mut engine);
                }
            },
            WindowEvent::Destroyed => {}
            WindowEvent::DroppedFile(_) => {}
            WindowEvent::HoveredFile(_) => {}
            WindowEvent::HoveredFileCancelled => {}
            WindowEvent::Focused(_) => {}
            WindowEvent::KeyboardInput { .. } => {}
            WindowEvent::ModifiersChanged(_) => {}
            WindowEvent::Ime(_) => {}
            WindowEvent::CursorMoved { .. } => {}
            WindowEvent::CursorEntered { .. } => {}
            WindowEvent::CursorLeft { .. } => {}
            WindowEvent::MouseWheel { .. } => {}
            WindowEvent::MouseInput { .. } => {}
            WindowEvent::PinchGesture { .. } => {}
            WindowEvent::PanGesture { .. } => {}
            WindowEvent::DoubleTapGesture { .. } => {}
            WindowEvent::RotationGesture { .. } => {}
            WindowEvent::TouchpadPressure { .. } => {}
            WindowEvent::AxisMotion { .. } => {}
            WindowEvent::Touch(_) => {}
            WindowEvent::ScaleFactorChanged { .. } => {}
            WindowEvent::ThemeChanged(_) => {}
            WindowEvent::Occluded(_) => {}
            WindowEvent::RedrawRequested => {
                self.app.on_redraw_window(event_loop, window_id, &mut engine);
            }
        }

        self.engine = Some(engine);
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // TODO: redraw requests here
        let Some(engine) = self.engine.take() else { return; };

        if engine.window_count() == 0 {
            event_loop.exit();
        } else {
            engine.windows().iter().for_each(|(_, window)| {
                window.borrow().window().request_redraw();
            })
        }

        self.engine = Some(engine);
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let Some(mut engine) = self.engine.take() else { return; };
        self.app.on_done(&mut engine);
        self.engine = Some(engine);
    }
}

impl<A: Application> ApplicationWrapper<A> {
    pub fn wrap(mut app: A, event_loop: &EventLoop<()>) -> anyhow::Result<Self> {
        let engine = Engine::init(event_loop, &mut app)?;

        Ok(Self {
            app,
            engine: Some(engine),
        })
    }
}

pub fn run<A: Application>(app: A) -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut wrapper = ApplicationWrapper::wrap(app, &event_loop)?;

    event_loop.run_app(&mut wrapper).map_err(anyhow::Error::from)
}

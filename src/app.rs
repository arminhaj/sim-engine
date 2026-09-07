use log::{error, info};
use std::error::Error;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle, WindowHandle, RawWindowHandle, HandleError, AppKitWindowHandle};
use wgpu::wgt::WgpuHasDisplayHandle;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

/// An enum representing the possible events we might want to send to the
/// window as a user.
#[derive(Debug)]
pub enum UserEvent {
    WakeUp,
}

#[derive(Debug)]
pub struct App {
    window: Option<Arc<Window>>,
    tx: Option<Sender<Arc<Window>>>,
}

impl App {
    /// Creates new App struct which implements ApplicationHandler.
    ///
    /// This does not initialize the window until event_loop.run_app() is called
    /// on the App struct, which consequently calls the resumed() ApplicationHandler implementation.
    pub fn new(tx: Sender<Arc<Window>>) -> App {
        App {
            window: None,
            tx: Some(tx),
        }
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> &mut Self {
        self.window = match event_loop.create_window(Window::default_attributes()) {
            Ok(t) => Some(Arc::new(t)),
            Err(e) => {
                error!("Error initializing window: {e}");
                None
            }
        };
        self
    }

    /// This function is responsible for sending the window to a worker thread down the
    /// oneshot channel, indicating that the application has been initialized.
    fn initialize_worker(&mut self) -> &mut Self {
        let _ = self
            .tx
            .take()
            .unwrap()
            .send(self.window.as_ref().unwrap().clone());
        self
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // initialize window and worker
        if self.window.is_none() {
            self.initialize_window(event_loop).initialize_worker();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            // closing the window
            WindowEvent::CloseRequested => event_loop.exit(),
            //
            _ => (),
        }

        // info!("{event_loop:?}");
        // info!("{window_id:?}");
        // info!("{event:?}");
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        info!("{event_loop:?}");
        info!("{event:?}");
    }
}

impl HasWindowHandle for App {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        match self.window {
            Some(_) => self.window.as_ref().unwrap().window_handle(),
            None => Err(HandleError::Unavailable),
        }
    }
}

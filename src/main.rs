use log::{debug, error, info, warn};
use std::error::Error;
use std::sync::{Arc, mpsc};

use winit::window::WindowId;
use winit::{
    application::ApplicationHandler,
    event::{WindowEvent, Event},
    event_loop::{ActiveEventLoop, EventLoop},
    window::Window,
};

use wgpu::{
    Instance,
    Surface,
    Adapter,
};

use winit::event_loop::{EventLoopBuilder, EventLoopProxy};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;

    let (tx, rx) = mpsc::channel();

    let window = None;
    let proxy = Some(event_loop.create_proxy());

    let mut app = App {
        window,
        proxy,
        tx,
    };

    debug!("Spawning worker thread");
    let handle = std::thread::spawn(move || {
        debug!("Emitting UserEvent");

        // we cannot send a UserEvent until the main thread initializes the window
        let proxy = rx.recv().unwrap();

        debug!("Obtained proxy {proxy:?}");
        let _ = proxy.send_event(UserEvent::WakeUp);
    });

    debug!("Running ApplicationHandler");
    event_loop.run_app(&mut app)?;


    let _ = handle.join();

    Ok(())
}

#[derive(Debug)]
enum UserEvent {
    WakeUp,
}

struct App {
    window: Option<Arc<Window>>,
    proxy: Option<EventLoopProxy<UserEvent>>,
    tx: mpsc::Sender<EventLoopProxy<UserEvent>>
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = match event_loop.create_window(Window::default_attributes()) {
                Ok(t) => Some(Arc::new(t)),
                Err(e) => {
                    error!("Error initializing window: {e}");
                    None
                }
            };

            self.tx.send(self.proxy.take().unwrap()).unwrap();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
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


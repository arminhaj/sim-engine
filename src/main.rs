pub mod app;
pub mod draw;

use app::{App, UserEvent};

use log::{debug, error, info, warn};
use std::sync::Arc;
use std::error::Error;

use tokio::sync::oneshot;

use winit::{
    application::ApplicationHandler,
    event::{Event, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop, EventLoopProxy},
    window::Window,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // initialize logger
    env_logger::init();

    // instantiate ApplicationHandler implementor
    let (tx, rx) = oneshot::channel();
    let mut app = App::new(tx);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    debug!("Spawning worker thread");
    let handle = std::thread::spawn(move || {
        debug!("Emitting UserEvent");

        // wait to receive the Window from App::resumed()
        let window = rx.blocking_recv();

        // now we can safely use the proxy.
        debug!("Obtained proxy {proxy:?}");
        let _ = proxy.send_event(UserEvent::WakeUp);

        worker_main();
    });

    // tell the event loop to start running our ApplicationHandler implementor
    debug!("Running app");
    event_loop.run_app(&mut app)?;

    Ok(())
}

/// Serve as an entry point for the worker thread
fn worker_main() {
    debug!("This is the worker thread's entry point to the program.");

    
}
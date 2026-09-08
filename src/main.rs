pub mod app;
pub mod draw;

use app::{App, UserEvent};

use log::debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use winit::error::EventLoopError;
use winit::{event_loop::EventLoop, window::Window};

#[tokio::main]
async fn main() -> anyhow::Result<(), EventLoopError> {
    // initialize logger
    env_logger::init();

    // instantiate ApplicationHandler implementor
    let (tx, rx) = oneshot::channel();
    let mut app = App::new(tx);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    debug!("Spawning task");
    tokio::spawn(async move {
        debug!("Emitting UserEvent");

        // wait to receive the Window from App::resumed()
        let window = rx.blocking_recv().unwrap();

        // now we can safely use the proxy.
        debug!("Obtained window {window:?}");
        let _ = proxy.send_event(UserEvent::WakeUp);

        worker_main(window);
    });

    event_loop.run_app(&mut app)
}

/// Serve as an entry point for the worker thread
fn worker_main(window: Arc<Window>) {
    debug!("This is the worker thread's entry point to the program.");
    unimplemented!();

    let canvas = draw::Canvas::new(window).await;
}

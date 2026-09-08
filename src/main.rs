pub mod app;
pub mod draw;

use app::{App, UserEvent};

use log::debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use winit::{
    event_loop::EventLoop,
    window::Window,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // initialize logger
    env_logger::init();

    // instantiate ApplicationHandler implementor
    let (tx, rx) = oneshot::channel();
    let mut app = App::new(tx);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();

    debug!("Spawning thread");
    let handle = std::thread::spawn(async move || -> anyhow::Result<()> {
        debug!("Emitting UserEvent");

        // wait to receive the Window from App::resumed()
        let window = rx.blocking_recv()?;

        // now we can safely use the proxy.
        debug!("Obtained window {window:?}");
        let _ = proxy.send_event(UserEvent::WakeUp);

        worker_main(window).await;
        Ok(())
    });


    let _ = event_loop.run_app(&mut app);

    let join_result = handle.join();
    if let Err(err) = join_result {
        std::panic::resume_unwind(err);
    }

    let worker_result = join_result.unwrap().await;
    worker_result
}

/// Serve as an entry point for the worker thread
async fn worker_main(window: Arc<Window>) {
    debug!("This is the worker thread's entry point to the program.");
    let canvas = draw::Canvas::new(window).await;
    
}
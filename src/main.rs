#![allow(warnings)]
pub mod app;

use app::{App, State, UserEvent};

use anyhow::Context;
use pollster::FutureExt;
use std::sync::Arc;
use tokio::runtime::{Handle, Runtime};
use tokio::sync::oneshot;
use tokio::task;
use tokio::task::LocalSet;
use tracing::{Event, Level, event, info, instrument, span, trace};
use tracing::{debug, error};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, FmtSubscriber, filter, fmt};
use winit::error::EventLoopError;
use winit::event_loop::{EventLoopProxy, OwnedDisplayHandle};
use winit::{event_loop::EventLoop, window::Window};

#[instrument]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _global_subscriber = FmtSubscriber::builder()
        .pretty()
        .with_target(true)
        .with_env_filter(EnvFilter::new("info,graphics=trace"))
        .with_span_events(FmtSpan::ACTIVE)
        .with_thread_names(true)
        .init();

    let (tx, rx) = oneshot::channel();

    // instantiate ApplicationHandler implementor
    let mut app = App::new(tx);
    debug!(?app);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    trace!("Built event loop.");

    let proxy = event_loop.create_proxy();
    trace!("Created an event loop proxy.");

    // spawn a blocking task to wait for window initialization
    let _join_handle = task::spawn_blocking(move || -> anyhow::Result<()> {
        let _window = rx.blocking_recv()?;

        // now we can safely use the proxy.
        if let Err(err) = proxy.send_event(UserEvent::InitializeState) {
            error!(?err, "Event loop is closed");
        }
        info!("Sent user event");

        Ok(())
    });

    event_loop.run_app(&mut app).map_err(|err| err.into())
}

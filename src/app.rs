use anyhow::{Context, anyhow};
use pollster::FutureExt;
use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Handle;
use tokio::sync::oneshot::Sender;
use tokio::task;
use tracing::{Level, debug, error, event, info, warn};
use tracing::{instrument, trace};
use wgpu::rwh::{
    AppKitWindowHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle,
    WindowHandle,
};
use wgpu::wgt::WgpuHasDisplayHandle;
use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, Surface,
};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};

/// An enum representing the possible events we might want to send to the
/// window as a user.
#[derive(Debug)]
pub enum UserEvent {
    WakeUp,
    InitializeState,
}

#[derive(Debug)]
pub struct App {
    oneshot: Option<Sender<Arc<Window>>>,
    window: Option<Arc<Window>>,
    state: Option<State>,
}

impl App {
    #[instrument]
    pub fn new(tx: Sender<Arc<Window>>) -> App {
        App {
            oneshot: Some(tx),
            window: None,
            state: None,
        }
    }

    fn initialize_window(&mut self, event_loop: &ActiveEventLoop) -> &mut Self {
        self.window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Some(Arc::new(window)),
            Err(err) => {
                error!(?err, "Error initializing window");
                None
            }
        };
        trace!(?self.window, "Successfully initialized window");
        self
    }

    /// This function is responsible for sending the window to a worker thread down the
    /// oneshot channel, indicating that the application has been initialized.
    ///
    /// If the channel has already been consumed, this function does nothing.
    #[instrument]
    fn send_window_down_oneshot(&mut self) -> &mut Self {
        let Some(oneshot) = self.oneshot.take() else {
            warn!("Oneshot channel has already been consumed!");
            return self;
        };

        if let Err(_) = oneshot.send(self.window.as_ref().unwrap().clone()) {
            error!("Failed to send window over oneshot channel.");
        }

        trace!("Successfully transmitted window over oneshot channel");
        self
    }

    // pub fn window(&self) -> Arc<Window> {
    //     self.window
    //         .as_ref()
    //         .ok_or_else(|| {
    //             anyhow!("attempted to access window before the application is initialized.")
    //         })
    //         .expect("this function should only be called after the application is run")
    //         .clone()
    // }
}

impl ApplicationHandler<UserEvent> for App {
    #[instrument]
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // initialize window and worker
        if self.window.is_none() {
            trace!("Obtained first resume event");
            self.initialize_window(event_loop)
                .send_window_down_oneshot();
        } else {
            trace!("Obtained resume event")
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

            _ => (),
        }
    }

    #[instrument]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let window = self.window.as_ref().unwrap().clone();
        match event {
            UserEvent::InitializeState => {
                self.state = State::new(window, event_loop)
                    .inspect(|state| {
                        debug!(?state);
                    })
                    .inspect_err(|err| {
                        error!(%err, "failed to initialize state from user event");
                    })
                    .ok();
            }
            _ => (),
        };
    }
}

#[derive(Debug)]
pub struct State {
    pub surface: Surface<'static>,
    pub device: Device,
    pub queue: Queue,
}

impl State {
    fn request_adapter(
        instance: &Instance,
        adapter_options: &RequestAdapterOptions,
    ) -> anyhow::Result<Adapter> {
        let adapter = instance
            .request_adapter(&adapter_options)
            .block_on()
            .inspect_err(|err| {
                error!(%err);
            })
            .context("requesting adapter")?;
        debug!(?adapter);

        Ok(adapter)
    }

    fn request_device_and_queue(
        adapter: &Adapter,
        device_descriptor: &DeviceDescriptor,
    ) -> anyhow::Result<(Device, Queue)> {
        let (device, queue) = adapter
            .request_device(&device_descriptor)
            .block_on()
            .inspect_err(|err| {
                error!(%err);
            })
            .context("requesting device")?;
        debug!(?device);

        Ok((device, queue))
    }

    fn new(window: Arc<Window>, event_loop: &ActiveEventLoop) -> anyhow::Result<State> {
        // create instance
        let instance_descriptor = InstanceDescriptor::new_with_display_handle(Box::new(
            event_loop.owned_display_handle(),
        ));
        let instance = Instance::new(instance_descriptor);
        event!(Level::DEBUG, ?instance);

        let adapter_options = RequestAdapterOptions {
            power_preference: PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: None,
            apply_limit_buckets: false,
        };
        let device_descriptor = DeviceDescriptor {
            label: Some("Metal"),
            required_features: Default::default(),
            required_limits: Default::default(),
            experimental_features: Default::default(),
            memory_hints: Default::default(),
            trace: Default::default(),
        };

        let adapter = Self::request_adapter(&instance, &adapter_options)
            .inspect_err(|err| error!(%err))
            .context("requesting adapter")?;
        let (device, queue) = Self::request_device_and_queue(&adapter, &device_descriptor)
            .inspect_err(|err| error!(%err))
            .context("requesting device")?;
        let surface = instance
            .create_surface(window)
            .inspect_err(|err| {
                error!(%err);
            })
            .context("creating surface")?;
        debug!(?surface);

        Ok(State {
            surface,
            device,
            queue,
        })
    }
}

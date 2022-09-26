#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]

use futures::executor::block_on;
use log::{info, trace};
use raw_window_handle::HasRawWindowHandle;

use tuber_ecs::ecs::Ecs;

pub type GraphicsResult<T> = Result<T, GraphicsError>;

#[derive(Debug, Clone)]
pub enum GraphicsError {
    SurfaceError(wgpu::SurfaceError),
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub trait GraphicsAPI {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()>;
}

pub struct Graphics {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    _window_size: WindowSize,
}

impl Graphics {
    pub fn new<Window>(window: &Window, window_size: WindowSize) -> Self
    where
        Window: HasRawWindowHandle,
    {
        info!("Initializing graphics API");
        let instance = Self::create_wgpu_instance();
        let surface = Self::create_render_surface(&instance, window);
        let adapter = Self::request_adapter(&instance, &surface);
        Self::log_adapter_details(&adapter);
        let (device, queue) = Self::request_device(&adapter);
        Self::configure_surface(&window_size, &surface, &adapter, &device);
        info!("Graphics API has been initialized successfully");

        Self {
            device,
            queue,
            surface,
            _window_size: window_size,
        }
    }

    fn create_wgpu_instance() -> wgpu::Instance {
        info!("Creating wgpu:: instance");
        wgpu::Instance::new(wgpu::Backends::all())
    }

    fn create_render_surface<Window>(instance: &wgpu::Instance, window: &Window) -> wgpu::Surface
    where
        Window: HasRawWindowHandle,
    {
        info!("Creating render surface");
        // Safety: The window is created by the engine and is valid
        // for as long as the engine is running
        unsafe { instance.create_surface(&window) }
    }

    fn request_adapter(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::Adapter {
        info!("Requesting video adapter");
        block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
        .unwrap()
    }

    fn request_device(adapter: &wgpu::Adapter) -> (wgpu::Device, wgpu::Queue) {
        info!("Requesting device");
        block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                ..Default::default()
            },
            None,
        ))
        .unwrap()
    }

    fn configure_surface(
        window_size: &WindowSize,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) {
        info!("Configuring render surface");
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(adapter)[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(device, &surface_configuration);
    }

    fn log_adapter_details(adapter: &wgpu::Adapter) {
        let adapter_details = adapter.get_info();
        info!("Adapter name: {}", adapter_details.name);
        info!("Adapter backend: {:?}", adapter_details.backend);
        info!("Adapter type: {:?}", adapter_details.device_type);
    }
}

impl GraphicsAPI for Graphics {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()> {
        trace!("Starting scene render");
        let output = self
            .surface
            .get_current_texture()
            .map_err(GraphicsError::SurfaceError)?;
        let _view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
        trace!("Render finished");

        Ok(())
    }
}

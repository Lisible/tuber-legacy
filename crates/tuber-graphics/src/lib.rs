use std::any::TypeId;

use futures::executor::block_on;
use futures::io;
use log::*;
use raw_window_handle::HasRawWindowHandle;

use tuber_core::asset::GenericLoader;
use tuber_ecs::ecs::Ecs;

use crate::resources::Resources;
use crate::textures::{texture_loader, TextureAsset};
use crate::wgpu::*;

mod wgpu;
mod resources;
mod textures;
mod shaders;

pub type GraphicsResult<T> = Result<T, GraphicsError>;

#[derive(Debug)]
pub enum GraphicsError {
    SurfaceError(WGPUSurfaceError),
    TextureFileOpenError(io::Error),
    ImageDecodeError(image::ImageError),
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub trait GraphicsAPI {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()>;
    fn loaders() -> Vec<(TypeId, GenericLoader)>;
}

pub struct Graphics {
    device: WGPUDevice,
    queue: WGPUQueue,
    surface: WGPUSurface,
    _window_size: WindowSize,
    _resources: Resources,
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
        let resources = Self::create_resources();
        info!("Graphics API has been initialized successfully");

        Self {
            device,
            queue,
            surface,
            _window_size: window_size,
            _resources: resources,
        }
    }

    pub fn create_resources() -> Resources {
        Resources::default()
    }

    fn create_wgpu_instance() -> WGPUInstance {
        info!("Creating WGPU instance");
        WGPUInstance::new(WGPUBackends::all())
    }

    fn create_render_surface<Window>(instance: &WGPUInstance, window: &Window) -> WGPUSurface
        where
            Window: HasRawWindowHandle,
    {
        info!("Creating render surface");
        // Safety: The window is created by the engine and is valid
        // for as long as the engine is running
        unsafe { instance.create_surface(&window) }
    }

    fn request_adapter(instance: &WGPUInstance, surface: &WGPUSurface) -> WGPUAdapter {
        info!("Requesting video adapter");
        block_on(instance.request_adapter(&WGPURequestAdapterOptions {
            power_preference: WGPUPowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
            .unwrap()
    }

    fn request_device(adapter: &WGPUAdapter) -> (WGPUDevice, WGPUQueue) {
        info!("Requesting device");
        block_on(adapter.request_device(
            &WGPUDeviceDescriptor {
                label: None,
                limits: if cfg!(target_arch = "wasm32") {
                    WGPULimits::downlevel_webgl2_defaults()
                } else {
                    WGPULimits::default()
                },
                ..Default::default()
            },
            None,
        ))
            .unwrap()
    }

    fn configure_surface(
        window_size: &WindowSize,
        surface: &WGPUSurface,
        adapter: &WGPUAdapter,
        device: &WGPUDevice,
    ) {
        info!("Configuring render surface");
        let surface_configuration = WGPUSurfaceConfiguration {
            usage: WGPUTextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(adapter)[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: WGPUPresentMode::Fifo,
        };
        surface.configure(device, &surface_configuration);
    }

    fn log_adapter_details(adapter: &WGPUAdapter) {
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
            .create_view(&WGPUTextureViewDescriptor::default());
        let command_encoder =
            self.device
                .create_command_encoder(&WGPUCommandEncoderDescriptor {
                    label: Some("command_encoder"),
                });

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
        trace!("Render finished");

        Ok(())
    }

    fn loaders() -> Vec<(TypeId, GenericLoader)> {
        vec!((TypeId::of::<TextureAsset>(), Box::new(texture_loader)))
    }
}

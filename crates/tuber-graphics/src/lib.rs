use std::any::TypeId;

use futures::executor::block_on;
use futures::io;
use log::*;
use raw_window_handle::HasRawWindowHandle;

use tuber_core::asset::{AssetStore, GenericLoader};
use tuber_ecs::ecs::Ecs;

use crate::render_graph::{RenderGraph, RenderGraphResources};
use crate::shaders::{shader_loader, Shader, ShaderAsset};
use crate::textures::{texture_loader, TextureAsset};
use crate::wgpu::*;

mod render_graph;
mod shaders;
mod textures;
mod wgpu;

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
    fn render_scene(&mut self, _ecs: &Ecs, asset_store: &mut AssetStore) -> GraphicsResult<()>;
    fn loaders() -> Vec<(TypeId, GenericLoader)>;
}

pub struct Graphics {
    device: WGPUDevice,
    queue: WGPUQueue,
    surface: WGPUSurface,
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
    fn render_scene(&mut self, _ecs: &Ecs, asset_store: &mut AssetStore) -> GraphicsResult<()> {
        trace!("Starting scene render");
        let output = self
            .surface
            .get_current_texture()
            .map_err(GraphicsError::SurfaceError)?;
        let view = output
            .texture
            .create_view(&WGPUTextureViewDescriptor::default());

        let mut render_graph_resources = RenderGraphResources::new();
        let view = render_graph_resources.import_texture_view(view);
        let mut render_graph = RenderGraph::new(&render_graph_resources, &self.device);

        render_graph
            .add_pass("draw_test_triangle")
            .with_color_attachment(view, ClearColor::from_rgba(1.0, 0.0, 0.0, 0.0))
            .dispatch(|_rpass| {});
        render_graph.compile();

        let mut command_encoder =
            self.device
                .create_command_encoder(&WGPUCommandEncoderDescriptor {
                    label: Some("command_encoder"),
                });
        render_graph.execute(&mut command_encoder);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
        trace!("Render finished");

        Ok(())
    }

    fn loaders() -> Vec<(TypeId, GenericLoader)> {
        vec![
            (TypeId::of::<TextureAsset>(), Box::new(texture_loader)),
            (TypeId::of::<ShaderAsset>(), Box::new(shader_loader)),
        ]
    }
}

pub struct ClearColor {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl ClearColor {
    pub fn from_rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}

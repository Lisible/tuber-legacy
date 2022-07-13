use futures::executor::block_on;
use log::*;
use raw_window_handle::HasRawWindowHandle;
use wgpu::{
    Adapter as WGPUAdapter, Backends as WGPUBackends,
    CommandEncoderDescriptor as WGPUCommandEncoderDescriptor, Device as WGPUDevice,
    DeviceDescriptor as WGPUDeviceDescriptor, Instance as WGPUInstance, Limits as WGPULimits,
    PowerPreference as WGPUPowerPreference, PresentMode as WGPUPresentMode, Queue as WGPUQueue,
    RequestAdapterOptions as WGPURequestAdapterOptions, Surface as WGPUSurface,
    SurfaceConfiguration as WGPUSurfaceConfiguration, SurfaceError as WGPUSurfaceError,
    TextureUsages as WGPUTextureUsages, TextureViewDescriptor as WGPUTextureViewDescriptor,
};

use tuber_ecs::ecs::Ecs;

use crate::render_graph::{RenderGraph, RenderPassDescriptor};
use crate::render_resource::{RenderResourceSource, RenderResourceStore};

mod render_graph;
mod render_resource;

pub type GraphicsResult<T> = Result<T, GraphicsError>;

#[derive(Debug, Clone)]
pub enum GraphicsError {
    SurfaceError(WGPUSurfaceError),
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub trait GraphicsAPI {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()>;
}

pub struct Graphics {
    device: WGPUDevice,
    queue: WGPUQueue,
    surface: WGPUSurface,
    _window_size: WindowSize,
    render_resource_store: RenderResourceStore,
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
            render_resource_store: RenderResourceStore::default(),
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
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()> {
        trace!("Starting scene render");
        let output = self
            .surface
            .get_current_texture()
            .map_err(GraphicsError::SurfaceError)?;
        let view = output
            .texture
            .create_view(&WGPUTextureViewDescriptor::default());
        let current_surface_texture_view_handle = self
            .render_resource_store
            .store_current_surface_texture_view(view);
        let mut command_encoder =
            self.device
                .create_command_encoder(&WGPUCommandEncoderDescriptor {
                    label: Some("command_encoder"),
                });

        let mut render_graph = RenderGraph::new(
            &self.device,
            &mut command_encoder,
            &self.render_resource_store,
        );

        let _pass_handle0 = render_graph.add_render_pass(RenderPassDescriptor {
            label: "do_nothing",
            inputs: vec![],
            outputs: vec![RenderResourceSource::RenderResource(
                current_surface_texture_view_handle,
            )],
            vertex_shader: None,
            fragment_shader: None,
            dispatch: Box::new(|_rpass| {}),
        });

        let _pass_handle = render_graph.add_render_pass(RenderPassDescriptor {
            label: "clear_render_target",
            inputs: vec![],
            outputs: vec![RenderResourceSource::PassOutput(_pass_handle0, 0)],
            vertex_shader: None,
            fragment_shader: None,
            dispatch: Box::new(|_rpass| {}),
        });

        let execution_order = render_graph.compile();
        render_graph.dispatch(&execution_order);

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
        trace!("Render finished");

        Ok(())
    }
}

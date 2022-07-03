use futures::executor::block_on;
use log::*;
use raw_window_handle::HasRawWindowHandle;
use wgpu::BindingResource::TextureView;
use wgpu::*;

use tuber_ecs::ecs::Ecs;

pub type GraphicsResult<T> = Result<T, GraphicsError>;

#[derive(Debug, Clone)]
pub enum GraphicsError {
    SurfaceError(SurfaceError),
}

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub trait GraphicsAPI {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()>;
}

pub struct Graphics {
    device: Device,
    queue: Queue,
    surface: Surface,
    window_size: WindowSize,
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
            window_size,
        }
    }

    fn create_wgpu_instance() -> Instance {
        info!("Creating WGPU instance");
        Instance::new(Backends::all())
    }

    fn create_render_surface<Window>(instance: &Instance, window: &Window) -> Surface
    where
        Window: HasRawWindowHandle,
    {
        info!("Creating render surface");
        // Safety: The window is created by the engine and is valid
        // for as long as the engine is running
        unsafe { instance.create_surface(&window) }
    }

    fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        info!("Requesting video adapter");
        block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap()
    }

    fn request_device(adapter: &Adapter) -> (Device, Queue) {
        info!("Requesting device");
        block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                limits: if cfg!(target_arch = "wasm32") {
                    Limits::downlevel_webgl2_defaults()
                } else {
                    Limits::default()
                },
                ..Default::default()
            },
            None,
        ))
        .unwrap()
    }

    fn configure_surface(
        window_size: &WindowSize,
        surface: &Surface,
        adapter: &Adapter,
        device: &Device,
    ) {
        info!("Configuring render surface");
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window_size.width,
            height: window_size.height,
            present_mode: PresentMode::Fifo,
        };
        surface.configure(&device, &surface_configuration);
    }

    fn log_adapter_details(adapter: &Adapter) {
        let adapter_details = adapter.get_info();
        info!("Adapter name: {}", adapter_details.name);
        info!("Adapter backend: {:?}", adapter_details.backend);
        info!("Adapter type: {:?}", adapter_details.device_type);
    }
}

impl GraphicsAPI for Graphics {
    fn render_scene(&mut self, _ecs: &Ecs) -> GraphicsResult<()> {
        info!("Starting scene render");
        let output = self
            .surface
            .get_current_texture()
            .map_err(GraphicsError::SurfaceError)?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });

        {
            let _render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 1.0,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();

        Ok(())
    }
}

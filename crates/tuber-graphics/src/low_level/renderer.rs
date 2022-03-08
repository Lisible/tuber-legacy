use crate::GraphicsError;
use crate::GraphicsResult;
use crate::Window;
use futures::executor::block_on;
use wgpu::*;

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    _surface_configuration: SurfaceConfiguration,
    _size: (u32, u32),
}
impl Renderer {
    /// Creates the renderer
    pub fn new(window: Window, window_size: (u32, u32)) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::POLYGON_MODE_LINE,
                limits: Limits::default(),
            },
            None,
        ))
        .unwrap();

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.0,
            height: window_size.1,
            present_mode: PresentMode::Fifo,
        };

        surface.configure(&device, &surface_configuration);

        Self {
            surface,
            device,
            queue,
            _surface_configuration: surface_configuration,
            _size: window_size,
        }
    }

    pub fn render(&mut self) -> GraphicsResult<()> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| GraphicsError::WGPUSurfaceError(e))?;
        let output_texture_view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let _render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &output_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();
        Ok(())
    }
}

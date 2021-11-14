use crate::TuberGraphicsWGPUError;
use futures::executor::block_on;
use tuber_graphics::{Window, WindowSize};

pub struct WGPUState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    size: WindowSize,
}

impl WGPUState {
    pub fn new(window: Window, window_size: WindowSize) -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Fifo,
        };

        Self {
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
        }
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        assert!(new_size.0 > 0);
        assert!(new_size.1 > 0);
        self.size = new_size;
        self.surface_configuration.width = new_size.0;
        self.surface_configuration.height = new_size.1;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    pub fn render(&mut self) -> Result<(), TuberGraphicsWGPUError> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| TuberGraphicsWGPUError::WGPUSurfaceError(e))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
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

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}

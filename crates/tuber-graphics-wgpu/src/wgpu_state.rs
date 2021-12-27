use crate::quad_renderer::QuadRenderer;
use crate::{DrawCommand, DrawCommandData, DrawType, TuberGraphicsWGPUError};
use futures::executor::block_on;
use std::collections::HashMap;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::QuadDescription;
use tuber_graphics::texture::{TextureData, TextureMetadata};
use tuber_graphics::{Window, WindowSize};

pub struct WGPUState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    size: WindowSize,
    quad_renderer: QuadRenderer,
    pending_draw_commands: Vec<DrawCommand>,
    textures_in_vram: HashMap<String, wgpu::Texture>,
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

        surface.configure(&device, &surface_configuration);

        let quad_renderer = QuadRenderer::new(&device, surface_configuration.format);

        Self {
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
            quad_renderer,
            textures_in_vram: HashMap::new(),
            pending_draw_commands: vec![],
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
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            for draw_command in &self.pending_draw_commands {
                {
                    match draw_command {
                        DrawCommand {
                            draw_command_data, ..
                        } if draw_command.draw_type() == DrawType::Quad => {
                            if let DrawCommandData::QuadDrawCommand(draw_command_data) =
                                draw_command_data
                            {
                                self.quad_renderer
                                    .render(&mut render_pass, draw_command_data);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        self.pending_draw_commands.clear();
        self.quad_renderer.clear_pending_quads();
        Ok(())
    }

    pub(crate) fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
    ) {
        self.pending_draw_commands.push(self.quad_renderer.prepare(
            &self.device,
            &self.queue,
            quad_description,
            transform,
            &self.textures_in_vram,
        ));
    }

    pub(crate) fn update_camera(
        &mut self,
        _camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        self.quad_renderer
            .set_camera(&self.queue, camera, transform);
    }

    pub(crate) fn load_texture_in_vram(&mut self, texture_data: &TextureData) {
        use crate::texture;
        let texture = texture::create_texture_from_data(&self.device, &self.queue, &texture_data);
        self.textures_in_vram
            .insert(texture_data.identifier.clone(), texture);
    }

    pub(crate) fn is_texture_in_vram(&self, texture_identifier: &str) -> bool {
        self.textures_in_vram.contains_key(texture_identifier)
    }
}

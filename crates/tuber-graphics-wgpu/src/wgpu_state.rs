use crate::composition::Compositor;
use crate::g_buffer::GBuffer;
use crate::quad_renderer::QuadRenderer;
use crate::texture::{
    create_texture_bind_group, create_texture_bind_group_layout, TextureBindGroup,
};
use crate::{DrawCommand, DrawCommandData, DrawType, TuberGraphicsWGPUError};
use futures::executor::block_on;
use std::collections::HashMap;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::g_buffer::GBufferComponent;
use tuber_graphics::low_level::QuadDescription;
use tuber_graphics::texture::TextureData;
use tuber_graphics::{Color, Window, WindowSize};
use wgpu::SurfaceTexture;

pub struct WGPUState {
    clear_color: Color,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    size: WindowSize,
    quad_renderer: QuadRenderer,
    compositor: Compositor,
    pending_draw_commands: Vec<DrawCommand>,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    textures_in_vram: HashMap<String, wgpu::Texture>,
    texture_bind_groups: HashMap<String, TextureBindGroup>,
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
        let compositor = Compositor::new(&device, surface_configuration.format);
        let texture_bind_group_layout = create_texture_bind_group_layout(&device);

        Self {
            clear_color: (0.0, 0.0, 0.0),
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
            quad_renderer,
            compositor,
            textures_in_vram: HashMap::new(),
            texture_bind_groups: HashMap::new(),
            pending_draw_commands: vec![],
            texture_bind_group_layout,
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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render_encoder"),
            });

        let g_buffer = self.geometry_pass(&mut encoder);
        self.compositor.prepare(&self.device, g_buffer);
        let output = self.composition_pass(&mut encoder).unwrap();

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        self.pending_draw_commands.clear();
        self.quad_renderer.clear_pending_quads();
        Ok(())
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent) {
        self.compositor
            .set_rendered_g_buffer_component(&self.queue, g_buffer_component);
    }

    fn composition_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<SurfaceTexture, TuberGraphicsWGPUError> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| TuberGraphicsWGPUError::WGPUSurfaceError(e))?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composition_pass"),
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

            self.compositor.render(&mut render_pass);
        }

        Ok(output)
    }

    fn create_g_buffer_texture_descriptor(&self, label: &'static str) -> wgpu::TextureDescriptor {
        wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: self.size.0,
                height: self.size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
        }
    }

    fn geometry_pass(&mut self, encoder: &mut wgpu::CommandEncoder) -> GBuffer {
        let albedo_map_texture_descriptor =
            self.create_g_buffer_texture_descriptor("albedo_map_texture");
        let albedo_map_texture = self.device.create_texture(&albedo_map_texture_descriptor);
        let albedo_map_view =
            albedo_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let normal_map_texture_descriptor =
            self.create_g_buffer_texture_descriptor("normal_map_texture");
        let normal_map_texture = self.device.create_texture(&normal_map_texture_descriptor);
        let normal_map_view =
            normal_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry_pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &albedo_map_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.clear_color.0 as f64,
                                g: self.clear_color.1 as f64,
                                b: self.clear_color.2 as f64,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                    wgpu::RenderPassColorAttachment {
                        view: &normal_map_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.5,
                                g: 0.5,
                                b: 1.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            for draw_command in &self.pending_draw_commands {
                self.render_draw_command(&mut render_pass, draw_command);
            }
        }

        GBuffer {
            albedo: albedo_map_texture,
            normal: normal_map_texture,
        }
    }

    fn render_draw_command<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        draw_command: &DrawCommand,
    ) {
        match draw_command {
            DrawCommand {
                draw_command_data, ..
            } if draw_command.draw_type() == DrawType::Quad => {
                if let DrawCommandData::QuadDrawCommand(draw_command_data) = draw_command_data {
                    self.quad_renderer.render(
                        render_pass,
                        draw_command_data,
                        &self.texture_bind_groups,
                    );
                }
            }
            _ => {}
        }
    }

    pub(crate) fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
    ) {
        self.pending_draw_commands.push(self.quad_renderer.prepare(
            &self.queue,
            quad_description,
            transform,
            &self.texture_bind_groups,
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

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = create_texture_bind_group(
            &self.device,
            &self.texture_bind_group_layout,
            &texture_view,
            &texture_sampler,
        );

        let texture_bind_group = TextureBindGroup {
            identifier: texture_data.identifier.to_string(),
            size: texture_data.size,
            bind_group,
        };

        self.texture_bind_groups
            .insert(texture_data.identifier.clone(), texture_bind_group);
        self.textures_in_vram
            .insert(texture_data.identifier.clone(), texture);
    }

    pub(crate) fn is_texture_in_vram(&self, texture_identifier: &str) -> bool {
        self.textures_in_vram.contains_key(texture_identifier)
    }
}

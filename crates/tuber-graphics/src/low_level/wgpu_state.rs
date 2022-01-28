use crate::camera::OrthographicCamera;
use crate::draw_command::{CommandBuffer, DrawQuadCommand};
use crate::g_buffer::GBufferComponent;
use crate::graphics::RenderId;
use crate::light_renderer::LightRenderer;
use crate::low_level::composition::Compositor;
use crate::low_level::g_buffer::GBuffer;
use crate::low_level::polygon_mode::PolygonMode;
use crate::low_level::primitives::{MaterialDescription, TextureId};
use crate::low_level::quad_renderer::QuadRenderer;
use crate::low_level::texture::create_texture_descriptor;
use crate::primitives::Quad;
use crate::quad_renderer::QuadRenderPassType;
use crate::{low_level, Color, Size2, TextureData, Window, WindowSize};
use futures::executor::block_on;
use nalgebra::Matrix4;
use std::collections::HashMap;
use tuber_ecs::EntityIndex;
use wgpu::{CommandEncoderDescriptor, SurfaceTexture, TextureViewDescriptor};

pub struct WGPUState {
    clear_color: Color,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    size: WindowSize,
    quad_renderer: QuadRenderer,
    light_renderer: LightRenderer,
    compositor: Compositor,

    next_texture_id: usize,
    textures: HashMap<TextureId, wgpu::Texture>,

    projection_matrix: Matrix4<f32>,
    view_transform: Matrix4<f32>,

    pre_renders: Vec<PreRender>,

    command_buffer: CommandBuffer,
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
                features: wgpu::Features::POLYGON_MODE_LINE,
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &surface_configuration);

        let quad_renderer = QuadRenderer::new(&device, surface_configuration.format);
        let light_renderer = LightRenderer::new(&device, surface_configuration.format);
        let compositor = Compositor::new(&device, surface_configuration.format);

        Self {
            clear_color: Color::BLACK.into(),
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
            quad_renderer,
            light_renderer,
            compositor,

            textures: HashMap::new(),
            next_texture_id: 0,

            projection_matrix: Matrix4::identity(),
            view_transform: Matrix4::identity(),
            pre_renders: vec![],
            command_buffer: CommandBuffer::new(),
        }
    }

    pub fn allocate_pre_render(&mut self, size_pixel: Size2<u32>) -> RenderId {
        let material = self.allocate_material(size_pixel);
        self.pre_renders.push(PreRender {
            size: Size2::new(size_pixel.width as f32, size_pixel.height as f32),
            material,
        });

        RenderId(self.pre_renders.len() - 1)
    }

    fn allocate_material(&mut self, size_pixel: Size2<u32>) -> MaterialDescription {
        MaterialDescription {
            albedo_map_id: self.allocate_texture(size_pixel, wgpu::TextureFormat::Bgra8UnormSrgb),
            normal_map_id: self.allocate_texture(size_pixel, wgpu::TextureFormat::Rgba8Unorm),
            emission_map_id: self.allocate_texture(size_pixel, wgpu::TextureFormat::Rgba8Unorm),
        }
    }

    fn allocate_texture(
        &mut self,
        texture_size: Size2<u32>,
        format: wgpu::TextureFormat,
    ) -> TextureId {
        let texture_id = self.next_texture_id();
        let texture_descriptor = create_texture_descriptor(None, texture_size, format);
        self.textures
            .insert(texture_id, self.device.create_texture(&texture_descriptor));
        texture_id
    }

    fn next_texture_id(&mut self) -> TextureId {
        let texture_id = TextureId(self.next_texture_id);
        self.next_texture_id += 1;
        texture_id
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        assert!(new_size.width > 0);
        assert!(new_size.height > 0);
        self.size = new_size;
        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    pub fn command_buffer_mut(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }

    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    pub fn render(&mut self) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());
        self.pre_render_pass(&mut command_encoder);
        let ui_render = self.ui_pass(&mut command_encoder);
        let g_buffer = self.geometry_pass(&mut command_encoder);
        let lit_render = self.lighting_pass(&mut command_encoder, g_buffer);
        let final_render = self.composition_pass(&mut command_encoder, &lit_render, &ui_render);
        self.quad_renderer.finish_preparation(&self.queue);
        self.queue.submit(std::iter::once(command_encoder.finish()));

        final_render.present();

        self.quad_renderer.clear_pending_quads();
        self.command_buffer_mut().clear();
    }

    pub fn ui_pass(&mut self, command_encoder: &mut wgpu::CommandEncoder) -> wgpu::Texture {
        let render_texture_descriptor = self.create_g_buffer_texture_descriptor("render_texture");
        let render_texture = self.device.create_texture(&render_texture_descriptor);
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let quad_group = self.quad_renderer.prepare_quad_group(
            &self.device,
            &self.queue,
            &self.textures,
            &self.projection_matrix,
            &Matrix4::identity(),
            &self.command_buffer.draw_ui_quad_commands(),
            true,
        );

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.quad_renderer.render_quad_group(
                &mut render_pass,
                QuadRenderPassType::UI,
                &quad_group,
            )
        }
        render_texture
    }

    pub fn pre_render_pass(&mut self, command_encoder: &mut wgpu::CommandEncoder) {
        for command in self.command_buffer.pre_draw_quads_commands() {
            let pre_render = &self.pre_renders[command.render_id.0];
            let albedo_map_id = pre_render.material.albedo_map_id;
            let normal_map_id = pre_render.material.normal_map_id;
            let emission_map_id = pre_render.material.emission_map_id;

            let albedo_texture = &self.textures[&albedo_map_id];
            let normal_texture = &self.textures[&normal_map_id];
            let emission_texture = &self.textures[&emission_map_id];

            let albedo_texture_view = albedo_texture.create_view(&TextureViewDescriptor::default());
            let normal_texture_view = normal_texture.create_view(&TextureViewDescriptor::default());
            let emission_texture_view =
                emission_texture.create_view(&TextureViewDescriptor::default());

            let quad_group = self.quad_renderer.prepare_quad_group(
                &self.device,
                &self.queue,
                &self.textures,
                &Matrix4::new_orthographic(
                    0.0,
                    pre_render.size.width,
                    pre_render.size.height,
                    0.0,
                    -1.0,
                    1.0,
                ),
                &Matrix4::identity(),
                &command.draw_quad_commands,
                false,
            );
            {
                let mut render_pass =
                    command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("quad_pre_render_pass"),
                        color_attachments: &[
                            wgpu::RenderPassColorAttachment {
                                view: &albedo_texture_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                },
                            },
                            wgpu::RenderPassColorAttachment {
                                view: &normal_texture_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                },
                            },
                            wgpu::RenderPassColorAttachment {
                                view: &emission_texture_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Load,
                                    store: true,
                                },
                            },
                        ],
                        depth_stencil_attachment: None,
                    });

                self.quad_renderer.render_quad_group(
                    &mut render_pass,
                    QuadRenderPassType::PreRender,
                    &quad_group,
                )
            }
        }
    }

    fn geometry_pass(&mut self, command_encoder: &mut wgpu::CommandEncoder) -> GBuffer {
        let albedo_map_texture_descriptor =
            self.create_g_buffer_texture_descriptor("albedo_map_texture");
        let albedo_map_texture = self.device.create_texture(&albedo_map_texture_descriptor);
        let albedo_map_view =
            albedo_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let normal_map_texture_descriptor = create_texture_descriptor(
            Some("normal_map_texture"),
            Size2::from(self.size),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let normal_map_texture = self.device.create_texture(&normal_map_texture_descriptor);
        let normal_map_view =
            normal_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let emission_map_texture_descriptor = create_texture_descriptor(
            Some("emission_map_texture"),
            Size2::from(self.size),
            wgpu::TextureFormat::Rgba8Unorm,
        );
        let emission_map_texture = self.device.create_texture(&emission_map_texture_descriptor);
        let emission_map_view =
            emission_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let position_map_texture_descriptor = create_texture_descriptor(
            Some("position_map_texture"),
            Size2::from(self.size),
            wgpu::TextureFormat::Rgba16Float,
        );
        let position_map_texture = self.device.create_texture(&position_map_texture_descriptor);
        let position_map_view =
            position_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut draw_commands = self
            .command_buffer()
            .draw_quad_commands()
            .iter()
            .cloned()
            .collect::<Vec<_>>();

        let mut render_draw_commands = self
            .command_buffer()
            .draw_pre_render_commands()
            .iter()
            .map(|pre_render_command| DrawQuadCommand {
                quad: Quad::with_size(pre_render_command.size),
                world_transform: pre_render_command.world_transform,
                material: self.pre_renders[pre_render_command.render_id.0]
                    .material
                    .clone(),
            })
            .collect::<Vec<_>>();

        draw_commands.append(&mut render_draw_commands);
        draw_commands.sort_by(|first_draw_command, second_draw_command| {
            (first_draw_command.world_transform.column(3).z as f32)
                .partial_cmp(&(second_draw_command.world_transform.column(3).z as f32))
                .unwrap()
        });

        let quad_group = self.quad_renderer.prepare_quad_group(
            &self.device,
            &self.queue,
            &self.textures,
            &self.projection_matrix,
            &self.view_transform,
            &draw_commands,
            false,
        );

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("geometry_pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &albedo_map_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: self.clear_color.r(),
                                g: self.clear_color.g(),
                                b: self.clear_color.b(),
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
                    wgpu::RenderPassColorAttachment {
                        view: &emission_map_view,
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
                    },
                    wgpu::RenderPassColorAttachment {
                        view: &position_map_view,
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
                    },
                ],
                depth_stencil_attachment: None,
            });

            self.quad_renderer.render_quad_group(
                &mut render_pass,
                QuadRenderPassType::Geometry,
                &quad_group,
            )
        }

        GBuffer {
            albedo: albedo_map_texture,
            normal: normal_map_texture,
            position: position_map_texture,
            emission: emission_map_texture,
        }
    }

    pub fn lighting_pass(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        g_buffer: GBuffer,
    ) -> wgpu::Texture {
        let render_texture_descriptor = self.create_g_buffer_texture_descriptor("render_texture");
        let render_texture = self.device.create_texture(&render_texture_descriptor);
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.light_renderer.prepare(
            &self.device,
            &self.queue,
            command_encoder,
            g_buffer,
            self.command_buffer.draw_light_commands(),
        );

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("lighting_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            self.light_renderer.render(&mut render_pass);
        }

        render_texture
    }

    fn composition_pass(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        lit_render: &wgpu::Texture,
        ui_render: &wgpu::Texture,
    ) -> SurfaceTexture {
        let output_texture = self.surface.get_current_texture().unwrap();
        let output_texture_view = output_texture
            .texture
            .create_view(&TextureViewDescriptor::default());
        self.compositor.prepare(&self.device, lit_render, ui_render);

        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composition_pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &output_texture_view,
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
        output_texture
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent) {
        self.compositor
            .set_rendered_g_buffer_component(&self.queue, g_buffer_component);
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: PolygonMode) {
        self.quad_renderer
            .set_polygon_mode(&self.device, polygon_mode);
    }

    pub(crate) fn update_camera(
        &mut self,
        _camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform_matrix: Matrix4<f32>,
    ) {
        self.projection_matrix = camera.projection_matrix();
        self.view_transform = transform_matrix;
    }

    pub(crate) fn load_texture_in_vram(&mut self, texture_data: &TextureData) -> TextureId {
        let texture_id = self.next_texture_id();
        self.textures.insert(
            texture_id,
            low_level::texture::create_texture_from_data(
                &self.device,
                &self.queue,
                texture_id,
                &texture_data,
            ),
        );
        texture_id
    }

    pub fn create_g_buffer_texture_descriptor(
        &self,
        label: &'static str,
    ) -> wgpu::TextureDescriptor {
        create_texture_descriptor(
            Some(label),
            Size2::from(self.size),
            wgpu::TextureFormat::Bgra8UnormSrgb,
        )
    }
}

pub struct PreRender {
    pub size: Size2,
    pub material: MaterialDescription,
}

pub trait IntoPolygonMode {
    fn into_polygon_mode(self) -> wgpu::PolygonMode;
}

impl IntoPolygonMode for PolygonMode {
    fn into_polygon_mode(self) -> wgpu::PolygonMode {
        match self {
            PolygonMode::Line => wgpu::PolygonMode::Line,
            PolygonMode::Fill => wgpu::PolygonMode::Fill,
        }
    }
}

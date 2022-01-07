use crate::composition::Compositor;
use crate::g_buffer::GBuffer;
use crate::quad_renderer::QuadRenderer;
use crate::texture::{
    create_texture_bind_group, create_texture_bind_group_layout, create_texture_descriptor,
};
use crate::TuberGraphicsWGPUError;
use futures::executor::block_on;
use nalgebra::Matrix4;
use tuber_core::transform::Transform2D;
use tuber_ecs::EntityIndex;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::g_buffer::GBufferComponent;
use tuber_graphics::low_level::polygon_mode::PolygonMode;
use tuber_graphics::low_level::primitives::{QuadDescription, TextureId};
use tuber_graphics::texture::TextureData;
use tuber_graphics::types::{Color, Size2, WindowSize};
use tuber_graphics::Window;
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
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,
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
            width: window_size.width(),
            height: window_size.height(),
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &surface_configuration);

        let quad_renderer = QuadRenderer::new(&device, surface_configuration.format);
        let compositor = Compositor::new(&device, surface_configuration.format);
        let texture_bind_group_layout = create_texture_bind_group_layout(&device);

        Self {
            clear_color: Color::BLACK.into(),
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
            quad_renderer,
            compositor,
            texture_bind_groups: vec![],
            texture_bind_group_layout,
        }
    }

    pub fn resize(&mut self, new_size: WindowSize) {
        assert!(new_size.width() > 0);
        assert!(new_size.height() > 0);
        self.size = new_size;
        self.surface_configuration.width = new_size.width();
        self.surface_configuration.height = new_size.height();
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    pub fn draw_quads(&mut self, quads: &[QuadDescription]) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("draw_quads_encoder"),
            });

        let g_buffer = self.geometry_pass(&mut encoder, quads);
        self.compositor.prepare(&self.device, g_buffer);
        let output = self.composition_pass(&mut encoder).unwrap();

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        self.quad_renderer.clear_pending_quads();
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

    fn geometry_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        quads: &[QuadDescription],
    ) -> GBuffer {
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

        self.quad_renderer.prepare(&self.device, &self.queue, quads);

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                ],
                depth_stencil_attachment: None,
            });

            self.quad_renderer
                .render(&mut render_pass, &self.texture_bind_groups)
        }

        GBuffer {
            albedo: albedo_map_texture,
            normal: normal_map_texture,
        }
    }

    pub(crate) fn update_camera(
        &mut self,
        _camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let projection_matrix = Matrix4::new_orthographic(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        );
        self.quad_renderer
            .set_projection_matrix(&self.queue, &projection_matrix, transform);
    }

    pub(crate) fn load_texture_in_vram(&mut self, texture_data: &TextureData) -> TextureId {
        use crate::texture;
        let texture_id = TextureId(self.texture_bind_groups.len());
        let texture =
            texture::create_texture_from_data(&self.device, &self.queue, texture_id, &texture_data);

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

        self.texture_bind_groups.push(bind_group);
        texture_id
    }

    pub fn create_g_buffer_texture_descriptor(
        &self,
        label: &'static str,
    ) -> wgpu::TextureDescriptor {
        create_texture_descriptor(label, Size2::from(self.size))
    }

    pub(crate) fn is_texture_in_vram(&self, texture_id: TextureId) -> bool {
        self.texture_bind_groups.len() > texture_id.0
    }
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

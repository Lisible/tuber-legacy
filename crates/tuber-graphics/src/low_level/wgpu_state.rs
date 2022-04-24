use std::collections::HashMap;

use futures::executor::block_on;
use wgpu::CommandEncoderDescriptor;

use tuber_ecs::EntityIndex;
use tuber_math::matrix::{Identity, Matrix4f};

use crate::camera::OrthographicCamera;
use crate::draw_command::CommandBuffer;
use crate::g_buffer::GBufferComponent;
use crate::low_level::composition::Compositor;
use crate::low_level::polygon_mode::PolygonMode;
use crate::low_level::primitives::TextureId;
use crate::low_level::render_passes::composition_pass::composition_pass;
use crate::low_level::render_passes::geometry_pass::geometry_pass;
use crate::low_level::render_passes::lighting_pass::lighting_pass;
use crate::low_level::render_passes::ui_pass::ui_pass;
use crate::low_level::renderers::light_renderer::LightRenderer;
use crate::low_level::renderers::mesh_renderer::MeshRenderer;
use crate::low_level::renderers::quad_renderer::QuadRenderer;
use crate::{low_level, Color, Size2, TextureData, Window, WindowSize};

pub struct WGPUState {
    clear_color: Color,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_configuration: wgpu::SurfaceConfiguration,
    size: WindowSize,

    quad_renderer: QuadRenderer,
    mesh_renderer: MeshRenderer,
    light_renderer: LightRenderer,
    compositor: Compositor,

    next_texture_id: usize,
    textures: HashMap<TextureId, wgpu::Texture>,

    projection_matrix: Matrix4f,
    view_transform: Matrix4f,

    command_buffer: CommandBuffer,

    ambient_light: Color,
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
        let mesh_renderer = MeshRenderer::new(&device, surface_configuration.format);
        let light_renderer = LightRenderer::new(&device, surface_configuration.format);
        let compositor = Compositor::new(&device, surface_configuration.format);

        Self {
            clear_color: Color::BLACK,
            surface,
            device,
            queue,
            surface_configuration,
            size: window_size,
            mesh_renderer,
            quad_renderer,
            light_renderer,
            compositor,

            textures: HashMap::new(),
            next_texture_id: 0,

            projection_matrix: Matrix4f::identity(),
            view_transform: Matrix4f::identity(),
            command_buffer: CommandBuffer::new(),

            ambient_light: Color::WHITE,
        }
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

    pub fn render(&mut self) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        let final_render = {
            let mut render_context = RenderContext {
                device: &self.device,
                queue: &self.queue,
                command_buffer: &self.command_buffer,
                viewport_size: self.size,
                textures: &self.textures,
                clear_color: self.clear_color,
                projection_matrix: &self.projection_matrix,
                view_transform: &self.view_transform,
                quad_renderer: &mut self.quad_renderer,
                mesh_renderer: &mut self.mesh_renderer,
                light_renderer: &mut self.light_renderer,
                compositor: &mut self.compositor,
            };

            let ui_render = ui_pass(&mut render_context, &mut command_encoder);
            let g_buffer = geometry_pass(&mut render_context, &mut command_encoder);
            let lit_render = lighting_pass(
                &mut render_context,
                &mut command_encoder,
                self.ambient_light,
                g_buffer,
            );
            composition_pass(
                &mut render_context,
                &mut command_encoder,
                &self.surface,
                &lit_render,
                &ui_render,
            )
        };

        self.quad_renderer
            .finish_preparation(&self.device, &mut command_encoder, &self.queue);
        self.queue.submit(std::iter::once(command_encoder.finish()));
        final_render.present();

        self.quad_renderer.clear_pending_quads();
        self.mesh_renderer.cleanup();
        self.command_buffer_mut().clear();
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn set_ambient_light(&mut self, ambient_light: Color) {
        self.ambient_light = ambient_light;
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
        transform_matrix: Matrix4f,
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
                texture_data,
            ),
        );
        texture_id
    }
}

pub(crate) struct RenderContext<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub command_buffer: &'a CommandBuffer,
    pub viewport_size: Size2<u32>,
    pub textures: &'a HashMap<TextureId, wgpu::Texture>,
    pub clear_color: Color,
    pub projection_matrix: &'a Matrix4f,
    pub view_transform: &'a Matrix4f,
    pub quad_renderer: &'a mut QuadRenderer,
    pub mesh_renderer: &'a mut MeshRenderer,
    pub light_renderer: &'a mut LightRenderer,
    pub compositor: &'a mut Compositor,
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

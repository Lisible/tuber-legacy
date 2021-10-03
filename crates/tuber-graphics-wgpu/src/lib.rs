use crate::quad_renderer::QuadRenderer;
use crate::texture::Texture;
use crate::tilemap_renderer::TilemapRenderer;
use futures;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Range;
use tuber_core::asset::AssetStore;
use tuber_core::tilemap::Tilemap;
use tuber_core::transform::Transform2D;
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::tilemap::TilemapRender;
use tuber_graphics::{
    low_level::LowLevelGraphicsAPI, low_level::QuadDescription, texture::Texture as TextureData,
    Color, Window, WindowSize,
};

mod quad_renderer;
mod texture;
mod tilemap_renderer;

#[derive(Debug)]
pub enum TuberGraphicsWGPUError {}

pub struct GraphicsWGPU {
    wgpu_state: Option<WGPUState>,
    draw_commands: Vec<DrawCommand>,
    textures: HashMap<String, Texture>,
    camera_id: Option<usize>,
    clear_color: Color,
}

pub struct WGPUState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    window_size: WindowSize,
    quad_renderer: QuadRenderer,
    tilemap_renderer: TilemapRenderer,
}

impl GraphicsWGPU {
    pub fn new() -> Self {
        Self {
            wgpu_state: None,
            draw_commands: vec![],
            textures: HashMap::new(),
            camera_id: None,
            clear_color: (0.0, 0.0, 0.0),
        }
    }
}

impl LowLevelGraphicsAPI for GraphicsWGPU {
    fn initialize(&mut self, window: Window, window_size: WindowSize, asset_store: &AssetStore) {
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = async {
            instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                })
                .await
        };
        let adapter = futures::executor::block_on(adapter).unwrap();

        let device_and_queue = async {
            adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features: wgpu::Features::empty(),
                        limits: wgpu::Limits::default(),
                        label: None,
                    },
                    None,
                )
                .await
        };
        let (device, queue) = futures::executor::block_on(device_and_queue).unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: window_size.0,
            height: window_size.1,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let format = sc_desc.format;

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let quad_renderer = QuadRenderer::new(&device, &queue, &format, &asset_store);
        let tilemap_renderer = TilemapRenderer::new(&device, &format);

        self.wgpu_state = Some(WGPUState {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            window_size,
            quad_renderer,
            tilemap_renderer,
        });
    }

    fn render(&mut self) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        let frame = state.swap_chain.get_current_frame().unwrap().output;
        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.draw_commands.sort();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
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
                }],
                depth_stencil_attachment: None,
            });

            for draw_command in &self.draw_commands {
                {
                    match draw_command {
                        DrawCommand {
                            draw_command_data, ..
                        } if draw_command.draw_type() == DrawType::Quad => {
                            if let DrawCommandData::QuadDrawCommand(draw_command_data) =
                                draw_command_data
                            {
                                state
                                    .quad_renderer
                                    .render(&mut render_pass, draw_command_data);
                            }
                        }
                        DrawCommand {
                            draw_command_data, ..
                        } if draw_command.draw_type() == DrawType::Tilemap => {
                            if let DrawCommandData::TilemapDrawCommand(draw_command_data) =
                                draw_command_data
                            {
                                state
                                    .tilemap_renderer
                                    .render(&mut render_pass, draw_command_data);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        state.queue.submit(std::iter::once(encoder.finish()));
        state.quad_renderer.cleanup();
        self.draw_commands.clear();
    }

    fn prepare_quad(
        &mut self,
        quad_description: &QuadDescription,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        self.draw_commands.push(state.quad_renderer.prepare(
            &state.device,
            &state.queue,
            quad_description,
            transform,
            apply_view_transform,
            &self.textures,
        ));
    }

    fn prepare_tilemap(
        &mut self,
        tilemap: &Tilemap,
        tilemap_render: &TilemapRender,
        transform: &Transform2D,
        asset_store: &AssetStore,
    ) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        let texture_atlas = asset_store
            .stored_asset(&tilemap_render.texture_atlas_identifier)
            .unwrap();
        if let Some(draw_command) = state.tilemap_renderer.prepare(
            &state.device,
            &state.queue,
            tilemap,
            tilemap_render,
            texture_atlas,
            transform,
            &self.textures,
        ) {
            self.draw_commands.push(draw_command);
        }
    }

    fn is_texture_in_vram(&self, texture_identifier: &str) -> bool {
        self.textures.contains_key(texture_identifier)
    }

    fn load_texture(&mut self, texture_data: &TextureData) {
        let state = self.wgpu_state.as_ref().expect("Graphics is uninitialized");
        let identifier = texture_data.identifier.clone();
        let texture =
            Texture::from_texture_data(&state.device, &state.queue, &texture_data).unwrap();
        self.textures.insert(identifier, texture);
    }

    fn update_camera(
        &mut self,
        camera_id: usize,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        self.camera_id = Some(camera_id);
        state
            .quad_renderer
            .set_camera(&state.queue, camera, transform);
        state
            .tilemap_renderer
            .set_camera(&state.queue, camera, transform);
    }

    fn set_clear_color(&mut self, color: (f32, f32, f32)) {
        self.clear_color = color;
    }

    fn on_window_resized(&mut self, new_size: WindowSize) {
        let state = self.wgpu_state.as_mut().expect("Graphics is uninitialized");
        state.window_size = new_size;
        state.sc_desc.width = new_size.0;
        state.sc_desc.height = new_size.1;
        state.swap_chain = state
            .device
            .create_swap_chain(&state.surface, &state.sc_desc);
    }
}

#[derive(Eq, PartialEq)]
pub struct DrawCommand {
    pub draw_command_data: DrawCommandData,
    pub z_order: i32,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub enum DrawCommandData {
    QuadDrawCommand(QuadDrawCommand),
    TilemapDrawCommand(TilemapDrawCommand),
}

impl DrawCommand {
    pub fn draw_type(&self) -> DrawType {
        match self.draw_command_data {
            DrawCommandData::QuadDrawCommand(_) => DrawType::Quad,
            DrawCommandData::TilemapDrawCommand(_) => DrawType::Tilemap,
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct QuadDrawCommand {
    pub draw_range: DrawRange,
    pub texture: Option<String>,
}

impl Ord for QuadDrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.texture.cmp(&other.texture)
    }
}

impl PartialOrd for QuadDrawCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

#[derive(Eq, PartialEq, PartialOrd)]
pub struct TilemapDrawCommand {
    pub tilemap_identifier: String,
}

impl Ord for TilemapDrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.tilemap_identifier.cmp(&other.tilemap_identifier)
    }
}

impl Ord for DrawCommand {
    fn cmp(&self, other: &Self) -> Ordering {
        let mut sort = self.z_order.cmp(&other.z_order);

        if sort == Ordering::Equal {
            sort = self.draw_command_data.cmp(&other.draw_command_data);
        }

        sort
    }
}

impl PartialOrd for DrawCommand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub enum DrawType {
    Quad,
    Tilemap,
}

#[derive(Eq, PartialEq)]
pub enum DrawRange {
    VertexIndexRange(Range<u32>),
    InstanceIndexRange(Range<u32>),
}

impl DrawRange {
    pub fn vertex_index_range(&self) -> Option<&Range<u32>> {
        match self {
            DrawRange::VertexIndexRange(range) => Some(range),
            _ => None,
        }
    }

    pub fn instance_index_range(&self) -> Option<&Range<u32>> {
        match self {
            DrawRange::InstanceIndexRange(range) => Some(range),
            _ => None,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float3,
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float2,
                    offset: std::mem::size_of::<[f32; 6]>() as wgpu::BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}

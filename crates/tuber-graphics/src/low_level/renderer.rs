use futures::executor::block_on;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::*;

use tuber_core::transform::{AsMatrix4, Transform};
use tuber_math::matrix::Identity;
use tuber_math::matrix::Matrix4f;

use crate::low_level::buffers::index_buffer::IndexBuffer;
use crate::low_level::buffers::uniform_buffer::UniformBuffer;
use crate::low_level::buffers::vertex_buffer::VertexBuffer;
use crate::low_level::mesh::Mesh;
use crate::low_level::primitives::{Index, Vertex};
use crate::low_level::texture_store::TextureStore;
use crate::GraphicsError;
use crate::GraphicsResult;
use crate::Window;

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    _surface_configuration: SurfaceConfiguration,
    _size: (u32, u32),

    render_pipeline: RenderPipeline,

    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    camera_buffer: Buffer,
    camera_bind_group: BindGroup,

    mesh_uniform_buffer: UniformBuffer<MeshUniform>,
    mesh_uniform_bind_group: BindGroup,
    mesh_metadata: Vec<MeshMetadata>,

    pending_vertices: Vec<Vertex>,
    pending_indices: Vec<Index>,
    pending_mesh_uniforms: Vec<MeshUniform>,

    texture_store: TextureStore,
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

        let vertex_buffer = VertexBuffer::with_capacity(&device, "vertex_buffer", 1000);
        let index_buffer = IndexBuffer::with_capacity(&device, "index_buffer", 100_000);
        let mesh_uniform_buffer = UniformBuffer::new(&device, "mesh_uniform_buffer", 100);

        let mut texture_store = TextureStore::new(&device);
        texture_store.load_texture_from_image_data(
            &device,
            &queue,
            "_placeholder",
            include_bytes!("../../textures/default_texture.png"),
        );
        texture_store.load_texture(&device, &queue, "_white", &[0xff, 0xff, 0xff, 0xff], 1, 1);
        texture_store.load_texture(&device, &queue, "_black", &[0x0, 0x0, 0x0, 0x0], 1, 1);

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("mesh_shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/mesh.wgsl").into()),
        });

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::default()]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let mesh_uniform_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("mesh_uniform_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(
                            std::mem::size_of::<MeshUniform>() as BufferAddress
                        ),
                    },
                    count: None,
                }],
            });

        let mesh_uniform_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("mesh_uniform_bind_group"),
            layout: &mesh_uniform_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer: mesh_uniform_buffer.buffer(),
                    offset: 0,
                    size: BufferSize::new(std::mem::size_of::<MeshUniform>() as BufferAddress),
                }),
            }],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mesh_render_pipeline_layout"),
            bind_group_layouts: &[
                texture_store.texture_bind_group_layout(),
                &camera_bind_group_layout,
                &mesh_uniform_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("mesh_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            _surface_configuration: surface_configuration,
            _size: window_size,

            render_pipeline,

            vertex_buffer,
            index_buffer,

            camera_buffer,
            camera_bind_group,

            mesh_uniform_buffer,
            mesh_uniform_bind_group,
            mesh_metadata: vec![],

            pending_vertices: vec![],
            pending_indices: vec![],
            pending_mesh_uniforms: vec![],
            texture_store,
        }
    }

    pub fn render(&mut self) -> GraphicsResult<()> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(GraphicsError::WGPUSurfaceError)?;
        let output_texture_view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        self.prepare_buffers(&mut command_encoder);

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &output_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);

            let placeholder_texture_bind_group = self
                .texture_store
                .texture_bind_group("_placeholder")
                .expect("Placeholder texture isn't loaded");
            for mesh_metadata in &self.mesh_metadata {
                let texture_bind_group = self
                    .texture_store
                    .texture_bind_group(&mesh_metadata.texture_identifier)
                    .unwrap_or(placeholder_texture_bind_group);

                render_pass.set_bind_group(0, texture_bind_group, &[]);
                render_pass.set_bind_group(
                    2,
                    &self.mesh_uniform_bind_group,
                    &[mesh_metadata.uniform_offset],
                );
                render_pass.draw_indexed(
                    mesh_metadata.start_index
                        ..(mesh_metadata.start_index + mesh_metadata.index_count) as u32,
                    0,
                    0..1,
                );
            }
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();

        self.clear_pending_meshes();
        self.vertex_buffer.clear();
        self.index_buffer.clear();
        self.mesh_uniform_buffer.clear();
        self.mesh_metadata.clear();
        Ok(())
    }

    fn prepare_buffers(&mut self, command_encoder: &mut CommandEncoder) {
        self.vertex_buffer.append_vertices(
            command_encoder,
            &self.device,
            &self.queue,
            &self.pending_vertices,
        );

        let index_count = self.pending_indices.len();

        // In order to conform to COPY_BUFFER_ALIGNMENT
        if self.pending_indices.len() % 2 != 0 {
            self.pending_indices.push(0);
        }

        self.index_buffer.append_indices(
            command_encoder,
            &self.device,
            &self.queue,
            &self.pending_indices,
            index_count,
        );

        self.mesh_uniform_buffer
            .append_uniforms(&self.queue, &self.pending_mesh_uniforms);
    }

    pub fn set_view_projection_matrix(&mut self, view_projection_matrix: Matrix4f) {
        let camera_uniform = CameraUniform {
            view_projection_matrix: view_projection_matrix.into(),
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera_uniform]),
        );
    }

    pub fn queue_mesh(
        &mut self,
        mesh: Mesh,
        world_transform: Transform,
        local_transform: Transform,
        texture_identifier: &str,
    ) {
        self.pending_vertices.extend_from_slice(&mesh.vertices);
        let mut start_index = *self.pending_indices.last().unwrap_or(&0);
        if start_index != 0 {
            start_index += 1;
        }

        let start = self.pending_indices.len();
        self.pending_indices.extend_from_slice(
            &mesh
                .indices
                .iter()
                .map(|index| start_index + index)
                .collect::<Vec<_>>(),
        );

        self.pending_mesh_uniforms.push(MeshUniform {
            world_transform: (world_transform.as_matrix4() * local_transform.as_matrix4()).into(),
            _padding: [0; 24],
        });

        self.mesh_metadata.push(MeshMetadata {
            uniform_offset: (self.mesh_metadata.len() * 256) as _,
            start_index: start as u32,
            index_count: mesh.indices.len() as u32,
            texture_identifier: texture_identifier.into(),
        });
    }

    fn clear_pending_meshes(&mut self) {
        self.pending_vertices.clear();
        self.pending_indices.clear();
        self.pending_mesh_uniforms.clear();
    }
}

struct MeshMetadata {
    pub uniform_offset: DynamicOffset,
    pub start_index: u32,
    pub index_count: u32,
    pub texture_identifier: String,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_projection_matrix: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        Self {
            view_projection_matrix: Matrix4f::identity().into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshUniform {
    world_transform: [[f32; 4]; 4],
    _padding: [u64; 24],
}

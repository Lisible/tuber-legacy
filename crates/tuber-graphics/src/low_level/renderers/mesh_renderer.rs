use crate::geometry::Vertex;
use crate::low_level::utils::{
    create_copyable_buffer, create_global_uniform_bind_group,
    create_global_uniform_bind_group_layout, create_global_uniform_buffer,
    create_uniform_bind_group,
};
use crate::primitives::{Index, Mesh};
use crate::Material;
use nalgebra::Matrix4;
use tuber_core::transform::Transform2D;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor, Buffer,
    BufferAddress, BufferSize, BufferUsages, Device, PolygonMode, RenderPipeline, TextureFormat,
};

const INITIAL_VERTEX_BUFFER_CAPACITY: usize = 1024;
const INITIAL_VERTEX_BUFFER_SIZE: BufferAddress =
    (INITIAL_VERTEX_BUFFER_CAPACITY * std::mem::size_of::<Vertex>()) as u64;
const INITIAL_INDEX_BUFFER_CAPACITY: usize = 1024;
const INITIAL_INDEX_BUFFER_SIZE: BufferAddress =
    (INITIAL_INDEX_BUFFER_CAPACITY * std::mem::size_of::<Index>()) as u64;
const INITIAL_MESH_UNIFORM_BUFFER_CAPACITY: usize = 100;
const INITIAL_MESH_UNIFORM_BUFFER_SIZE: BufferAddress =
    (INITIAL_INDEX_BUFFER_CAPACITY * std::mem::size_of::<MeshUniform>()) as u64;

pub struct MeshRenderer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,

    global_uniform_buffer: Buffer,
    global_uniform_bind_group_layout: BindGroupLayout,
    global_uniform_bind_group: BindGroup,

    mesh_uniform_buffer: Buffer,
    mesh_uniform_bind_group_layout: BindGroupLayout,
    mesh_uniform_bind_group: BindGroup,

    texture_bind_group_layout: BindGroupLayout,

    render_pipeline: RenderPipeline,
}

impl MeshRenderer {
    pub fn new(device: &Device, surface_texture_format: TextureFormat) -> Self {
        let vertex_buffer = create_copyable_buffer(
            device,
            "mesh_renderer_vertex_buffer",
            INITIAL_VERTEX_BUFFER_SIZE,
            BufferUsages::VERTEX,
        );
        let index_buffer = create_copyable_buffer(
            device,
            "mesh_renderer_index_buffer",
            INITIAL_INDEX_BUFFER_SIZE,
            BufferUsages::INDEX,
        );

        let global_uniform_buffer = create_global_uniform_buffer(
            device,
            "mesh_renderer_global_uniform_buffer",
            GlobalUniform,
        );
        let global_uniform_bind_group_layout = create_global_uniform_bind_group_layout(
            device,
            "mesh_renderer_global_uniform_bind_group_layout",
        );
        let global_uniform_bind_group = create_global_uniform_bind_group(
            device,
            "mesh_renderer_global_uniform_bind_group",
            &global_uniform_bind_group_layout,
            &global_uniform_buffer,
        );

        let mesh_uniform_buffer = create_copyable_buffer(
            device,
            "mesh_renderer_mesh_uniform_buffer",
            INITIAL_MESH_UNIFORM_BUFFER_SIZE,
            BufferUsages::UNIFORM,
        );
        let mesh_uniform_bind_group_layout = MeshUniform::create_bind_group_layout(device);
        let mesh_uniform_bind_group = create_uniform_bind_group::<MeshUniform>(
            device,
            "mesh_renderer_mesh_uniform_bind_group",
            &mesh_uniform_bind_group_layout,
            &mesh_uniform_buffer,
        );

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);

        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &texture_bind_group_layout,
            &global_uniform_bind_group_layout,
            &mesh_uniform_bind_group_layout,
        );

        Self {
            vertex_buffer,
            index_buffer,

            global_uniform_buffer,
            global_uniform_bind_group_layout,
            global_uniform_bind_group,

            mesh_uniform_buffer,
            mesh_uniform_bind_group_layout,
            mesh_uniform_bind_group,

            texture_bind_group_layout,
            render_pipeline,
        }
    }

    /// Submits a mesh for rendering
    pub fn draw_mesh(&mut self, parameters: DrawMeshParameters) {}

    /// Renders
    pub fn render(&mut self) {}

    fn create_texture_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesh_renderer_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        global_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("mesh_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/quad.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mesh_renderer_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_uniform_bind_group_layout,
                    mesh_uniform_bind_group_layout,
                    texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesh_renderer_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[
                    wgpu::ColorTargetState {
                        format: surface_texture_format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: Default::default(),
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: Default::default(),
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: Default::default(),
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                    wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: PolygonMode::Fill,
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        })
    }
}

/// Parameters for draw_mesh()
pub struct DrawMeshParameters {
    /// The mesh to draw
    pub mesh: Mesh,
    /// The material to draw the mesh with
    pub material: Material,
    /// The transform of the mesh
    pub transform: Matrix4<f32>,
    /// The view matrix to use to render the mesh
    pub view: Matrix4<f32>,
    /// The projection matrix to use to render the mesh
    pub projection: Matrix4<f32>,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshUniform;
impl MeshUniform {
    pub fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("mesh_renderer_mesh_uniform_bind_group_layout"),
            entries: &[],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform;

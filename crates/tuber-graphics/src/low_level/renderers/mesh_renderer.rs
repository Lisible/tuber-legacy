use crate::geometry::Vertex;
use crate::low_level::buffers::index_buffer::IndexBuffer;
use crate::low_level::buffers::uniform_buffer::UniformBuffer;
use crate::low_level::buffers::vertex_buffer::VertexBuffer;
use crate::low_level::texture::create_default_sampler;
use crate::primitives::{Mesh, TextureId};
use crate::Material;
use nalgebra::Matrix4;
use std::collections::HashMap;
use wgpu::{
    BindGroupDescriptor, BindGroupLayout, CommandEncoder, Device, IndexFormat, PolygonMode, Queue,
    RenderPass, RenderPipeline, Texture, TextureFormat, TextureViewDescriptor,
};

const INITIAL_VERTEX_BUFFER_CAPACITY: usize = 1000;
const INITIAL_INDEX_BUFFER_CAPACITY: usize = 3000;
const INITIAL_MESH_BUFFER_CAPACITY: usize = 100;

pub struct MeshRenderer {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    mesh_uniform_buffer: UniformBuffer<MeshUniform>,

    texture_bind_group_layout: BindGroupLayout,
    texture_bind_groups: HashMap<Material, wgpu::BindGroup>,

    render_pipeline: RenderPipeline,
    draw_metadata: Vec<DrawMetadata>,
}

impl MeshRenderer {
    pub fn new(device: &Device, surface_texture_format: TextureFormat) -> Self {
        let vertex_buffer = VertexBuffer::with_capacity(
            device,
            "mesh_renderer_vertex_buffer",
            INITIAL_VERTEX_BUFFER_CAPACITY,
        );
        let index_buffer = IndexBuffer::with_capacity(
            device,
            "mesh_renderer_index_buffer",
            INITIAL_INDEX_BUFFER_CAPACITY,
        );

        let mesh_uniform_buffer =
            UniformBuffer::new(device, "mesh_uniform", INITIAL_MESH_BUFFER_CAPACITY);

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);

        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &texture_bind_group_layout,
            mesh_uniform_buffer.bind_group_layout(),
        );

        Self {
            vertex_buffer,
            index_buffer,

            mesh_uniform_buffer,

            texture_bind_group_layout,
            texture_bind_groups: HashMap::new(),
            render_pipeline,
            draw_metadata: vec![],
        }
    }

    /// Submits a mesh for rendering
    pub fn draw_mesh(
        &mut self,
        command_encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        textures: &HashMap<TextureId, Texture>,
        params: DrawMeshParameters,
    ) {
        self.vertex_buffer
            .append_vertices(command_encoder, device, queue, params.mesh.vertices());
        let current_offset = self.index_buffer.current_offset();
        self.index_buffer
            .append_indices(command_encoder, device, queue, params.mesh.indices());

        let mesh_uniform_offset = self.mesh_uniform_buffer.current_offset();
        self.mesh_uniform_buffer.append_uniforms(
            command_encoder,
            device,
            queue,
            &[MeshUniform {
                transform_matrix: params.transform.into(),
                projection_matrix: params.projection.into(),
                view_matrix: params.view.into(),
            }],
        );

        let texture_bind_group = self.create_texture_bind_group(device, textures, &params.material);
        self.texture_bind_groups
            .insert(params.material.clone(), texture_bind_group);

        self.draw_metadata.push(DrawMetadata {
            start_offset: current_offset as u32,
            length: params.mesh.indices().len() as u32,
            mesh_uniform_offset: mesh_uniform_offset as u32,
            material: params.material.clone(),
        });
    }

    /// Renders
    pub fn render<'rpass: 'pass, 'pass>(&'rpass self, render_pass: &mut RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);

        for draw_metadata in &self.draw_metadata {
            render_pass.set_bind_group(1, &self.texture_bind_groups[&draw_metadata.material], &[]);
            render_pass.set_bind_group(
                0,
                self.mesh_uniform_buffer.bind_group(),
                &[draw_metadata.mesh_uniform_offset],
            );
            render_pass.draw_indexed(
                draw_metadata.start_offset..(draw_metadata.start_offset + draw_metadata.length),
                0,
                0..1,
            )
        }
    }

    pub fn cleanup(&mut self) {
        self.draw_metadata.clear();
        self.vertex_buffer.clear();
        self.index_buffer.clear();
        self.mesh_uniform_buffer.clear();
    }

    fn create_texture_bind_group(
        &mut self,
        device: &wgpu::Device,
        textures: &HashMap<TextureId, wgpu::Texture>,
        material: &Material,
    ) -> wgpu::BindGroup {
        let albedo_map_texture = &textures[&material.albedo_map_id];
        let albedo_map_view = albedo_map_texture.create_view(&TextureViewDescriptor::default());
        let albedo_map_sampler = create_default_sampler(device);

        let normal_map_texture = &textures[&material.normal_map_id];
        let normal_map_view = normal_map_texture.create_view(&TextureViewDescriptor::default());
        let normal_map_sampler = create_default_sampler(device);

        let emission_map_texture = &textures[&material.emission_map_id];
        let emission_map_view = emission_map_texture.create_view(&TextureViewDescriptor::default());
        let emission_map_sampler = create_default_sampler(device);

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_map_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&emission_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&emission_map_sampler),
                },
            ],
        })
    }

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
        mesh_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("mesh_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/mesh.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("mesh_renderer_render_pipeline_layout"),
                bind_group_layouts: &[mesh_uniform_bind_group_layout, texture_bind_group_layout],
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
                cull_mode: None,
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

struct DrawMetadata {
    pub start_offset: u32,
    pub length: u32,
    pub mesh_uniform_offset: u32,
    pub material: Material,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MeshUniform {
    transform_matrix: [[f32; 4]; 4],
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
}

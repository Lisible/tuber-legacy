use crate::geometry::Vertex;
use crate::texture::create_texture_bind_group_layout;
use crate::wgpu_state::IntoPolygonMode;
use nalgebra::Matrix4;
use tuber_core::transform::{IntoMatrix4, Transform2D};
use tuber_graphics::low_level::polygon_mode::PolygonMode;
use tuber_graphics::low_level::primitives::{QuadDescription, TextureId};
use wgpu::{BufferDescriptor, CommandEncoderDescriptor};

const QUAD_UNIFORM_SIZE: u64 = std::mem::size_of::<QuadUniform>() as u64;
const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<GlobalUniform>() as u64;
const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
const MIN_BUFFER_QUAD_COUNT: u64 = 1000;
const VERTEX_PER_QUAD: u64 = 6;
const QUAD_SIZE: u64 = VERTEX_PER_QUAD * VERTEX_SIZE;
const MIN_BUFFER_SIZE: u64 = MIN_BUFFER_QUAD_COUNT * QUAD_SIZE;

pub(crate) struct QuadRenderer {
    polygon_mode: PolygonMode,
    vertex_buffer_size: u64,
    vertex_buffer: wgpu::Buffer,
    global_uniform_buffer: wgpu::Buffer,
    global_bind_group_layout: wgpu::BindGroupLayout,
    global_bind_group: wgpu::BindGroup,
    quad_uniform_buffer_size: u64,
    quad_uniform_buffer: wgpu::Buffer,
    quad_bind_group_layout: wgpu::BindGroupLayout,
    quad_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    quad_uniform_alignment: wgpu::BufferAddress,
    surface_texture_format: wgpu::TextureFormat,
    quad_metadata: Vec<QuadMetadata>,
}

impl QuadRenderer {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let quad_uniform_alignment =
            device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let vertex_buffer = Self::create_vertex_buffer(device);
        let global_uniform_buffer = Self::create_global_uniform_buffer(device);
        let global_bind_group_layout = Self::create_global_bind_group_layout(device);
        let global_bind_group = Self::create_global_bind_group(
            device,
            &global_bind_group_layout,
            &global_uniform_buffer,
        );
        let quad_uniform_buffer = Self::create_quad_uniform_buffer(device, quad_uniform_alignment);
        let quad_bind_group_layout = Self::create_quad_bind_group_layout(device);
        let quad_bind_group =
            Self::create_quad_bind_group(device, &quad_bind_group_layout, &quad_uniform_buffer);
        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &global_bind_group_layout,
            &quad_bind_group_layout,
            PolygonMode::Fill.into_polygon_mode(),
        );

        Self {
            polygon_mode: PolygonMode::Fill,
            vertex_buffer_size: MIN_BUFFER_SIZE,
            vertex_buffer,
            global_uniform_buffer,
            global_bind_group_layout: global_bind_group_layout,
            global_bind_group,
            quad_uniform_buffer_size: MIN_BUFFER_QUAD_COUNT * quad_uniform_alignment,
            quad_uniform_buffer,
            quad_bind_group_layout: quad_bind_group_layout,
            quad_bind_group,
            render_pipeline,
            quad_uniform_alignment,
            surface_texture_format,
            quad_metadata: vec![],
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        quads: &[QuadDescription],
    ) {
        while quads.len() as u64 * QUAD_SIZE > self.vertex_buffer_size {
            self.reallocate_buffers(device, queue);
        }

        for quad in quads {
            self.prepare_quad(queue, quad);
        }
    }

    pub fn prepare_quad(&mut self, queue: &wgpu::Queue, quad: &QuadDescription) {
        let albedo_map_description = &quad.material.albedo_map_description;
        let normal_map_description = &quad.material.normal_map_description;
        let texture_region = &albedo_map_description.texture_region;

        self.add_uniform_to_buffer(
            queue,
            QuadUniform {
                model: quad.transform.clone().into_matrix4().into(),
            },
        );

        let color = [quad.color.r(), quad.color.g(), quad.color.b()];
        self.add_vertices_to_buffer(
            queue,
            &[
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    color,
                    tex_coords: [texture_region.x, texture_region.y],
                },
                Vertex {
                    position: [0.0, quad.size.height(), 0.0],
                    color,
                    tex_coords: [texture_region.x, texture_region.y + texture_region.height],
                },
                Vertex {
                    position: [quad.size.width(), 0.0, 0.0],
                    color,
                    tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                },
                Vertex {
                    position: [quad.size.width(), 0.0, 0.0],
                    color,
                    tex_coords: [texture_region.x + texture_region.width, texture_region.y],
                },
                Vertex {
                    position: [0.0, quad.size.height(), 0.0],
                    color,
                    tex_coords: [texture_region.x, texture_region.y + texture_region.height],
                },
                Vertex {
                    position: [quad.size.width(), quad.size.height(), 0.0],
                    color,
                    tex_coords: [
                        texture_region.x + texture_region.width,
                        texture_region.y + texture_region.height,
                    ],
                },
            ],
        );

        self.quad_metadata.push(QuadMetadata {
            albedo_map_texture_id: albedo_map_description.identifier,
            normal_map_texture_id: normal_map_description.identifier,
            uniform_offset: self.quad_metadata.len() as u32 * self.quad_uniform_alignment as u32,
        });
    }

    pub fn reallocate_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let new_vertex_buffer_size = self.vertex_buffer_size * 2;
        let new_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
            size: new_vertex_buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let new_quad_uniform_buffer_size = self.quad_uniform_buffer_size * 2;
        let new_quad_uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad_renderer_quad_uniform_buffer"),
            size: new_quad_uniform_buffer_size,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("quad_renderer_reallocate_vertex_buffer_encoder"),
        });
        encoder.copy_buffer_to_buffer(
            &self.vertex_buffer,
            0,
            &new_vertex_buffer,
            0,
            self.vertex_buffer_size,
        );
        encoder.copy_buffer_to_buffer(
            &self.quad_uniform_buffer,
            0,
            &new_quad_uniform_buffer,
            0,
            self.quad_uniform_buffer_size,
        );
        queue.submit(std::iter::once(encoder.finish()));

        self.vertex_buffer_size = new_vertex_buffer_size;
        self.vertex_buffer = new_vertex_buffer;

        self.quad_uniform_buffer_size = new_quad_uniform_buffer_size;
        self.quad_uniform_buffer = new_quad_uniform_buffer;
    }

    pub fn render<'rpass: 'pass, 'pass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        texture_bind_groups: &'rpass Vec<wgpu::BindGroup>,
    ) {
        for (i, quad_metadata) in self.quad_metadata.iter().enumerate() {
            let i = i as u32;
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.quad_bind_group,
                &[quad_metadata.uniform_offset.into()],
            );
            render_pass.set_bind_group(
                2,
                &texture_bind_groups[quad_metadata.albedo_map_texture_id.0],
                &[],
            );
            render_pass.set_bind_group(
                3,
                &texture_bind_groups[quad_metadata.normal_map_texture_id.0],
                &[],
            );
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(
                i * VERTEX_PER_QUAD as u32..(i + 1) * VERTEX_PER_QUAD as u32,
                0..1,
            );
        }
    }

    pub fn clear_pending_quads(&mut self) {
        self.quad_metadata.clear();
    }

    pub fn set_projection_matrix(
        &mut self,
        queue: &wgpu::Queue,
        projection_matrix: &Matrix4<f32>,
        transform: &Transform2D,
    ) {
        let view_matrix: Matrix4<f32> = (*transform).into_matrix4();
        let uniform = GlobalUniform {
            view_projection: (projection_matrix * view_matrix.try_inverse().unwrap()).into(),
        };
        queue.write_buffer(
            &self.global_uniform_buffer,
            0u64,
            bytemuck::cast_slice(&[uniform]),
        );
    }

    pub fn set_polygon_mode(&mut self, device: &wgpu::Device, polygon_mode: PolygonMode) {
        self.polygon_mode = polygon_mode;
        self.render_pipeline = Self::create_render_pipeline(
            device,
            self.surface_texture_format,
            &self.global_bind_group_layout,
            &self.quad_bind_group_layout,
            polygon_mode.into_polygon_mode(),
        );
    }

    fn add_uniform_to_buffer(&mut self, queue: &wgpu::Queue, quad_uniform: QuadUniform) {
        queue.write_buffer(
            &self.quad_uniform_buffer,
            (self.quad_metadata.len() * self.quad_uniform_alignment as usize)
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[quad_uniform]),
        );
    }

    fn add_vertices_to_buffer(&mut self, queue: &wgpu::Queue, vertices: &[Vertex]) {
        queue.write_buffer(
            &self.vertex_buffer,
            self.quad_metadata.len() as u64 * VERTEX_PER_QUAD * VERTEX_SIZE,
            bytemuck::cast_slice(vertices),
        );
    }

    fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
            size: MIN_BUFFER_SIZE,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_global_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_global_uniform_buffer"),
            size: GLOBAL_UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_global_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_renderer_global_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(GLOBAL_UNIFORM_SIZE),
                },
                count: None,
            }],
        })
    }

    fn create_global_bind_group(
        device: &wgpu::Device,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        global_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_renderer_global_bind_group"),
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn create_quad_uniform_buffer(
        device: &wgpu::Device,
        quad_uniform_alignment: wgpu::BufferAddress,
    ) -> wgpu::Buffer {
        assert!(QUAD_UNIFORM_SIZE <= quad_uniform_alignment);
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_quad_uniform_buffer"),
            size: (MIN_BUFFER_QUAD_COUNT * quad_uniform_alignment) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_quad_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_renderer_quad_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(QUAD_UNIFORM_SIZE),
                },
                count: None,
            }],
        })
    }

    fn create_quad_bind_group(
        device: &wgpu::Device,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        quad_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_renderer_quad_bind_group"),
            layout: &quad_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &quad_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(QUAD_UNIFORM_SIZE),
                }),
            }],
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/quad.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_bind_group_layout,
                    quad_bind_group_layout,
                    &create_texture_bind_group_layout(device),
                    &create_texture_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_renderer_render_pipeline"),
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
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode,
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

struct QuadMetadata {
    albedo_map_texture_id: TextureId,
    normal_map_texture_id: TextureId,
    uniform_offset: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadUniform {
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    view_projection: [[f32; 4]; 4],
}

use crate::geometry::Vertex;
use crate::texture::{create_texture_bind_group_layout, TextureBindGroup};
use crate::DrawRange::VertexIndexRange;
use crate::{DrawCommand, DrawCommandData, QuadDrawCommand};
use nalgebra::Matrix4;
use std::collections::HashMap;
use tuber_core::transform::{IntoMatrix4, Transform2D};
use tuber_graphics::camera::OrthographicCamera;
use tuber_graphics::low_level::QuadDescription;
use tuber_graphics::texture::{
    TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER, DEFAULT_TEXTURE_IDENTIFIER,
    WHITE_TEXTURE_IDENTIFIER,
};
use wgpu::{
    BindGroupLayoutDescriptor, BufferDescriptor, PipelineLayoutDescriptor, RenderPipelineDescriptor,
};

const MAX_QUAD_COUNT: usize = 1000;
const QUAD_UNIFORM_SIZE: u64 = std::mem::size_of::<QuadUniform>() as u64;
const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<GlobalUniform>() as u64;
const VERTEX_PER_QUAD: usize = 6;

pub(crate) struct QuadRenderer {
    vertex_buffer: wgpu::Buffer,
    global_uniform_buffer: wgpu::Buffer,
    _global_bind_group_layout: wgpu::BindGroupLayout,
    global_bind_group: wgpu::BindGroup,
    quad_uniform_buffer: wgpu::Buffer,
    _quad_bind_group_layout: wgpu::BindGroupLayout,
    quad_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    quad_uniform_alignment: wgpu::BufferAddress,
    pending_quad_count: usize,
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
        );

        Self {
            vertex_buffer,
            global_uniform_buffer,
            _global_bind_group_layout: global_bind_group_layout,
            global_bind_group,
            quad_uniform_buffer,
            _quad_bind_group_layout: quad_bind_group_layout,
            quad_bind_group,
            render_pipeline,
            quad_uniform_alignment,
            pending_quad_count: 0usize,
        }
    }

    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        quad: &QuadDescription,
        transform: &Transform2D,
        texture_bind_groups: &HashMap<String, TextureBindGroup>,
    ) -> DrawCommand {
        assert!(self.pending_quad_count < MAX_QUAD_COUNT);

        let (albedo_map_identifier, normalized_texture_region) =
            if let Some(texture) = &quad.material.albedo_map_description {
                if texture_bind_groups.contains_key(&texture.identifier) {
                    (texture.identifier.as_str(), texture.texture_region)
                } else {
                    (
                        DEFAULT_TEXTURE_IDENTIFIER,
                        TextureRegion::new(0.0, 0.0, 1.0, 1.0),
                    )
                }
            } else {
                (
                    WHITE_TEXTURE_IDENTIFIER,
                    TextureRegion::new(0.0, 0.0, 1.0, 1.0),
                )
            };

        let normal_map_identifier = DEFAULT_NORMAL_MAP_IDENTIFIER;

        self.add_uniform_to_buffer(
            queue,
            QuadUniform {
                model: transform.clone().into_matrix4().into(),
            },
        );

        let color = [quad.color.0, quad.color.1, quad.color.2];
        self.add_vertices_to_buffer(
            queue,
            &[
                Vertex {
                    position: [0.0, 0.0, 0.0],
                    color,
                    tex_coords: [normalized_texture_region.x, normalized_texture_region.y],
                },
                Vertex {
                    position: [0.0, quad.height, 0.0],
                    color,
                    tex_coords: [
                        normalized_texture_region.x,
                        normalized_texture_region.y + normalized_texture_region.height,
                    ],
                },
                Vertex {
                    position: [quad.width, 0.0, 0.0],
                    color,
                    tex_coords: [
                        normalized_texture_region.x + normalized_texture_region.width,
                        normalized_texture_region.y,
                    ],
                },
                Vertex {
                    position: [quad.width, 0.0, 0.0],
                    color,
                    tex_coords: [
                        normalized_texture_region.x + normalized_texture_region.width,
                        normalized_texture_region.y,
                    ],
                },
                Vertex {
                    position: [0.0, quad.height, 0.0],
                    color,
                    tex_coords: [
                        normalized_texture_region.x,
                        normalized_texture_region.y + normalized_texture_region.height,
                    ],
                },
                Vertex {
                    position: [quad.width, quad.height, 0.0],
                    color,
                    tex_coords: [
                        normalized_texture_region.x + normalized_texture_region.width,
                        normalized_texture_region.y + normalized_texture_region.height,
                    ],
                },
            ],
        );

        self.pending_quad_count += 1;

        DrawCommand {
            draw_command_data: DrawCommandData::QuadDrawCommand(QuadDrawCommand {
                draw_range: VertexIndexRange(
                    ((self.pending_quad_count - 1) * VERTEX_PER_QUAD) as u32
                        ..(self.pending_quad_count * VERTEX_PER_QUAD) as u32,
                ),
                uniform_offset: ((self.pending_quad_count - 1)
                    * self.quad_uniform_alignment as usize) as _,
                albedo_map_identifier: albedo_map_identifier.to_string(),
                normal_map_identifier: normal_map_identifier.to_string(),
            }),
            z_order: transform.translation.2,
        }
    }

    pub fn render<'rpass: 'pass, 'pass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        draw_command: &QuadDrawCommand,
        texture_bind_groups: &'rpass HashMap<String, TextureBindGroup>,
    ) {
        let vertex_index_range = draw_command
            .draw_range
            .vertex_index_range()
            .expect("Vertex index range expected");

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.global_bind_group, &[]);
        render_pass.set_bind_group(1, &self.quad_bind_group, &[draw_command.uniform_offset]);
        render_pass.set_bind_group(
            2,
            &texture_bind_groups[&draw_command.albedo_map_identifier].bind_group,
            &[],
        );
        render_pass.set_bind_group(
            3,
            &texture_bind_groups[&draw_command.normal_map_identifier].bind_group,
            &[],
        );

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(vertex_index_range.clone(), 0..1);
    }

    pub fn clear_pending_quads(&mut self) {
        self.pending_quad_count = 0;
    }

    pub fn set_camera(
        &mut self,
        queue: &wgpu::Queue,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        let projection_matrix: Matrix4<f32> = Matrix4::new_orthographic(
            camera.left,
            camera.right,
            camera.bottom,
            camera.top,
            camera.near,
            camera.far,
        );
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

    fn add_uniform_to_buffer(&mut self, queue: &wgpu::Queue, quad_uniform: QuadUniform) {
        queue.write_buffer(
            &self.quad_uniform_buffer,
            (self.pending_quad_count * self.quad_uniform_alignment as usize) as wgpu::BufferAddress,
            bytemuck::cast_slice(&[quad_uniform]),
        );
    }

    fn add_vertices_to_buffer(&mut self, queue: &wgpu::Queue, vertices: &[Vertex]) {
        queue.write_buffer(
            &self.vertex_buffer,
            (self.pending_quad_count * VERTEX_PER_QUAD * std::mem::size_of::<Vertex>())
                as wgpu::BufferAddress,
            bytemuck::cast_slice(vertices),
        );
    }

    fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
            size: (std::mem::size_of::<Vertex>() * VERTEX_PER_QUAD * MAX_QUAD_COUNT)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_global_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_global_uniform_buffer"),
            size: GLOBAL_UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_global_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
            size: (MAX_QUAD_COUNT * quad_uniform_alignment as usize) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_quad_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/quad.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("quad_renderer_render_pipeline_layout"),
            bind_group_layouts: &[
                global_bind_group_layout,
                quad_bind_group_layout,
                &create_texture_bind_group_layout(device),
                &create_texture_bind_group_layout(device),
            ],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
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
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                    wgpu::ColorTargetState {
                        format: surface_texture_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    },
                ],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
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

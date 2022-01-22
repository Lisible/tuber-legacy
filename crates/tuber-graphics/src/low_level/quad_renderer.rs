use crate::draw_command::DrawQuadCommand;
use crate::geometry::Vertex;
use crate::low_level::polygon_mode::PolygonMode;
use crate::low_level::primitives::TextureId;
use crate::low_level::texture::create_texture_bind_group_layout;
use crate::low_level::wgpu_state::IntoPolygonMode;
use nalgebra::Matrix4;
use tuber_core::transform::{IntoMatrix4, Transform2D};
use wgpu::{BufferDescriptor, CommandEncoderDescriptor};

const QUAD_UNIFORM_SIZE: u64 = std::mem::size_of::<QuadUniform>() as u64;
const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<GlobalUniform>() as u64;
const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
const VERTEX_PER_QUAD: u64 = 6;
const QUAD_SIZE: u64 = VERTEX_PER_QUAD * VERTEX_SIZE;
const MIN_QUAD_COUNT: usize = 1000;
const MIN_GLOBAL_UNIFORM_COUNT: usize = 10;

pub(crate) struct QuadRenderer {
    polygon_mode: PolygonMode,
    vertex_buffer_size: u64,
    vertex_buffer: wgpu::Buffer,
    global_uniform_buffer_size: u64,
    global_uniform_buffer: wgpu::Buffer,
    global_bind_group_layout: wgpu::BindGroupLayout,
    global_bind_group: wgpu::BindGroup,
    quad_uniform_buffer_size: u64,
    quad_uniform_buffer: wgpu::Buffer,
    quad_bind_group_layout: wgpu::BindGroupLayout,
    quad_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    ui_render_pipeline: wgpu::RenderPipeline,
    min_uniform_alignment: wgpu::BufferAddress,
    surface_texture_format: wgpu::TextureFormat,
    quad_metadata: Vec<QuadMetadata>,
    quad_groups: Vec<QuadGroup>,
    quad_count: usize,
    max_quad_count: usize,
    global_uniform_count: usize,
    max_global_uniform_count: usize,
}

impl QuadRenderer {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer_size = MIN_QUAD_COUNT as u64 * VERTEX_PER_QUAD * VERTEX_SIZE;
        let vertex_buffer = Self::create_vertex_buffer(device, vertex_buffer_size);

        let min_uniform_alignment =
            device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;
        let global_uniform_buffer_size = MIN_GLOBAL_UNIFORM_COUNT as u64 * min_uniform_alignment;
        assert!(std::mem::size_of::<GlobalUniform>() <= min_uniform_alignment as usize);
        let global_uniform_buffer =
            Self::create_global_uniform_buffer(device, global_uniform_buffer_size);
        let global_bind_group_layout = Self::create_global_bind_group_layout(device);
        let global_bind_group = Self::create_global_bind_group(
            device,
            &global_bind_group_layout,
            &global_uniform_buffer,
        );

        let quad_uniform_buffer_size = MIN_QUAD_COUNT as u64 * min_uniform_alignment;
        assert!(std::mem::size_of::<QuadUniform>() <= min_uniform_alignment as usize);
        let quad_uniform_buffer =
            Self::create_quad_uniform_buffer(device, quad_uniform_buffer_size);
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

        let ui_render_pipeline = Self::create_ui_render_pipeline(
            device,
            surface_texture_format,
            &global_bind_group_layout,
            &quad_bind_group_layout,
            PolygonMode::Fill.into_polygon_mode(),
        );

        Self {
            polygon_mode: PolygonMode::Fill,
            vertex_buffer_size,
            vertex_buffer,
            global_uniform_buffer_size,
            global_uniform_buffer,
            global_bind_group_layout,
            global_bind_group,
            quad_uniform_buffer_size,
            quad_uniform_buffer,
            quad_bind_group_layout,
            quad_bind_group,
            render_pipeline,
            ui_render_pipeline,
            min_uniform_alignment,
            surface_texture_format,
            quad_metadata: vec![],
            quad_groups: vec![],
            quad_count: 0,
            max_quad_count: MIN_QUAD_COUNT,
            global_uniform_count: 0,
            max_global_uniform_count: MIN_GLOBAL_UNIFORM_COUNT,
        }
    }

    pub fn prepare_quad_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        projection_matrix: &Matrix4<f32>,
        view_transform: &Transform2D,
        draw_quad_commands: &[DrawQuadCommand],
    ) -> QuadGroup {
        self.ensure_max_quad_count(
            device,
            queue,
            (self.quad_count + draw_quad_commands.len()) as u64,
        );

        self.ensure_max_global_uniform_count(device, queue, (self.global_uniform_count + 1) as u64);
        queue.write_buffer(
            &self.global_uniform_buffer,
            (self.global_uniform_count
                * device.limits().min_uniform_buffer_offset_alignment as usize)
                as wgpu::BufferAddress,
            bytemuck::cast_slice(&[GlobalUniform {
                view_projection: (projection_matrix
                    * view_transform.into_matrix4().try_inverse().unwrap())
                .into(),
            }]),
        );

        let quad_group = QuadGroup {
            start_quad: self.quad_count as u64,
            end_quad: (self.quad_count + draw_quad_commands.len()) as u64,
            global_uniform: self.global_uniform_count as u64,
        };

        for (index, draw_quad_command) in draw_quad_commands.iter().enumerate() {
            let quad_index = quad_group.start_quad + index as u64;
            queue.write_buffer(
                &self.quad_uniform_buffer,
                (quad_index * self.min_uniform_alignment) as wgpu::BufferAddress,
                bytemuck::cast_slice(&[QuadUniform {
                    model: draw_quad_command
                        .world_transform
                        .clone()
                        .into_matrix4()
                        .into(),
                }]),
            );

            queue.write_buffer(
                &self.vertex_buffer,
                (quad_index * QUAD_SIZE) as wgpu::BufferAddress,
                bytemuck::cast_slice(&[
                    draw_quad_command.quad.top_left,
                    draw_quad_command.quad.bottom_left,
                    draw_quad_command.quad.top_right,
                    draw_quad_command.quad.top_right,
                    draw_quad_command.quad.bottom_left,
                    draw_quad_command.quad.bottom_right,
                ]),
            );

            self.quad_metadata.push(QuadMetadata {
                albedo_map_texture_id: draw_quad_command.material.albedo_map_id,
                normal_map_texture_id: draw_quad_command.material.normal_map_id,
                uniform_offset: self.quad_metadata.len() as u32 * self.min_uniform_alignment as u32,
            });
        }

        self.quad_count += draw_quad_commands.len();
        self.global_uniform_count += 1;
        quad_group
    }

    pub fn ensure_max_quad_count(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        max_quad_count: u64,
    ) {
        if self.max_quad_count > max_quad_count as usize {
            return;
        }

        self.reallocate_buffers(device, queue, max_quad_count);
    }

    pub fn ensure_max_global_uniform_count(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        max_global_uniform_count: u64,
    ) {
        if self.max_global_uniform_count > max_global_uniform_count as usize {
            return;
        }

        let new_max_global_uniform_count = self.max_global_uniform_count * 2;
        let new_global_uniform_buffer_size = new_max_global_uniform_count
            * device.limits().min_uniform_buffer_offset_alignment as usize;
        let new_global_uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad_renderer_global_uniform_buffer"),
            size: new_global_uniform_buffer_size as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("quad_renderer_reallocate_vertex_buffer_encoder"),
        });
        encoder.copy_buffer_to_buffer(
            &self.global_uniform_buffer,
            0,
            &new_global_uniform_buffer,
            0,
            self.global_uniform_buffer_size,
        );
        queue.submit(std::iter::once(encoder.finish()));
        self.max_global_uniform_count = new_max_global_uniform_count;
        self.global_uniform_buffer = new_global_uniform_buffer;
    }

    pub fn reallocate_buffers(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        ensured_quad_count: u64,
    ) {
        let new_max_quad_count = ensured_quad_count.max(MIN_QUAD_COUNT as u64);
        let new_vertex_buffer_size = new_max_quad_count * QUAD_SIZE;
        let new_vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
            size: new_vertex_buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let new_quad_uniform_buffer_size =
            new_max_quad_count as u64 * device.limits().min_uniform_buffer_offset_alignment as u64;
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

        self.vertex_buffer_size = new_vertex_buffer_size.into();
        self.vertex_buffer = new_vertex_buffer;

        self.quad_uniform_buffer_size = new_quad_uniform_buffer_size;
        self.quad_uniform_buffer = new_quad_uniform_buffer;
        self.max_quad_count = new_max_quad_count as usize;
    }

    pub fn render_quad_group<'rpass: 'pass, 'pass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        texture_bind_groups: &'rpass Vec<wgpu::BindGroup>,
        quad_render_pass_type: QuadRenderPassType,
        quad_group: &QuadGroup,
    ) {
        let render_pipeline = match quad_render_pass_type {
            QuadRenderPassType::Geometry => &self.render_pipeline,
            QuadRenderPassType::UI => &self.ui_render_pipeline,
        };

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(
            0,
            &self.global_bind_group,
            &[((quad_group.global_uniform * self.min_uniform_alignment) as u32).into()],
        );

        for (i, quad_metadata) in self.quad_metadata
            [quad_group.start_quad as usize..quad_group.end_quad as usize]
            .iter()
            .enumerate()
        {
            let i = i as u32;
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
                (quad_group.start_quad as u32 + i as u32) * VERTEX_PER_QUAD as u32
                    ..(quad_group.start_quad as u32 + i as u32 + 1) * VERTEX_PER_QUAD as u32,
                0..1,
            );
        }
    }

    pub fn clear_pending_quads(&mut self) {
        self.quad_metadata.clear();
        self.quad_groups.clear();
        self.global_uniform_count = 0;
        self.quad_count = 0;
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
        self.ui_render_pipeline = Self::create_ui_render_pipeline(
            device,
            self.surface_texture_format,
            &self.global_bind_group_layout,
            &self.quad_bind_group_layout,
            polygon_mode.into_polygon_mode(),
        );
    }

    fn create_vertex_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_vertex_buffer"),
            size,
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        })
    }

    fn create_global_uniform_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_global_uniform_buffer"),
            size,
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
                    has_dynamic_offset: true,
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
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &global_uniform_buffer,
                    offset: 0,
                    size: wgpu::BufferSize::new(GLOBAL_UNIFORM_SIZE),
                }),
            }],
        })
    }

    fn create_quad_uniform_buffer(device: &wgpu::Device, size: u64) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_renderer_quad_uniform_buffer"),
            size,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
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
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/quad.wgsl").into()),
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

    fn create_ui_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_ui_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ui.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_ui_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_bind_group_layout,
                    quad_bind_group_layout,
                    &create_texture_bind_group_layout(device),
                    &create_texture_bind_group_layout(device),
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_renderer_ui_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
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
                }],
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

pub(crate) enum QuadRenderPassType {
    Geometry,
    UI,
}

#[derive(Debug)]
pub struct QuadGroup {
    pub start_quad: u64,
    pub end_quad: u64,
    pub global_uniform: u64,
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

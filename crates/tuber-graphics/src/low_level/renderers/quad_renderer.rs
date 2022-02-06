use crate::draw_command::DrawQuadCommand;
use crate::geometry::Vertex;
use crate::low_level::polygon_mode::PolygonMode;
use crate::low_level::primitives::TextureId;
use crate::low_level::texture::create_default_sampler;
use crate::low_level::uniform_buffer::UniformBuffer;
use crate::low_level::wgpu_state::IntoPolygonMode;
use crate::Material;
use nalgebra::Matrix4;
use std::collections::HashMap;
use wgpu::{
    BindGroupDescriptor, BufferDescriptor, CommandEncoder, CommandEncoderDescriptor, Device,
    TextureViewDescriptor,
};

const QUAD_UNIFORM_SIZE: u64 = std::mem::size_of::<QuadUniform>() as u64;
const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<QuadGroupUniform>() as u64;
const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
const VERTEX_PER_QUAD: u64 = 6;
const QUAD_SIZE: u64 = VERTEX_PER_QUAD * VERTEX_SIZE;
const MIN_QUAD_COUNT: usize = 1000;
const MIN_GLOBAL_UNIFORM_COUNT: usize = 10;

pub(crate) struct QuadRenderer {
    vertex_buffer_size: u64,
    vertex_buffer: wgpu::Buffer,

    quad_group_uniform_buffer: UniformBuffer<QuadGroupUniform>,
    quad_uniform_buffer: UniformBuffer<QuadUniform>,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    ui_texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: HashMap<Material, wgpu::BindGroup>,
    ui_texture_bind_groups: HashMap<Material, wgpu::BindGroup>,

    pre_render_pipeline: wgpu::RenderPipeline,
    render_pipeline: wgpu::RenderPipeline,
    ui_render_pipeline: wgpu::RenderPipeline,

    polygon_mode: PolygonMode,
    min_uniform_alignment: wgpu::BufferAddress,
    surface_texture_format: wgpu::TextureFormat,

    quad_metadata: Vec<QuadMetadata>,
    quad_count: usize,
    max_quad_count: usize,
    global_uniform_count: usize,
    max_global_uniform_count: usize,
    pending_vertices: Vec<Vertex>,
    pending_quad_group_uniforms: Vec<QuadGroupUniform>,
    pending_quad_uniforms: Vec<QuadUniform>,
}

impl QuadRenderer {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer_size = MIN_QUAD_COUNT as u64 * VERTEX_PER_QUAD * VERTEX_SIZE;
        let vertex_buffer = Self::create_vertex_buffer(device, vertex_buffer_size);

        let min_uniform_alignment =
            device.limits().min_uniform_buffer_offset_alignment as wgpu::BufferAddress;

        let quad_group_uniform_buffer =
            UniformBuffer::new(device, "quad_renderer_quad_group_uniform", 4);
        let quad_uniform_buffer = UniformBuffer::new(device, "quad_renderer_quad_uniform", 10);

        let texture_bind_group_layout = Self::create_texture_bind_group_layout(device);
        let ui_texture_bind_group_layout = Self::create_ui_texture_bind_group_layout(device);

        let pre_render_pipeline = Self::create_pre_render_pipeline(
            device,
            surface_texture_format,
            &texture_bind_group_layout,
            quad_group_uniform_buffer.bind_group_layout(),
            quad_uniform_buffer.bind_group_layout(),
            PolygonMode::Fill.into_polygon_mode(),
        );

        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &texture_bind_group_layout,
            quad_group_uniform_buffer.bind_group_layout(),
            quad_uniform_buffer.bind_group_layout(),
            PolygonMode::Fill.into_polygon_mode(),
        );

        let ui_render_pipeline = Self::create_ui_render_pipeline(
            device,
            surface_texture_format,
            &ui_texture_bind_group_layout,
            quad_group_uniform_buffer.bind_group_layout(),
            quad_uniform_buffer.bind_group_layout(),
            PolygonMode::Fill.into_polygon_mode(),
        );

        Self {
            vertex_buffer_size,
            vertex_buffer,

            quad_group_uniform_buffer,
            quad_uniform_buffer,

            texture_bind_group_layout,
            ui_texture_bind_group_layout,
            texture_bind_groups: HashMap::new(),
            ui_texture_bind_groups: HashMap::new(),

            pre_render_pipeline,
            render_pipeline,
            ui_render_pipeline,

            polygon_mode: PolygonMode::Fill,
            min_uniform_alignment,
            surface_texture_format,

            quad_metadata: vec![],
            quad_count: 0,
            max_quad_count: MIN_QUAD_COUNT,
            global_uniform_count: 0,
            max_global_uniform_count: MIN_GLOBAL_UNIFORM_COUNT,
            pending_vertices: vec![],
            pending_quad_group_uniforms: vec![],
            pending_quad_uniforms: vec![],
        }
    }

    pub fn prepare_quad_group(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_encoder: &mut wgpu::CommandEncoder,
        textures: &HashMap<TextureId, wgpu::Texture>,
        projection_matrix: &Matrix4<f32>,
        view_transform: &Matrix4<f32>,
        draw_quad_commands: &[DrawQuadCommand],
        ui: bool,
    ) -> QuadGroup {
        self.ensure_max_quad_count(
            device,
            queue,
            (self.quad_count + draw_quad_commands.len()) as u64,
        );

        self.quad_group_uniform_buffer.ensure_capacity(
            device,
            command_encoder,
            self.global_uniform_count + 1,
        );

        self.quad_uniform_buffer.ensure_capacity(
            device,
            command_encoder,
            self.quad_count + draw_quad_commands.len(),
        );

        self.pending_quad_group_uniforms.push(QuadGroupUniform {
            view_projection: (projection_matrix * view_transform.try_inverse().unwrap()).into(),
            _padding: [0.0; 48],
        });

        let quad_group = QuadGroup {
            start_quad: self.quad_count as u64,
            end_quad: (self.quad_count + draw_quad_commands.len()) as u64,
            global_uniform: self.global_uniform_count as u64,
        };

        for draw_quad_command in draw_quad_commands {
            let mut effective_transform = draw_quad_command.world_transform.clone();
            effective_transform.column_mut(3).z = 0.0;

            let material = draw_quad_command.material.clone();

            if ui {
                let texture_bind_group =
                    self.create_ui_texture_bind_group(device, textures, &material);
                self.ui_texture_bind_groups
                    .insert(material, texture_bind_group);
            } else {
                let texture_bind_group =
                    self.create_texture_bind_group(device, textures, &material);
                self.texture_bind_groups
                    .insert(material, texture_bind_group);
            }

            self.pending_quad_uniforms.push(QuadUniform {
                model: effective_transform.into(),
                _padding: [0.0; 48],
            });

            self.pending_vertices.extend_from_slice(&[
                draw_quad_command.quad.top_left,
                draw_quad_command.quad.bottom_left,
                draw_quad_command.quad.top_right,
                draw_quad_command.quad.top_right,
                draw_quad_command.quad.bottom_left,
                draw_quad_command.quad.bottom_right,
            ]);

            self.quad_metadata.push(QuadMetadata {
                material_description: draw_quad_command.material.clone(),
                uniform_offset: (self.quad_metadata.len() * self.min_uniform_alignment as usize)
                    as u32,
            });
        }

        self.quad_count += draw_quad_commands.len();
        self.global_uniform_count += 1;
        quad_group
    }

    pub fn create_texture_bind_group(
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

    pub fn finish_preparation(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        queue: &wgpu::Queue,
    ) {
        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.pending_vertices),
        );

        self.quad_group_uniform_buffer.append_uniforms(
            command_encoder,
            device,
            queue,
            &self.pending_quad_group_uniforms,
        );

        self.quad_uniform_buffer.append_uniforms(
            command_encoder,
            device,
            queue,
            &self.pending_quad_uniforms,
        );

        self.pending_vertices.clear();
        self.pending_quad_group_uniforms.clear();
        self.pending_quad_uniforms.clear();
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
        queue.submit(std::iter::once(encoder.finish()));

        self.vertex_buffer_size = new_vertex_buffer_size.into();
        self.vertex_buffer = new_vertex_buffer;
        self.max_quad_count = new_max_quad_count as usize;
    }

    pub fn render_quad_group<'rpass: 'pass, 'pass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'pass>,
        quad_render_pass_type: QuadRenderPassType,
        quad_group: &QuadGroup,
    ) {
        let render_pipeline = match quad_render_pass_type {
            QuadRenderPassType::PreRender => &self.pre_render_pipeline,
            QuadRenderPassType::Geometry => &self.render_pipeline,
            QuadRenderPassType::UI => &self.ui_render_pipeline,
        };

        render_pass.set_pipeline(render_pipeline);
        render_pass.set_bind_group(
            0,
            self.quad_group_uniform_buffer.bind_group(),
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
                self.quad_uniform_buffer.bind_group(),
                &[quad_metadata.uniform_offset.into()],
            );

            if quad_render_pass_type == QuadRenderPassType::UI {
                render_pass.set_bind_group(
                    2,
                    &self.ui_texture_bind_groups[&quad_metadata.material_description],
                    &[],
                );
            } else {
                render_pass.set_bind_group(
                    2,
                    &self.texture_bind_groups[&quad_metadata.material_description],
                    &[],
                );
            }
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
        self.global_uniform_count = 0;
        self.quad_count = 0;
        self.quad_uniform_buffer.clear();
        self.quad_group_uniform_buffer.clear();
    }

    pub fn set_polygon_mode(&mut self, device: &wgpu::Device, polygon_mode: PolygonMode) {
        self.polygon_mode = polygon_mode;
        self.render_pipeline = Self::create_render_pipeline(
            device,
            self.surface_texture_format,
            &self.texture_bind_group_layout,
            self.quad_group_uniform_buffer.bind_group_layout(),
            self.quad_uniform_buffer.bind_group_layout(),
            polygon_mode.into_polygon_mode(),
        );
        self.ui_render_pipeline = Self::create_ui_render_pipeline(
            device,
            self.surface_texture_format,
            &self.texture_bind_group_layout,
            self.quad_group_uniform_buffer.bind_group_layout(),
            self.quad_uniform_buffer.bind_group_layout(),
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
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/quad.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_bind_group_layout,
                    quad_bind_group_layout,
                    texture_bind_group_layout,
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

    fn create_pre_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/quad.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_bind_group_layout,
                    quad_bind_group_layout,
                    texture_bind_group_layout,
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
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        quad_bind_group_layout: &wgpu::BindGroupLayout,
        polygon_mode: wgpu::PolygonMode,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("quad_renderer_ui_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/ui.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("quad_renderer_ui_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_bind_group_layout,
                    quad_bind_group_layout,
                    texture_bind_group_layout,
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

    pub fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_renderer_texture_bind_group_layout"),
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

    pub fn create_ui_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_renderer_ui_texture_bind_group_layout"),
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
            ],
        })
    }

    pub fn create_ui_texture_bind_group(
        &self,
        device: &wgpu::Device,
        textures: &HashMap<TextureId, wgpu::Texture>,
        material: &Material,
    ) -> wgpu::BindGroup {
        let albedo_map_texture = &textures[&material.albedo_map_id];
        let albedo_map_view = albedo_map_texture.create_view(&TextureViewDescriptor::default());
        let albedo_map_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.ui_texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&albedo_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&albedo_map_sampler),
                },
            ],
        })
    }
}

#[derive(PartialEq)]
pub(crate) enum QuadRenderPassType {
    PreRender,
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
    material_description: Material,
    uniform_offset: u32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadUniform {
    model: [[f32; 4]; 4],
    _padding: [f32; 48],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct QuadGroupUniform {
    view_projection: [[f32; 4]; 4],
    _padding: [f32; 48],
}

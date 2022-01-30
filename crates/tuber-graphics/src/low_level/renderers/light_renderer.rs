use crate::draw_command::DrawLightCommand;
use crate::geometry::Vertex;
use crate::low_level::g_buffer::GBuffer;
use crate::Color;
use wgpu::util::DeviceExt;

const VERTEX_COUNT: usize = 6;
const MIN_POINT_LIGHT_CAPACITY: usize = 20;
const POINT_LIGHT_UNIFORM_BUFFER_LABEL: &'static str = "light_renderer_point_light_uniform_buffer";
const DEFAULT_AMBIENT_LIGHT: [f32; 3] = [1.0, 1.0, 1.0];

pub struct LightRenderer {
    vertex_buffer: wgpu::Buffer,

    global_uniform_buffer: wgpu::Buffer,
    _global_uniform_bind_group_layout: wgpu::BindGroupLayout,
    global_uniform_bind_group: wgpu::BindGroup,

    point_light_uniform_buffer: wgpu::Buffer,
    _point_light_uniform_bind_group_layout: wgpu::BindGroupLayout,
    point_light_uniform_bind_group: wgpu::BindGroup,
    point_light_capacity: usize,

    g_buffer_bind_group_layout: wgpu::BindGroupLayout,
    g_buffer_bind_group: Option<wgpu::BindGroup>,

    render_pipeline: wgpu::RenderPipeline,
}

impl LightRenderer {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(device);

        let global_uniform_buffer = Self::create_global_uniform_buffer(device);
        let global_uniform_bind_group_layout =
            Self::create_global_uniform_bind_group_layout(device);
        let global_uniform_bind_group = Self::create_global_uniform_bind_group(
            device,
            &global_uniform_bind_group_layout,
            &global_uniform_buffer,
        );

        let point_light_uniform_bind_group_layout =
            Self::create_point_light_uniform_bind_group_layout(device);

        let g_buffer_bind_group_layout = Self::create_g_buffer_bind_group_layout(device);

        let point_light_uniform_buffer =
            Self::create_point_light_uniform_buffer(device, MIN_POINT_LIGHT_CAPACITY as u32);
        let point_light_uniform_bind_group = Self::create_point_light_uniform_bind_group(
            device,
            &point_light_uniform_bind_group_layout,
            &point_light_uniform_buffer,
        );

        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &global_uniform_bind_group_layout,
            &g_buffer_bind_group_layout,
            &point_light_uniform_bind_group_layout,
        );

        Self {
            vertex_buffer,

            global_uniform_buffer,
            _global_uniform_bind_group_layout: global_uniform_bind_group_layout,
            global_uniform_bind_group,

            point_light_uniform_buffer,
            _point_light_uniform_bind_group_layout: point_light_uniform_bind_group_layout,
            point_light_uniform_bind_group,
            point_light_capacity: MIN_POINT_LIGHT_CAPACITY,

            g_buffer_bind_group_layout,
            g_buffer_bind_group: None,
            render_pipeline,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        command_encoder: &mut wgpu::CommandEncoder,
        ambient_light: Color,
        g_buffer: GBuffer,
        draw_light_commands: &[DrawLightCommand],
    ) {
        self.ensure_point_light_uniform_capacity(
            device,
            command_encoder,
            draw_light_commands.len(),
        );

        self.g_buffer_bind_group = Some(Self::create_g_buffer_bind_group(
            device,
            &self.g_buffer_bind_group_layout,
            g_buffer,
        ));

        let uniforms = draw_light_commands
            .iter()
            .map(|command| PointLightUniform {
                position: command.world_transform.column(3).xyz().into(),
                radius: command.light.radius,
                ambient_color: command.light.ambient.into(),
                _padding: 0,
                diffuse_color: command.light.diffuse.into(),
                _padding2: 0,
                specular_color: command.light.specular.into(),
                _padding3: 0,
            })
            .collect::<Vec<_>>();

        queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[GlobalUniform {
                ambient_light: ambient_light.into(),
                light_count: draw_light_commands.len() as i32,
            }]),
        );

        queue.write_buffer(
            &self.point_light_uniform_buffer,
            0,
            bytemuck::cast_slice(&uniforms),
        );
    }

    fn ensure_point_light_uniform_capacity(
        &mut self,
        device: &wgpu::Device,
        command_encoder: &mut wgpu::CommandEncoder,
        capacity: usize,
    ) {
        if self.point_light_capacity >= capacity {
            return;
        }

        self.reallocate_light_uniform_buffer(device, command_encoder, capacity);
    }

    fn reallocate_light_uniform_buffer(
        &mut self,
        device: &wgpu::Device,
        command_encoder: &mut wgpu::CommandEncoder,
        capacity: usize,
    ) {
        let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(POINT_LIGHT_UNIFORM_BUFFER_LABEL),
            size: (capacity as u32 * device.limits().min_uniform_buffer_offset_alignment)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let old_buffer_size =
            self.point_light_capacity as u32 * device.limits().min_uniform_buffer_offset_alignment;
        command_encoder.copy_buffer_to_buffer(
            &self.point_light_uniform_buffer,
            0,
            &new_buffer,
            0,
            old_buffer_size as wgpu::BufferAddress,
        );

        self.point_light_uniform_buffer = new_buffer;
        self.point_light_capacity = capacity;
    }

    pub fn render<'rpass: 'pass, 'pass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);

        render_pass.set_bind_group(0, &self.global_uniform_bind_group, &[]);

        if let Some(g_buffer_bind_group) = &self.g_buffer_bind_group {
            render_pass.set_bind_group(1, g_buffer_bind_group, &[]);
        }

        render_pass.set_bind_group(2, &self.point_light_uniform_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..VERTEX_COUNT as u32, 0..1);
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        global_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        g_buffer_bind_group_layout: &wgpu::BindGroupLayout,
        point_light_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("light_renderer_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/light.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("light_renderer_render_pipeline_layout"),
                bind_group_layouts: &[
                    global_uniform_bind_group_layout,
                    g_buffer_bind_group_layout,
                    point_light_uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("light_renderer_render_pipeline"),
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
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

    fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let vertices = vec![
            Vertex {
                position: [-1.0, 1.0, 1.0],
                texture_coordinates: [0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                texture_coordinates: [0.0, 1.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                texture_coordinates: [1.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                texture_coordinates: [1.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                texture_coordinates: [0.0, 1.0],
                ..Default::default()
            },
            Vertex {
                position: [1.0, -1.0, 1.0],
                texture_coordinates: [1.0, 1.0],
                ..Default::default()
            },
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_renderer_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_point_light_uniform_buffer(device: &wgpu::Device, capacity: u32) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(POINT_LIGHT_UNIFORM_BUFFER_LABEL),
            size: (capacity * device.limits().min_uniform_buffer_offset_alignment)
                as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    fn create_point_light_uniform_bind_group_layout(
        device: &wgpu::Device,
    ) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light_renderer_point_light_uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn create_point_light_uniform_bind_group(
        device: &wgpu::Device,
        point_light_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        point_light_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_renderer_point_light_uniform_bind_group"),
            layout: &point_light_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: point_light_uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn create_global_uniform_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(POINT_LIGHT_UNIFORM_BUFFER_LABEL),
            contents: bytemuck::cast_slice(&[GlobalUniform {
                ambient_light: DEFAULT_AMBIENT_LIGHT,
                light_count: 0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_global_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light_renderer_global_uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn create_global_uniform_bind_group(
        device: &wgpu::Device,
        global_uniform_bind_group_layout: &wgpu::BindGroupLayout,
        global_uniform_buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("light_renderer_global_uniform_bind_group"),
            layout: &global_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        })
    }

    fn create_g_buffer_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("light_renderer_g_buffer_bind_group_layout"),
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
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: false,
                        comparison: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_g_buffer_bind_group(
        device: &wgpu::Device,
        g_buffer_bind_group_layout: &wgpu::BindGroupLayout,
        g_buffer: GBuffer,
    ) -> wgpu::BindGroup {
        let albedo_map_view = g_buffer
            .albedo
            .create_view(&wgpu::TextureViewDescriptor::default());
        let albedo_map_sampler = Self::create_sampler(device);
        let normal_map_view = g_buffer
            .normal
            .create_view(&wgpu::TextureViewDescriptor::default());
        let normal_map_sampler = Self::create_sampler(device);
        let emission_map_view = g_buffer
            .emission
            .create_view(&wgpu::TextureViewDescriptor::default());
        let emission_map_sampler = Self::create_sampler(device);
        let position_map_view = g_buffer
            .position
            .create_view(&wgpu::TextureViewDescriptor::default());
        let position_map_sampler = Self::create_sampler(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &g_buffer_bind_group_layout,
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
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(&position_map_view),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&position_map_sampler),
                },
            ],
        })
    }

    fn create_sampler(device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    ambient_light: [f32; 3],
    light_count: i32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightUniform {
    position: [f32; 3],
    radius: f32,
    ambient_color: [f32; 3],
    _padding: u32,
    diffuse_color: [f32; 3],
    _padding2: u32,
    specular_color: [f32; 3],
    _padding3: u32,
}

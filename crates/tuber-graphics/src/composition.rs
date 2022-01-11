use crate::geometry::Vertex;
use crate::low_level::g_buffer::GBuffer;
use crate::GBufferComponent;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPipelineDescriptor,
    TextureViewDescriptor,
};

const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<GlobalUniform>() as u64;

pub(crate) struct Compositor {
    vertex_buffer: wgpu::Buffer,
    g_buffer_bind_group_layout: wgpu::BindGroupLayout,
    g_buffer_bind_group: Option<wgpu::BindGroup>,
    global_uniform: GlobalUniform,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Compositor {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(device);
        let g_buffer_bind_group_layout = Self::create_g_buffer_bind_group_layout(device);

        let global_uniform = GlobalUniform {
            rendered_g_buffer_component: 0,
        };
        let global_uniform_buffer = Self::create_global_uniform_buffer(device, &global_uniform);
        let global_uniform_bind_group_layout =
            Self::create_global_uniform_bind_group_layout(device);
        let global_uniform_bind_group = Self::create_global_uniform_bind_group(
            device,
            &global_uniform_bind_group_layout,
            &global_uniform_buffer,
        );

        let render_pipeline = Self::create_render_pipeline(
            device,
            surface_texture_format,
            &g_buffer_bind_group_layout,
            &global_uniform_bind_group_layout,
        );

        Self {
            vertex_buffer,
            g_buffer_bind_group_layout,
            g_buffer_bind_group: None,
            global_uniform,
            global_uniform_buffer,
            global_uniform_bind_group,
            render_pipeline,
        }
    }

    pub fn prepare(&mut self, device: &wgpu::Device, g_buffer: GBuffer) {
        self.g_buffer_bind_group = Some(Self::create_g_buffer_bind_group(
            device,
            &self.g_buffer_bind_group_layout,
            g_buffer,
        ));
    }

    pub fn render<'rpass: 'pass, 'pass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.global_uniform_bind_group, &[]);
        if let Some(texture_bind_group) = &self.g_buffer_bind_group {
            render_pass.set_bind_group(1, texture_bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..6, 0..1);
    }

    pub fn set_rendered_g_buffer_component(
        &mut self,
        queue: &wgpu::Queue,
        g_buffer_component: GBufferComponent,
    ) {
        self.global_uniform.rendered_g_buffer_component = g_buffer_component as i32;
        self.update_global_uniform(queue);
    }

    pub fn update_global_uniform(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.global_uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.global_uniform]),
        );
    }

    fn create_vertex_buffer(device: &wgpu::Device) -> wgpu::Buffer {
        let vertices = vec![
            Vertex {
                position: [-1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 1.0],
                color: [1.0, 1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
        ];

        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("compositor_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_g_buffer_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("compositor_g_buffer_bind_group_layout"),
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
            ],
        })
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        global_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("compositor_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/composition.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("compositor_render_pipeline_layout"),
            bind_group_layouts: &[global_uniform_bind_group_layout, texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("compositor_render_pipeline"),
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

    fn create_g_buffer_bind_group(
        device: &wgpu::Device,
        g_buffer_bind_group_layout: &wgpu::BindGroupLayout,
        g_buffer: GBuffer,
    ) -> wgpu::BindGroup {
        let albedo_map_view = g_buffer
            .albedo
            .create_view(&TextureViewDescriptor::default());
        let albedo_map_sampler = Self::create_g_buffer_sampler(device);
        let normal_map_view = g_buffer
            .normal
            .create_view(&TextureViewDescriptor::default());
        let normal_map_sampler = Self::create_g_buffer_sampler(device);

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
            ],
        })
    }

    fn create_g_buffer_sampler(device: &wgpu::Device) -> wgpu::Sampler {
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

    fn create_global_uniform_buffer(
        device: &wgpu::Device,
        global_uniform: &GlobalUniform,
    ) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("compositor_global_uniform_buffer"),
            contents: bytemuck::cast_slice(&[global_uniform.clone()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    fn create_global_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("compositor_global_uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(GLOBAL_UNIFORM_SIZE),
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
            label: Some("compositor_global_uniform_bind_groupt"),
            layout: global_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform_buffer.as_entire_binding(),
            }],
        })
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct GlobalUniform {
    rendered_g_buffer_component: i32,
}

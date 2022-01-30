use crate::geometry::Vertex;
use crate::GBufferComponent;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroupLayoutDescriptor, PipelineLayoutDescriptor, RenderPipelineDescriptor,
    TextureViewDescriptor,
};

const GLOBAL_UNIFORM_SIZE: u64 = std::mem::size_of::<GlobalUniform>() as u64;

pub(crate) struct Compositor {
    vertex_buffer: wgpu::Buffer,
    lit_render_bind_group_layout: wgpu::BindGroupLayout,
    lit_render_bind_group: Option<wgpu::BindGroup>,
    ui_render_bind_group_layout: wgpu::BindGroupLayout,
    ui_render_bind_group: Option<wgpu::BindGroup>,
    global_uniform: GlobalUniform,
    global_uniform_buffer: wgpu::Buffer,
    global_uniform_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

impl Compositor {
    pub fn new(device: &wgpu::Device, surface_texture_format: wgpu::TextureFormat) -> Self {
        let vertex_buffer = Self::create_vertex_buffer(device);
        let lit_render_bind_group_layout = Self::create_lit_render_bind_group_layout(device);
        let ui_render_bind_group_layout = Self::create_ui_render_bind_group_layout(device);

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
            &lit_render_bind_group_layout,
            &ui_render_bind_group_layout,
            &global_uniform_bind_group_layout,
        );

        Self {
            vertex_buffer,
            lit_render_bind_group_layout,
            lit_render_bind_group: None,
            ui_render_bind_group_layout,
            ui_render_bind_group: None,
            global_uniform,
            global_uniform_buffer,
            global_uniform_bind_group,
            render_pipeline,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        lit_render: &wgpu::Texture,
        ui_render: &wgpu::Texture,
    ) {
        self.lit_render_bind_group = Some(Self::create_lit_render_bind_group(
            device,
            &self.lit_render_bind_group_layout,
            lit_render,
        ));

        self.ui_render_bind_group = Some(Self::create_ui_render_bind_group(
            device,
            &self.ui_render_bind_group_layout,
            ui_render,
        ));
    }

    pub fn render<'rpass: 'pass, 'pass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.global_uniform_bind_group, &[]);
        if let Some(texture_bind_group) = &self.lit_render_bind_group {
            render_pass.set_bind_group(1, texture_bind_group, &[]);
        }

        if let Some(texture_bind_group) = &self.ui_render_bind_group {
            render_pass.set_bind_group(2, texture_bind_group, &[]);
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
            label: Some("compositor_vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn create_lit_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("compositor_lit_render_bind_group_layout"),
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

    fn create_ui_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("compositor_ui_render_bind_group_layout"),
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

    fn create_render_pipeline(
        device: &wgpu::Device,
        surface_texture_format: wgpu::TextureFormat,
        g_buffer_bind_group_layout: &wgpu::BindGroupLayout,
        ui_bind_group_layout: &wgpu::BindGroupLayout,
        global_uniform_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("compositor_shader_module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/composition.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("compositor_render_pipeline_layout"),
            bind_group_layouts: &[
                global_uniform_bind_group_layout,
                g_buffer_bind_group_layout,
                ui_bind_group_layout,
            ],
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

    fn create_lit_render_bind_group(
        device: &wgpu::Device,
        lit_render_bind_group_layout: &wgpu::BindGroupLayout,
        lit_render: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        let lit_render_view = lit_render.create_view(&TextureViewDescriptor::default());
        let lit_render_sampler = Self::create_sampler(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &lit_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&lit_render_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&lit_render_sampler),
                },
            ],
        })
    }

    fn create_ui_render_bind_group(
        device: &wgpu::Device,
        ui_render_bind_group_layout: &wgpu::BindGroupLayout,
        ui_render: &wgpu::Texture,
    ) -> wgpu::BindGroup {
        let render_view = ui_render.create_view(&TextureViewDescriptor::default());
        let render_sampler = Self::create_sampler(device);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &ui_render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&render_sampler),
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

use crate::low_level::mesh::Mesh;
use crate::low_level::primitives::Vertex;
use crate::GraphicsError;
use crate::GraphicsResult;
use crate::Window;
use futures::executor::block_on;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::*;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
        color: [1.0, 0.0, 0.0],
        texture_coordinates: [0.0, 0.0],
    },
    Vertex {
        position: [0.0, 1.0, 0.0],
        color: [0.0, 1.0, 0.0],
        texture_coordinates: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        color: [0.0, 0.0, 1.0],
        texture_coordinates: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 0.0, 0.0],
        color: [0.0, 0.0, 1.0],
        texture_coordinates: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[0, 3, 1, 3, 2, 1];

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    _surface_configuration: SurfaceConfiguration,
    _size: (u32, u32),

    render_pipeline: RenderPipeline,

    vertex_buffer: Buffer,
    index_buffer: Buffer,

    num_indices: u32,

    diffuse_bind_group: BindGroup,

    pending_meshes: Vec<Mesh>,
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

        let diffuse_bytes = include_bytes!("../../textures/default_texture.png");
        let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
        let diffuse_rgba = diffuse_image.as_rgba8().unwrap();
        use image::GenericImageView;
        let dimensions = diffuse_image.dimensions();

        let texture_size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(&TextureDescriptor {
            label: Some("diffuse_texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            diffuse_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture_size,
        );

        let diffuse_texture_view = diffuse_texture.create_view(&TextureViewDescriptor::default());
        let diffuse_texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture_sampler),
                },
            ],
        });

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("mesh_shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/mesh.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mesh_render_pipeline_layout"),
            bind_group_layouts: &[&texture_bind_group_layout],
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

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            _surface_configuration: surface_configuration,
            _size: window_size,

            render_pipeline,

            vertex_buffer,
            index_buffer,

            num_indices,

            diffuse_bind_group,

            pending_meshes: vec![],
        }
    }

    pub fn render(&mut self) -> GraphicsResult<()> {
        let output = self
            .surface
            .get_current_texture()
            .map_err(|e| GraphicsError::WGPUSurfaceError(e))?;
        let output_texture_view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut command_encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &output_texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
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
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(std::iter::once(command_encoder.finish()));
        output.present();

        self.clear_pending_meshes();
        Ok(())
    }

    pub fn queue_mesh(&mut self, mesh: Mesh) {
        self.pending_meshes.push(mesh);
    }

    fn clear_pending_meshes(&mut self) {
        self.pending_meshes.clear();
    }
}

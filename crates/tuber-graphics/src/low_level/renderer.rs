use crate::low_level::mesh::Mesh;
use crate::GraphicsError;
use crate::GraphicsResult;
use crate::Window;
use futures::executor::block_on;
use wgpu::*;

pub struct Renderer {
    surface: Surface,
    device: Device,
    queue: Queue,
    _surface_configuration: SurfaceConfiguration,
    _size: (u32, u32),

    render_pipeline: RenderPipeline,

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

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: Some("mesh_shader"),
            source: ShaderSource::Wgsl(include_str!("../shaders/mesh.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("mesh_render_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("mesh_render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
                clamp_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            surface,
            device,
            queue,
            _surface_configuration: surface_configuration,
            _size: window_size,

            render_pipeline,

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
            render_pass.draw(0..3, 0..1);
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

use log::trace;
use wgpu::{
    Color as WGPUColor, CommandEncoder as WGPUCommandEncoder, Device as WGPUDevice,
    LoadOp as WGPULoadOp, Operations as WGPUOperations,
    RenderPassColorAttachment as WGPURenderPassColorAttachment,
    RenderPassDescriptor as WGPURenderPassDescriptor, RenderPipeline as WGPURenderPipeline,
};

use crate::render_resource::RenderResourceHandle;
use crate::RenderResourceStore;

pub struct RenderGraph<'g> {
    _device: &'g WGPUDevice,
    command_encoder: &'g mut WGPUCommandEncoder,
    render_resource_store: &'g RenderResourceStore,
    passes: Vec<RenderPassDescriptor<'g>>,
    pipelines: Vec<Option<WGPURenderPipeline>>,
    color_attachments: Vec<Vec<Option<WGPURenderPassColorAttachment<'g>>>>,
}

impl<'g> RenderGraph<'g> {
    pub fn new(
        device: &'g WGPUDevice,
        command_encoder: &'g mut WGPUCommandEncoder,
        render_resource_store: &'g RenderResourceStore,
    ) -> Self {
        Self {
            _device: device,
            command_encoder,
            render_resource_store,
            passes: vec![],
            pipelines: vec![],
            color_attachments: vec![],
        }
    }

    pub fn add_render_pass(&mut self, render_pass_descriptor: RenderPassDescriptor<'g>) {
        trace!("Adding render pass \"{}\"", render_pass_descriptor.label);
        if render_pass_descriptor.fragment_shader.is_none()
            && render_pass_descriptor.vertex_shader.is_none()
        {
            self.pipelines.push(None);
        } else {
            self.pipelines
                .push(Some(self.generate_pipeline(&render_pass_descriptor)));
        }
        self.passes.push(render_pass_descriptor);
    }

    /// Generates an appropriate execution order for the render pass
    pub fn compile(&mut self) -> Vec<usize> {
        {
            for pass in &self.passes {
                let render_targets = pass
                    .outputs
                    .iter()
                    .filter(|output| output.is_a_texture_view())
                    .cloned()
                    .collect::<Vec<_>>();
                self.color_attachments
                    .push(Self::generate_color_attachments(
                        self.render_resource_store,
                        &render_targets,
                    ));
            }
        }

        (0..self.passes.len()).collect()
    }

    pub fn dispatch(&mut self, execution_order: &[usize]) {
        for &pass_index in execution_order.iter() {
            let current_pass = &mut self.passes[pass_index];
            let mut rpass = self
                .command_encoder
                .begin_render_pass(&WGPURenderPassDescriptor {
                    label: Some(current_pass.label),
                    color_attachments: &self.color_attachments[pass_index],
                    depth_stencil_attachment: None,
                });
            // set pipeline
            // bind stuff
            // dispatch
            (self.passes[pass_index].dispatch)(&mut rpass);
        }
    }

    fn generate_pipeline(
        &self,
        _render_pass_descriptor: &RenderPassDescriptor<'g>,
    ) -> WGPURenderPipeline {
        unimplemented!()
        /*let pipeline_layout = self
            .device
            .create_pipeline_layout(&WGPUPipelineLayoutDescriptor {
                label: Self::pipeline_layout_label(render_pass_descriptor.label),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        self.device
            .create_render_pipeline(&WGPURenderPipelineDescriptor {
                label: Self::pipeline_label(render_pass_descriptor.label),
                layout: Some(&pipeline_layout),
                vertex: WGPUVertexState {
                    module: &(),
                    entry_point: "vs_main",
                    buffers: &[],
                },
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                fragment: None,
                multiview: None,
            })*/
    }

    fn generate_color_attachments<'rs: 't, 't>(
        render_resource_store: &'rs RenderResourceStore,
        render_targets: &[RenderResourceHandle],
    ) -> Vec<Option<WGPURenderPassColorAttachment<'t>>> {
        render_targets
            .iter()
            .map(|&render_target| {
                Self::generate_color_attachment(render_resource_store, render_target)
            })
            .collect()
    }

    fn generate_color_attachment(
        render_resource_store: &RenderResourceStore,
        render_target: RenderResourceHandle,
    ) -> Option<WGPURenderPassColorAttachment> {
        let view = render_resource_store.texture_view(render_target).unwrap();
        Some(WGPURenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: WGPUOperations {
                load: WGPULoadOp::Clear(WGPUColor {
                    r: 0.1,
                    g: 0.2,
                    b: 0.3,
                    a: 1.0,
                }),
                store: true,
            },
        })
    }

    /*fn pipeline_layout_label(render_pass_label: &'a str) -> WGPULabel {
        Some(&format!("{}_pipeline_layout", render_pass_label))
    }

    fn pipeline_label(render_pass_label: &'a str) -> WGPULabel {
        Some(&format!("{}_pipeline", render_pass_label))
    }*/
}

pub struct RenderPassDescriptor<'a> {
    pub label: &'a str,
    pub inputs: Vec<RenderResourceHandle>,
    pub outputs: Vec<RenderResourceHandle>,
    pub vertex_shader: Option<String>,
    pub fragment_shader: Option<String>,
    pub dispatch: Box<dyn Fn(&mut wgpu::RenderPass<'_>)>,
}

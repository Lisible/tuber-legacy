use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use log::trace;
use wgpu::{LoadOp, PipelineLayoutDescriptor, RenderPipelineDescriptor};

use crate::{
    ClearColor, WGPUColor, WGPUCommandEncoder, WGPUDevice, WGPULoadOp, WGPUOperations,
    WGPUPipeline, WGPUPipelineLayout, WGPURenderPass, WGPURenderPassColorAttachment,
    WGPURenderPassDescriptor, WGPUTextureView,
};

pub struct RenderGraph<'res> {
    resources: &'res RenderGraphResources,
    device: &'res WGPUDevice,
    render_passes: Vec<RenderPass<'res>>,
    pass_execution_order: Vec<usize>,
}

impl<'g, 'res> RenderGraph<'res> {
    pub fn new(resources: &'res RenderGraphResources, device: &'res WGPUDevice) -> Self {
        Self {
            resources,
            device,
            render_passes: vec![],
            pass_execution_order: vec![1, 0],
        }
    }

    pub fn add_pass(&'g mut self, pass_identifier: &'g str) -> RenderPassBuilder<'g, 'res> {
        RenderPassBuilder::new(self, pass_identifier)
    }

    pub fn compile(&mut self) {}

    pub fn execute(&mut self, command_encoder: &mut WGPUCommandEncoder) {
        for &render_pass_index in &self.pass_execution_order {
            let render_pass = &self.render_passes[render_pass_index];
            trace!("Rendering pass {}", render_pass.identifier);
            let mut wgpu_render_pass =
                command_encoder.begin_render_pass(&WGPURenderPassDescriptor {
                    label: None,
                    color_attachments: &render_pass.color_attachments,
                    depth_stencil_attachment: None,
                });

            (render_pass.dispatch_fn)(&mut wgpu_render_pass);
        }
    }

    fn generate_pass(&mut self, render_pass_descriptor: RenderPassDescriptor<'g>) {
        let color_attachments: Vec<_> = render_pass_descriptor
            .color_attachments
            .iter()
            .map(|optional_color_attachment| {
                optional_color_attachment.as_ref().map(|color_attachment| {
                    let &ClearColor { r, g, b, a } = &color_attachment.clear_color;
                    WGPURenderPassColorAttachment {
                        view: &self
                            .resources
                            .texture_view(color_attachment.texture_view_handle),
                        resolve_target: None,
                        ops: WGPUOperations {
                            load: WGPULoadOp::Clear(WGPUColor { r, g, b, a }),
                            store: true,
                        },
                    }
                })
            })
            .collect();

        self.render_passes.push(RenderPass {
            identifier: render_pass_descriptor.identifier.into(),
            color_attachments,
            dispatch_fn: render_pass_descriptor.dispatch_fn,
        })
    }
}

pub struct RenderPassBuilder<'g, 'res> {
    render_graph: &'g mut RenderGraph<'res>,
    identifier: &'g str,
    color_attachments: Vec<Option<ColorAttachment>>,
    dispatch_fn: Option<Box<dyn Fn(&mut WGPURenderPass)>>,
}

impl<'g, 'res> RenderPassBuilder<'g, 'res> {
    fn new(render_graph: &'g mut RenderGraph<'res>, identifier: &'g str) -> Self {
        Self {
            render_graph,
            identifier,
            color_attachments: vec![],
            dispatch_fn: None,
        }
    }

    pub fn with_color_attachment(
        mut self,
        texture_view_handle: TextureViewHandle,
        clear_color: ClearColor,
    ) -> Self {
        self.color_attachments.push(Some(ColorAttachment {
            texture_view_handle,
            clear_color,
        }));
        self
    }

    pub fn dispatch<F>(mut self, dispatch_fn: F)
    where
        F: Fn(&mut WGPURenderPass) + 'static,
    {
        self.dispatch_fn = Some(Box::new(dispatch_fn));
        self.render_graph.generate_pass(RenderPassDescriptor {
            identifier: self.identifier,
            color_attachments: self.color_attachments,
            dispatch_fn: self.dispatch_fn.expect(&format!(
                "No dispatch function provided for pass {}",
                self.identifier
            )),
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TextureViewHandle {
    id: usize,
}

impl From<usize> for TextureViewHandle {
    fn from(id: usize) -> Self {
        Self { id }
    }
}

pub struct ColorAttachment {
    texture_view_handle: TextureViewHandle,
    clear_color: ClearColor,
}

struct RenderPassDescriptor<'a> {
    identifier: &'a str,
    color_attachments: Vec<Option<ColorAttachment>>,
    dispatch_fn: Box<dyn Fn(&mut WGPURenderPass)>,
}

struct RenderPass<'tex> {
    identifier: String,
    color_attachments: Vec<Option<WGPURenderPassColorAttachment<'tex>>>,
    dispatch_fn: Box<dyn Fn(&mut WGPURenderPass)>,
}

pub struct RenderGraphResources {
    texture_views: Vec<WGPUTextureView>,
}

impl RenderGraphResources {
    pub fn new() -> Self {
        Self {
            texture_views: vec![],
        }
    }

    pub fn import_texture_view(&mut self, texture_view: WGPUTextureView) -> TextureViewHandle {
        self.texture_views.push(texture_view);
        (self.texture_views.len() - 1).into()
    }

    fn texture_view(&self, texture_view_handle: TextureViewHandle) -> &WGPUTextureView {
        &self.texture_views[texture_view_handle.id]
    }
}

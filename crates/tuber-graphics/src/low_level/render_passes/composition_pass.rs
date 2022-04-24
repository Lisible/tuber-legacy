use wgpu::{SurfaceTexture, TextureViewDescriptor};

use crate::wgpu_state::RenderContext;

pub(crate) fn composition_pass(
    context: &mut RenderContext,
    command_encoder: &mut wgpu::CommandEncoder,
    surface: &wgpu::Surface,
    lit_render: &wgpu::Texture,
    ui_render: &wgpu::Texture,
) -> SurfaceTexture {
    let output_texture = surface.get_current_texture().unwrap();
    let output_texture_view = output_texture
        .texture
        .create_view(&TextureViewDescriptor::default());
    context
        .compositor
        .prepare(context.device, lit_render, ui_render);

    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composition_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
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

        context.compositor.render(&mut render_pass);
    }
    output_texture
}

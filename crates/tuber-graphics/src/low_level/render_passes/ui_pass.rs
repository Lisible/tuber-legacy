use tuber_math::matrix::{Identity, Matrix4};

use crate::low_level::renderers::quad_renderer::QuadRenderPassType;
use crate::low_level::texture::create_g_buffer_texture_descriptor;
use crate::wgpu_state::RenderContext;

pub(crate) fn ui_pass(
    context: &mut RenderContext,
    command_encoder: &mut wgpu::CommandEncoder,
) -> wgpu::Texture {
    let render_texture_descriptor =
        create_g_buffer_texture_descriptor("render_texture", context.viewport_size);
    let render_texture = context.device.create_texture(&render_texture_descriptor);
    let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let quad_group = context.quad_renderer.prepare_quad_group(
        context.device,
        command_encoder,
        context.textures,
        context.projection_matrix,
        &Matrix4::identity(),
        context.command_buffer.draw_ui_quad_commands(),
        true,
    );

    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("geometry_pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &render_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        context.quad_renderer.render_quad_group(
            &mut render_pass,
            QuadRenderPassType::UI,
            &quad_group,
        )
    }
    render_texture
}

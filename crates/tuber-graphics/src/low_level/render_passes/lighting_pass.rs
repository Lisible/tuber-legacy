use crate::low_level::g_buffer::GBuffer;
use crate::low_level::texture::create_g_buffer_texture_descriptor;
use crate::wgpu_state::RenderContext;
use crate::Color;

pub(crate) fn lighting_pass(
    context: &mut RenderContext,
    command_encoder: &mut wgpu::CommandEncoder,
    ambient_light: Color,
    g_buffer: GBuffer,
) -> wgpu::Texture {
    let render_texture_descriptor =
        create_g_buffer_texture_descriptor("render_texture", context.viewport_size);
    let render_texture = context.device.create_texture(&render_texture_descriptor);
    let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
    context.light_renderer.prepare(
        &context.device,
        &context.queue,
        command_encoder,
        ambient_light,
        g_buffer,
        context.command_buffer.draw_light_commands(),
    );

    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("lighting_pass"),
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

        context.light_renderer.render(&mut render_pass);
    }

    render_texture
}

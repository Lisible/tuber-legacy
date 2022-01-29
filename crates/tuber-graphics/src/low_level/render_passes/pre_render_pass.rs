use crate::quad_renderer::QuadRenderPassType;
use crate::wgpu_state::RenderContext;
use nalgebra::Matrix4;
use wgpu::TextureViewDescriptor;

pub(crate) fn pre_render_pass(
    context: &mut RenderContext,
    command_encoder: &mut wgpu::CommandEncoder,
) {
    for command in context.command_buffer.pre_draw_quads_commands() {
        let pre_render = &context.pre_renders[command.render_id.0];
        let albedo_map_id = pre_render.material.albedo_map_id;
        let normal_map_id = pre_render.material.normal_map_id;
        let emission_map_id = pre_render.material.emission_map_id;

        let albedo_texture = &context.textures[&albedo_map_id];
        let normal_texture = &context.textures[&normal_map_id];
        let emission_texture = &context.textures[&emission_map_id];

        let albedo_texture_view = albedo_texture.create_view(&TextureViewDescriptor::default());
        let normal_texture_view = normal_texture.create_view(&TextureViewDescriptor::default());
        let emission_texture_view = emission_texture.create_view(&TextureViewDescriptor::default());

        let quad_group = context.quad_renderer.prepare_quad_group(
            &context.device,
            &context.queue,
            &context.textures,
            &Matrix4::new_orthographic(
                0.0,
                pre_render.size.width,
                pre_render.size.height,
                0.0,
                -1.0,
                1.0,
            ),
            &Matrix4::identity(),
            &command.draw_quad_commands,
            false,
        );
        {
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("quad_pre_render_pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                        view: &albedo_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    },
                    wgpu::RenderPassColorAttachment {
                        view: &normal_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    },
                    wgpu::RenderPassColorAttachment {
                        view: &emission_texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            context.quad_renderer.render_quad_group(
                &mut render_pass,
                QuadRenderPassType::PreRender,
                &quad_group,
            )
        }
    }
}

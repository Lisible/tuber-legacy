use crate::draw_command::DrawQuadCommand;
use crate::low_level::g_buffer::GBuffer;
use crate::low_level::renderers::quad_renderer::QuadRenderPassType;
use crate::low_level::texture::{create_g_buffer_texture_descriptor, create_texture_descriptor};
use crate::primitives::Quad;
use crate::wgpu_state::RenderContext;

pub(crate) fn geometry_pass(
    context: &mut RenderContext,
    command_encoder: &mut wgpu::CommandEncoder,
) -> GBuffer {
    let albedo_map_texture_descriptor =
        create_g_buffer_texture_descriptor("albedo_map_texture", context.viewport_size);
    let albedo_map_texture = context
        .device
        .create_texture(&albedo_map_texture_descriptor);
    let albedo_map_view = albedo_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let normal_map_texture_descriptor = create_texture_descriptor(
        Some("normal_map_texture"),
        context.viewport_size,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let normal_map_texture = context
        .device
        .create_texture(&normal_map_texture_descriptor);
    let normal_map_view = normal_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let emission_map_texture_descriptor = create_texture_descriptor(
        Some("emission_map_texture"),
        context.viewport_size,
        wgpu::TextureFormat::Rgba8Unorm,
    );
    let emission_map_texture = context
        .device
        .create_texture(&emission_map_texture_descriptor);
    let emission_map_view =
        emission_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let position_map_texture_descriptor = create_texture_descriptor(
        Some("position_map_texture"),
        context.viewport_size,
        wgpu::TextureFormat::Rgba16Float,
    );
    let position_map_texture = context
        .device
        .create_texture(&position_map_texture_descriptor);
    let position_map_view =
        position_map_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut draw_commands = context
        .command_buffer
        .draw_quad_commands()
        .iter()
        .cloned()
        .collect::<Vec<_>>();

    draw_commands.sort_by(|first_draw_command, second_draw_command| {
        (first_draw_command.world_transform.column(3).z as f32)
            .partial_cmp(&(second_draw_command.world_transform.column(3).z as f32))
            .unwrap()
    });

    let quad_group = context.quad_renderer.prepare_quad_group(
        context.device,
        context.queue,
        command_encoder,
        context.textures,
        context.projection_matrix,
        context.view_transform,
        &draw_commands,
        false,
    );

    {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("geometry_pass"),
            color_attachments: &[
                wgpu::RenderPassColorAttachment {
                    view: &albedo_map_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: context.clear_color.r(),
                            g: context.clear_color.g(),
                            b: context.clear_color.b(),
                            a: 1.0,
                        }),
                        store: true,
                    },
                },
                wgpu::RenderPassColorAttachment {
                    view: &normal_map_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 0.5,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                },
                wgpu::RenderPassColorAttachment {
                    view: &emission_map_view,
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
                },
                wgpu::RenderPassColorAttachment {
                    view: &position_map_view,
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
                },
            ],
            depth_stencil_attachment: None,
        });

        context.quad_renderer.render_quad_group(
            &mut render_pass,
            QuadRenderPassType::Geometry,
            &quad_group,
        )
    }

    GBuffer {
        albedo: albedo_map_texture,
        normal: normal_map_texture,
        position: position_map_texture,
        emission: emission_map_texture,
    }
}

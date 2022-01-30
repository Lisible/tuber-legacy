use crate::graphics::RenderId;
use crate::primitives::Quad;
use crate::renderable::light::PointLight;
use crate::{Material, Size2};
use nalgebra::Matrix4;

pub struct CommandBuffer {
    pre_draw_quads_command_buffer: Vec<PreDrawQuadsCommand>,
    draw_pre_render_command_buffer: Vec<DrawPreRenderCommand>,
    draw_quad_command_buffer: Vec<DrawQuadCommand>,
    draw_ui_quad_command_buffer: Vec<DrawQuadCommand>,
    draw_light_command_buffer: Vec<DrawLightCommand>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            pre_draw_quads_command_buffer: vec![],
            draw_pre_render_command_buffer: vec![],
            draw_quad_command_buffer: vec![],
            draw_ui_quad_command_buffer: vec![],
            draw_light_command_buffer: vec![],
        }
    }

    pub fn add(&mut self, command: Command) {
        match command {
            Command::DrawQuad(draw_quad_command) => {
                self.draw_quad_command_buffer.push(draw_quad_command)
            }
            Command::PreDrawQuads(pre_draw_quads_command) => self
                .pre_draw_quads_command_buffer
                .push(pre_draw_quads_command),
            Command::DrawPreRender(draw_pre_render_command) => self
                .draw_pre_render_command_buffer
                .push(draw_pre_render_command),
            Command::DrawUIQuad(draw_quad_command) => {
                self.draw_ui_quad_command_buffer.push(draw_quad_command)
            }
            Command::DrawLight(draw_light_command) => {
                self.draw_light_command_buffer.push(draw_light_command)
            }
        }
    }

    pub fn draw_quad_commands(&self) -> &[DrawQuadCommand] {
        &self.draw_quad_command_buffer
    }

    pub fn draw_ui_quad_commands(&self) -> &[DrawQuadCommand] {
        &self.draw_ui_quad_command_buffer
    }

    pub fn pre_draw_quads_commands(&self) -> &[PreDrawQuadsCommand] {
        &self.pre_draw_quads_command_buffer
    }

    pub fn draw_pre_render_commands(&self) -> &[DrawPreRenderCommand] {
        &self.draw_pre_render_command_buffer
    }

    pub fn draw_light_commands(&self) -> &[DrawLightCommand] {
        &self.draw_light_command_buffer
    }

    pub fn clear(&mut self) {
        self.draw_quad_command_buffer.clear();
        self.draw_ui_quad_command_buffer.clear();
        self.pre_draw_quads_command_buffer.clear();
        self.draw_pre_render_command_buffer.clear();
        self.draw_light_command_buffer.clear();
    }
}

#[derive(Debug)]
pub enum Command {
    PreDrawQuads(PreDrawQuadsCommand),
    DrawPreRender(DrawPreRenderCommand),
    DrawQuad(DrawQuadCommand),
    DrawUIQuad(DrawQuadCommand),
    DrawLight(DrawLightCommand),
}

#[derive(Debug)]
pub struct DrawLightCommand {
    pub light: PointLight,
    pub world_transform: Matrix4<f32>,
}

#[derive(Debug)]
pub struct PreDrawQuadsCommand {
    pub render_id: RenderId,
    pub draw_quad_commands: Vec<DrawQuadCommand>,
}

#[derive(Debug)]
pub struct DrawPreRenderCommand {
    pub render_id: RenderId,
    pub size: Size2,
    pub world_transform: Matrix4<f32>,
}

#[derive(Debug, Clone)]
pub struct DrawQuadCommand {
    pub quad: Quad,
    pub world_transform: Matrix4<f32>,
    pub material: Material,
}

use tuber_math::matrix::Matrix4f;

use crate::primitives::{Mesh, Quad};
use crate::renderable::light::PointLight;
use crate::Material;

pub struct CommandBuffer {
    draw_mesh_command_buffer: Vec<DrawMeshCommand>,
    draw_quad_command_buffer: Vec<DrawQuadCommand>,
    draw_ui_quad_command_buffer: Vec<DrawQuadCommand>,
    draw_light_command_buffer: Vec<DrawLightCommand>,
}

impl CommandBuffer {
    pub fn new() -> Self {
        Self {
            draw_mesh_command_buffer: vec![],
            draw_quad_command_buffer: vec![],
            draw_ui_quad_command_buffer: vec![],
            draw_light_command_buffer: vec![],
        }
    }

    pub fn add(&mut self, command: DrawCommand) {
        match command {
            DrawCommand::Mesh(draw_mesh_command) => {
                self.draw_mesh_command_buffer.push(draw_mesh_command)
            }
            DrawCommand::Quad(draw_quad_command) => {
                self.draw_quad_command_buffer.push(draw_quad_command)
            }
            DrawCommand::UIQuad(draw_quad_command) => {
                self.draw_ui_quad_command_buffer.push(draw_quad_command)
            }
            DrawCommand::Light(draw_light_command) => {
                self.draw_light_command_buffer.push(draw_light_command)
            }
        }
    }

    pub fn draw_mesh_commands(&self) -> &[DrawMeshCommand] {
        &self.draw_mesh_command_buffer
    }

    pub fn draw_quad_commands(&self) -> &[DrawQuadCommand] {
        &self.draw_quad_command_buffer
    }

    pub fn draw_ui_quad_commands(&self) -> &[DrawQuadCommand] {
        &self.draw_ui_quad_command_buffer
    }

    pub fn draw_light_commands(&self) -> &[DrawLightCommand] {
        &self.draw_light_command_buffer
    }

    pub fn clear(&mut self) {
        self.draw_mesh_command_buffer.clear();
        self.draw_quad_command_buffer.clear();
        self.draw_ui_quad_command_buffer.clear();
        self.draw_light_command_buffer.clear();
    }
}

#[derive(Debug)]
pub enum DrawCommand {
    Quad(DrawQuadCommand),
    Mesh(DrawMeshCommand),
    UIQuad(DrawQuadCommand),
    Light(DrawLightCommand),
}

#[derive(Debug)]
pub struct DrawLightCommand {
    pub light: PointLight,
    pub world_transform: Matrix4f,
}

#[derive(Debug, Clone)]
pub struct DrawQuadCommand {
    pub quad: Quad,
    pub world_transform: Matrix4f,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct DrawMeshCommand {
    pub mesh: Mesh,
    pub world_transform: Matrix4f,
    pub material: Material,
}

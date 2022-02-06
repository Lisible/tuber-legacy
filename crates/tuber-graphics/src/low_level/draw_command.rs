use crate::primitives::{Mesh, Quad};
use crate::renderable::light::PointLight;
use crate::Material;
use nalgebra::Matrix4;

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

    pub fn add(&mut self, command: Command) {
        match command {
            Command::DrawMesh(draw_mesh_command) => {
                self.draw_mesh_command_buffer.push(draw_mesh_command)
            }
            Command::DrawQuad(draw_quad_command) => {
                self.draw_quad_command_buffer.push(draw_quad_command)
            }
            Command::DrawUIQuad(draw_quad_command) => {
                self.draw_ui_quad_command_buffer.push(draw_quad_command)
            }
            Command::DrawLight(draw_light_command) => {
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
pub enum Command {
    DrawQuad(DrawQuadCommand),
    DrawMesh(DrawMeshCommand),
    DrawUIQuad(DrawQuadCommand),
    DrawLight(DrawLightCommand),
}

#[derive(Debug)]
pub struct DrawLightCommand {
    pub light: PointLight,
    pub world_transform: Matrix4<f32>,
}

#[derive(Debug, Clone)]
pub struct DrawQuadCommand {
    pub quad: Quad,
    pub world_transform: Matrix4<f32>,
    pub material: Material,
}

#[derive(Debug, Clone)]
pub struct DrawMeshCommand {
    pub mesh: Mesh,
    pub world_transform: Matrix4<f32>,
    pub material: Material,
}

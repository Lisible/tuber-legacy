use crate::{
    Color, MaterialDescription, QuadDescription, Size2, TextureDescription, TextureMetadata,
};
use std::collections::HashMap;
use tuber_core::transform::Transform2D;

pub struct ImmediateGUI {
    commands: Vec<ImmediateGUICommand>,
}
impl ImmediateGUI {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }

    pub fn frame(&mut self, size: Size2, transform: Transform2D) {
        self.commands
            .push(ImmediateGUICommand::Frame(FrameCommand { size, transform }));
    }

    pub fn generate_quads(
        &mut self,
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Vec<QuadDescription> {
        self.commands
            .drain(..)
            .flat_map(|command| command.into_quad_descriptions(texture_metadata))
            .collect()
    }
}

enum ImmediateGUICommand {
    Frame(FrameCommand),
}

impl IntoQuadDescriptions for ImmediateGUICommand {
    fn into_quad_descriptions(
        self,
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Vec<QuadDescription> {
        match self {
            ImmediateGUICommand::Frame(frame_command) => {
                frame_command.into_quad_descriptions(texture_metadata)
            }
        }
    }
}

trait IntoQuadDescriptions {
    fn into_quad_descriptions(
        self,
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Vec<QuadDescription>;
}

struct FrameCommand {
    size: Size2,
    transform: Transform2D,
}

impl IntoQuadDescriptions for FrameCommand {
    fn into_quad_descriptions(
        self,
        texture_metadata: &HashMap<String, TextureMetadata>,
    ) -> Vec<QuadDescription> {
        vec![QuadDescription {
            size: self.size,
            color: Color::WHITE,
            material: MaterialDescription {
                albedo_map_description: TextureDescription::default_albedo_map_description(
                    texture_metadata,
                ),
                normal_map_description: TextureDescription::default_normal_map_description(
                    texture_metadata,
                ),
            },
            transform: self.transform,
        }]
    }
}

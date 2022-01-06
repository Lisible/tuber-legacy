use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::default::Default;

use crate::api::LowLevelGraphicsAPI;
use tuber_core::asset::{AssetMetadata, AssetStore, GenericLoader};
use tuber_core::transform::Transform2D;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::R;
use tuber_ecs::EntityIndex;

use crate::bitmap_font::BitmapFont;
use crate::camera::{Active, OrthographicCamera};
use crate::g_buffer::GBufferComponent;
use crate::low_level::*;
use crate::material::Material;
use crate::primitives::{MaterialDescription, QuadDescription, TextureDescription};
use crate::renderable::shape::RectangleShape;
use crate::renderable::sprite::{AnimatedSprite, Sprite};
use crate::texture::{
    texture_atlas_loader, texture_loader, TextureAtlas, TextureData, TextureMetadata,
    TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER,
};
use crate::types::{Color, Size2};

pub mod bitmap_font;
pub mod camera;
pub mod g_buffer;
pub mod low_level;
pub mod material;
pub mod renderable;
pub mod texture;
pub mod types;

#[derive(Debug)]
pub enum GraphicsError {
    TextureFileOpenError(std::io::Error),
    TextureMetadataNotFound,
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),
    BitmapFontFileReadError(std::io::Error),
}

pub struct Graphics {
    graphics_impl: Box<dyn LowLevelGraphicsAPI>,
    texture_metadata: HashMap<String, TextureMetadata>,
    pending_quads: Vec<QuadDescription>,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
            pending_quads: vec![],
        }
    }
    pub fn initialize(
        &mut self,
        window: Window,
        window_size: (u32, u32),
        asset_store: &mut AssetStore,
    ) {
        self.graphics_impl
            .initialize(window, window_size, asset_store);
        self.load_texture_in_vram(&texture::create_white_texture());
        self.load_texture_in_vram(&texture::create_placeholder_texture());
        self.load_texture_in_vram(&texture::create_normal_map_texture());
    }

    pub fn render(&mut self) {
        self.graphics_impl.draw_quads(&self.pending_quads);
        self.pending_quads.clear();
    }

    pub fn draw_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        self.pending_quads.push(QuadDescription {
            size: Size2::new(rectangle.width, rectangle.height),
            color: rectangle.color.into(),
            material: MaterialDescription::default(),
            transform: transform.clone(),
        });
    }

    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
        asset_manager: &mut AssetStore,
    ) -> Result<(), GraphicsError> {
        self.load_texture_in_vram_if_required(asset_manager, &sprite.material.albedo_map);

        let texture_metadata = self.texture_metadata.get(&sprite.material.albedo_map);
        let texture_metadata = match texture_metadata {
            Some(metadata) => metadata,
            None => &TextureMetadata {
                width: 32,
                height: 32,
            },
        };

        let (texture_width, texture_height) = (
            texture_metadata.width as f32,
            texture_metadata.height as f32,
        );

        let effective_transform = Transform2D {
            translation: (
                transform.translation.0 + sprite.offset.0,
                transform.translation.1 + sprite.offset.1,
                transform.translation.2 + sprite.offset.2,
            ),
            ..*transform
        };

        self.pending_quads.push(QuadDescription {
            size: Size2::new(sprite.width, sprite.height),
            color: Color::WHITE.into(),
            material: MaterialDescription {
                albedo_map_description: Some(TextureDescription {
                    identifier: sprite.material.albedo_map.clone(),
                    texture_region: TextureRegion {
                        x: sprite.texture_region.x / texture_width,
                        y: sprite.texture_region.y / texture_height,
                        width: sprite.texture_region.width / texture_width,
                        height: sprite.texture_region.height / texture_height,
                    },
                }),
                normal_map_description: None,
            },
            transform: effective_transform.clone(),
        });
        Ok(())
    }

    pub fn draw_animated_sprite(
        &mut self,
        animated_sprite: &AnimatedSprite,
        transform: &Transform2D,
        asset_store: &mut AssetStore,
    ) -> Result<(), GraphicsError> {
        self.load_material_in_vram_if_required(asset_store, &animated_sprite.material);

        let TextureMetadata { width, height } = self
            .texture_metadata
            .get(&animated_sprite.material.albedo_map)
            .ok_or(GraphicsError::TextureMetadataNotFound)?;

        let (texture_width, texture_height) = (*width as f32, *height as f32);

        let current_keyframe = animated_sprite.animation_state.keyframes
            [animated_sprite.animation_state.current_keyframe];

        let mut normalized_texture_region = TextureRegion::new(
            current_keyframe.x / texture_width,
            current_keyframe.y / texture_height,
            current_keyframe.width / texture_width,
            current_keyframe.height / texture_height,
        );

        if animated_sprite.animation_state.flip_x {
            normalized_texture_region = normalized_texture_region.flip_x();
        }

        self.pending_quads.push(QuadDescription {
            size: Size2::new(animated_sprite.width, animated_sprite.height),
            color: Color::WHITE.into(),
            material: MaterialDescription {
                albedo_map_description: Some(TextureDescription {
                    identifier: animated_sprite.material.albedo_map.clone(),
                    texture_region: normalized_texture_region,
                }),
                normal_map_description: Some(TextureDescription {
                    identifier: animated_sprite
                        .material
                        .normal_map
                        .clone()
                        .unwrap_or(DEFAULT_NORMAL_MAP_IDENTIFIER.into()),
                    texture_region: normalized_texture_region,
                }),
            },
            transform: transform.clone(),
        });

        Ok(())
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font_identifier: &str,
        transform: &Transform2D,
        asset_store: &mut AssetStore,
    ) {
        let (font_atlas, font_texture) = {
            let font = asset_store.asset::<BitmapFont>(font_identifier).unwrap();
            (
                font.font_atlas().to_string(),
                font.font_atlas_texture().to_string(),
            )
        };

        {
            asset_store.load::<TextureAtlas>(&font_atlas).unwrap();
            asset_store.load::<TextureData>(&font_texture).unwrap();
            self.load_texture_in_vram_if_required(asset_store, &font_texture);
        }

        let font = asset_store
            .stored_asset::<BitmapFont>(font_identifier)
            .unwrap();
        let texture_atlas = asset_store
            .stored_asset::<TextureAtlas>(font.font_atlas())
            .unwrap();

        let texture = &self.texture_metadata[&font_texture];
        let font_region = texture_atlas
            .texture_region(font_identifier)
            .expect("Font region not found");

        let mut offset_x = transform.translation.0;
        let mut offset_y = transform.translation.1;
        for character in text.chars() {
            if character == '\n' {
                offset_y += (font.line_height() + font.line_spacing()) as f32;
                offset_x = transform.translation.0;
                continue;
            }

            let glyph_data = if font.ignore_case() {
                if let Some(glyph) = font.glyph(character.to_ascii_uppercase()) {
                    glyph
                } else {
                    font.glyph(character.to_ascii_lowercase())
                        .expect("Glyph not found")
                }
            } else {
                font.glyph(character).expect("Glyph not found")
            };

            let glyph_region = glyph_data.region();
            let mut glyph_transform = transform.clone();
            glyph_transform.translation.0 = offset_x;
            glyph_transform.translation.1 = offset_y;
            glyph_transform.rotation_center = (-offset_x, -offset_y);

            self.pending_quads.push(QuadDescription {
                size: Size2::new(glyph_region.width, glyph_region.height),
                color: Color::BLACK.into(),
                material: MaterialDescription {
                    albedo_map_description: Some(TextureDescription {
                        identifier: font_texture.clone().into(),
                        texture_region: TextureRegion {
                            x: (font_region.x + glyph_region.x) / texture.width as f32,
                            y: (font_region.y + glyph_region.y) / texture.height as f32,
                            width: glyph_region.width / texture.width as f32,
                            height: glyph_region.height / texture.height as f32,
                        },
                    }),
                    normal_map_description: None,
                },
                transform: glyph_transform.clone(),
            });

            offset_x += glyph_region.width + font.letter_spacing() as f32;
        }
    }

    fn load_material_in_vram_if_required(
        &mut self,
        asset_manager: &mut AssetStore,
        material: &Material,
    ) {
        self.load_texture_in_vram_if_required(asset_manager, &material.albedo_map);
        if let Some(normal_map) = &material.normal_map {
            self.load_texture_in_vram_if_required(asset_manager, normal_map);
        }
    }

    fn load_texture_in_vram_if_required(
        &mut self,
        asset_manager: &mut AssetStore,
        texture_identifier: &str,
    ) {
        if !self.graphics_impl.is_texture_in_vram(texture_identifier) {
            self.load_texture_from_asset_in_vram(asset_manager, texture_identifier);
        }
    }

    fn load_texture_from_asset_in_vram(
        &mut self,
        asset_manager: &mut AssetStore,
        texture_identifier: &str,
    ) {
        let texture = asset_manager.asset::<TextureData>(texture_identifier);
        if let Ok(texture) = texture {
            self.load_texture_in_vram(texture);
        }
    }

    fn load_texture_in_vram(&mut self, texture: &TextureData) {
        self.texture_metadata.insert(
            texture.identifier.clone(),
            TextureMetadata {
                width: texture.size.0,
                height: texture.size.1,
            },
        );

        self.graphics_impl.load_texture_in_vram(texture);
    }

    pub fn render_scene(&mut self, ecs: &Ecs, asset_store: &mut AssetStore) {
        let (camera_id, (camera, _, camera_transform)) = ecs
            .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
            .expect("There is no camera");
        self.update_camera(camera_id, &camera, &camera_transform);

        for (_, (rectangle_shape, transform)) in ecs.query::<(R<RectangleShape>, R<Transform2D>)>()
        {
            self.draw_rectangle(&rectangle_shape, &transform);
        }
        for (_, (sprite, transform)) in ecs.query::<(R<Sprite>, R<Transform2D>)>() {
            self.draw_sprite(&sprite, &transform, asset_store).unwrap();
        }
        for (_, (animated_sprite, transform)) in ecs.query::<(R<AnimatedSprite>, R<Transform2D>)>()
        {
            self.draw_animated_sprite(&animated_sprite, &transform, asset_store)
                .unwrap();
        }

        self.render();
    }

    pub fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform: &Transform2D,
    ) {
        self.graphics_impl
            .update_camera(camera_id, camera, transform);
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.graphics_impl.set_clear_color(clear_color);
    }

    pub fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent) {
        self.graphics_impl
            .set_rendered_g_buffer_component(g_buffer_component);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics_impl.on_window_resized((width, height));
    }

    pub fn loaders() -> Vec<(TypeId, GenericLoader)> {
        vec![
            (TypeId::of::<TextureData>(), Box::new(texture_loader)),
            (TypeId::of::<TextureAtlas>(), Box::new(texture_atlas_loader)),
            (TypeId::of::<BitmapFont>(), Box::new(font_loader)),
        ]
    }
}

fn font_loader(asset_metadata: &AssetMetadata) -> Box<dyn Any> {
    let mut font_file_path = asset_metadata.asset_path.clone();
    font_file_path.push(&asset_metadata.metadata["font_data"]);
    Box::new(BitmapFont::from_file(&font_file_path).unwrap())
}

pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

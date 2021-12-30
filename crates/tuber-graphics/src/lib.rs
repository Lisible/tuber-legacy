use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::default::Default;

use tuber_core::asset::{AssetMetadata, AssetStore, GenericLoader};
use tuber_core::tilemap::Tilemap;
use tuber_core::transform::Transform2D;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::{R, W};
use tuber_ecs::system::SystemBundle;
use tuber_ecs::EntityIndex;

use crate::bitmap_font::BitmapFont;
use crate::camera::{Active, OrthographicCamera};
use crate::low_level::*;
use crate::material::{Material, MaterialTexture};
use crate::shape::RectangleShape;
use crate::sprite::{sprite_animation_step_system, AnimatedSprite, Sprite};
use crate::texture::{
    texture_atlas_loader, texture_loader, TextureAtlas, TextureData, TextureMetadata, TextureRegion,
};
use crate::tilemap::TilemapRender;
use crate::ui::{Frame, Image, NoViewTransform, Text};

pub mod bitmap_font;
pub mod camera;
pub mod low_level;
pub mod material;
pub mod shape;
pub mod sprite;
pub mod texture;
pub mod tilemap;
pub mod ui;

#[derive(Debug)]
pub enum GraphicsError {
    TextureFileOpenError(std::io::Error),
    TextureMetadataNotFound,
    AtlasDescriptionFileOpenError(std::io::Error),
    ImageDecodeError(ImageError),
    SerdeError(serde_json::error::Error),
    BitmapFontFileReadError(std::io::Error),
}

pub type Color = (f32, f32, f32);

pub type WindowSize = (u32, u32);
pub struct Window<'a>(pub Box<&'a dyn HasRawWindowHandle>);
unsafe impl HasRawWindowHandle for Window<'_> {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.0.raw_window_handle()
    }
}

pub struct Graphics {
    graphics_impl: Box<dyn LowLevelGraphicsAPI>,
    texture_metadata: HashMap<String, TextureMetadata>,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
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
    }

    pub fn render(&mut self) {
        self.graphics_impl.render();
    }

    pub fn prepare_rectangle(
        &mut self,
        rectangle: &RectangleShape,
        transform: &Transform2D,
        apply_view_transform: bool,
    ) {
        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: rectangle.width,
                height: rectangle.height,
                color: rectangle.color,
                texture: None,
            },
            transform,
            apply_view_transform,
        );
    }

    pub fn prepare_animated_sprite(
        &mut self,
        animated_sprite: &AnimatedSprite,
        transform: &Transform2D,
        apply_view_transform: bool,
        asset_store: &mut AssetStore,
    ) -> Result<(), GraphicsError> {
        self.load_texture_in_vram_if_required(asset_store, &animated_sprite.texture_identifier);

        let TextureMetadata { width, height } = self
            .texture_metadata
            .get(&animated_sprite.texture_identifier)
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

        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: animated_sprite.width,
                height: animated_sprite.height,
                color: (1.0, 1.0, 1.0),
                texture: Some(TextureDescription {
                    identifier: animated_sprite.texture_identifier.clone(),
                    texture_region: normalized_texture_region,
                }),
            },
            transform,
            apply_view_transform,
        );

        Ok(())
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

    pub fn prepare_sprite(
        &mut self,
        sprite: &Sprite,
        transform: &Transform2D,
        apply_view_transform: bool,
        asset_manager: &mut AssetStore,
    ) -> Result<(), GraphicsError> {
        self.load_texture_in_vram_if_required(
            asset_manager,
            dbg!(&sprite.material.albedo_map.identifier),
        );

        let texture_metadata = self
            .texture_metadata
            .get(&sprite.material.albedo_map.identifier);
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

        self.graphics_impl.prepare_quad(
            &QuadDescription {
                width: sprite.width,
                height: sprite.height,
                color: (1.0, 1.0, 1.0),
                texture: Some(TextureDescription {
                    identifier: sprite.material.albedo_map.identifier.clone(),
                    texture_region: TextureRegion {
                        x: sprite.material.albedo_map.region.x / texture_width,
                        y: sprite.material.albedo_map.region.y / texture_height,
                        width: sprite.material.albedo_map.region.width / texture_width,
                        height: sprite.material.albedo_map.region.height / texture_height,
                    },
                }),
            },
            &effective_transform,
            apply_view_transform,
        );
        Ok(())
    }

    pub fn prepare_tilemap(
        &mut self,
        tilemap: &Tilemap,
        tilemap_render: &TilemapRender,
        transform: &Transform2D,
        asset_store: &mut AssetStore,
    ) {
        {
            asset_store
                .load::<TextureAtlas>(&tilemap_render.texture_atlas_identifier)
                .unwrap();
            asset_store
                .load::<TextureData>(&tilemap_render.texture_identifier)
                .unwrap();
            self.load_texture_in_vram_if_required(asset_store, &tilemap_render.texture_identifier);
        }

        self.graphics_impl
            .prepare_tilemap(tilemap, tilemap_render, transform, asset_store);
    }

    pub fn prepare_text(
        &mut self,
        text: &str,
        font_identifier: &str,
        transform: &Transform2D,
        apply_view_transform: bool,
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

            self.graphics_impl.prepare_quad(
                &QuadDescription {
                    width: glyph_region.width,
                    height: glyph_region.height,
                    color: (0.0, 0.0, 0.0),
                    texture: Some(TextureDescription {
                        identifier: font_texture.clone().into(),
                        texture_region: TextureRegion {
                            x: (font_region.x + glyph_region.x) / texture.width as f32,
                            y: (font_region.y + glyph_region.y) / texture.height as f32,
                            width: glyph_region.width / texture.width as f32,
                            height: glyph_region.height / texture.height as f32,
                        },
                    }),
                },
                &glyph_transform,
                apply_view_transform,
            );

            offset_x += glyph_region.width + font.letter_spacing() as f32;
        }
    }

    pub fn render_scene(&mut self, ecs: &Ecs, asset_store: &mut AssetStore) {
        let (camera_id, (camera, _, camera_transform)) = ecs
            .query_one::<(R<OrthographicCamera>, R<Active>, R<Transform2D>)>()
            .expect("There is no camera");
        self.update_camera(camera_id, &camera, &camera_transform);

        for (_, (tilemap, tilemap_render, transform)) in
            ecs.query::<(R<Tilemap>, R<TilemapRender>, R<Transform2D>)>()
        {
            self.prepare_tilemap(&tilemap, &tilemap_render, &transform, asset_store);
        }

        for (_, (rectangle_shape, transform)) in ecs.query::<(R<RectangleShape>, R<Transform2D>)>()
        {
            self.prepare_rectangle(&rectangle_shape, &transform, true);
        }
        for (_, (sprite, transform)) in ecs.query::<(R<Sprite>, R<Transform2D>)>() {
            self.prepare_sprite(&sprite, &transform, true, asset_store)
                .unwrap();
        }
        for (_, (animated_sprite, transform)) in ecs.query::<(R<AnimatedSprite>, R<Transform2D>)>()
        {
            self.prepare_animated_sprite(&animated_sprite, &transform, true, asset_store)
                .unwrap();
        }

        for (_, (mut tilemap_render,)) in ecs.query::<(W<TilemapRender>,)>() {
            tilemap_render.dirty = false;
        }

        for (id, (frame, transform)) in ecs.query::<(R<Frame>, R<Transform2D>)>() {
            let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
            self.prepare_rectangle(
                &RectangleShape {
                    width: frame.width,
                    height: frame.height,
                    color: frame.color,
                },
                &transform,
                apply_view_transform,
            );
        }

        for (id, (text, transform)) in ecs.query::<(R<Text>, R<Transform2D>)>() {
            let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
            self.prepare_text(
                text.text(),
                text.font(),
                &transform,
                apply_view_transform,
                asset_store,
            );
        }

        for (id, (image, transform)) in ecs.query::<(R<Image>, R<Transform2D>)>() {
            let apply_view_transform = !ecs.query_one_by_id::<(R<NoViewTransform>,)>(id).is_some();
            let sprite = Sprite {
                width: image.width,
                height: image.height,
                offset: (0.0, 0.0, 0),
                material: Material {
                    albedo_map: MaterialTexture {
                        identifier: image.texture_identifier.clone(),
                        region: image.texture_region,
                    },
                    ..Default::default()
                },
            };

            self.prepare_sprite(&sprite, &transform, apply_view_transform, asset_store)
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

    pub fn default_system_bundle() -> SystemBundle<()> {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(sprite_animation_step_system);
        system_bundle
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.graphics_impl.set_clear_color(clear_color);
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

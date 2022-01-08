use image::ImageError;
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::any::{Any, TypeId};
use std::collections::HashMap;

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
use crate::polygon_mode::PolygonMode;
use crate::primitives::{MaterialDescription, QuadDescription, TextureDescription};
use crate::renderable::shape::RectangleShape;
use crate::renderable::sprite::{AnimatedSprite, Sprite};
use crate::renderable::tilemap::{Tile, Tilemap};
use crate::texture::{
    texture_atlas_loader, texture_loader, TextureAtlas, TextureData, TextureMetadata,
    TextureRegion, DEFAULT_NORMAL_MAP_IDENTIFIER, MISSING_TEXTURE_IDENTIFIER,
};
use crate::types::{Color, Size2};

pub mod animation;
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
    tilemap_renders: Vec<QuadDescription>,
}

impl Graphics {
    pub fn new(graphics_impl: Box<dyn LowLevelGraphicsAPI>) -> Self {
        Self {
            graphics_impl,
            texture_metadata: HashMap::new(),
            pending_quads: vec![],
            tilemap_renders: vec![],
        }
    }
    pub fn initialize(
        &mut self,
        window: Window,
        window_size: Size2<u32>,
        asset_store: &mut AssetStore,
    ) {
        self.graphics_impl
            .initialize(window, window_size, asset_store);
        self.load_texture_in_vram(&texture::create_white_texture());
        self.load_texture_in_vram(&texture::create_placeholder_texture());
        self.load_texture_in_vram(&texture::create_normal_map_texture());
    }

    pub fn render(&mut self) {
        self.pending_quads.sort_by(|quad, other| {
            quad.transform
                .translation
                .2
                .cmp(&other.transform.translation.2)
        });
        self.graphics_impl.draw_quads(&self.pending_quads);
        self.pending_quads.clear();
    }

    pub fn draw_rectangle(&mut self, rectangle: &RectangleShape, transform: &Transform2D) {
        self.pending_quads.push(QuadDescription {
            size: Size2::new(rectangle.width, rectangle.height),
            color: rectangle.color.into(),
            material: MaterialDescription {
                albedo_map_description: TextureDescription::default_albedo_map_description(
                    &self.texture_metadata,
                ),
                normal_map_description: TextureDescription::default_normal_map_description(
                    &self.texture_metadata,
                ),
            },
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
            None => &self.texture_metadata[MISSING_TEXTURE_IDENTIFIER],
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
                albedo_map_description: TextureDescription {
                    identifier: self.texture_metadata[&sprite.material.albedo_map].texture_id,
                    texture_region: TextureRegion {
                        x: sprite.texture_region.x / texture_width,
                        y: sprite.texture_region.y / texture_height,
                        width: sprite.texture_region.width / texture_width,
                        height: sprite.texture_region.height / texture_height,
                    },
                },
                normal_map_description: match &sprite.material.normal_map {
                    Some(normal_map) => TextureDescription {
                        identifier: self.texture_metadata[normal_map].texture_id,
                        texture_region: TextureRegion {
                            x: sprite.texture_region.x / texture_width,
                            y: sprite.texture_region.y / texture_height,
                            width: sprite.texture_region.width / texture_width,
                            height: sprite.texture_region.height / texture_height,
                        },
                    },
                    None => {
                        TextureDescription::default_normal_map_description(&self.texture_metadata)
                    }
                },
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

        let TextureMetadata { width, height, .. } = self
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
                albedo_map_description: TextureDescription {
                    identifier: self.texture_metadata[&animated_sprite.material.albedo_map]
                        .texture_id,
                    texture_region: normalized_texture_region,
                },
                normal_map_description: TextureDescription {
                    identifier: match &animated_sprite.material.normal_map {
                        Some(normal_map) => self.texture_metadata[normal_map].texture_id,
                        None => self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
                    },
                    texture_region: normalized_texture_region,
                },
            },
            transform: transform.clone(),
        });

        Ok(())
    }

    pub fn draw_tilemap(
        &mut self,
        asset_store: &mut AssetStore,
        tilemap: &mut Tilemap,
        transform: &Transform2D,
    ) {
        if tilemap.render_id().is_none() {
            let tilemap_size = tilemap.size().clone();
            let tile_size = tilemap.tile_size().clone();

            let tilemap_size_pixel = Size2::new(
                tilemap_size.width as u32 * tile_size.width,
                tilemap_size.height as u32 * tile_size.height,
            );

            let destination_quad = self.graphics_impl.create_transparent_quad(Size2::new(
                tilemap_size_pixel.width as f32,
                tilemap_size_pixel.height as f32,
            ));

            self.render_entire_tilemap(
                asset_store,
                tilemap,
                &tilemap_size,
                &tile_size,
                &destination_quad,
            );

            let next_render_id = self.tilemap_renders.len();
            self.tilemap_renders.push(destination_quad);
            tilemap.set_render_id(RenderId(next_render_id));
        } else if tilemap
            .tiles()
            .iter()
            .any(|tile| matches!(tile, &Some(Tile::AnimatedTile(_))))
        {
            self.rerender_animated_tiles(tilemap);
        }

        self.pending_quads.push(QuadDescription {
            transform: transform.clone(),
            ..self.tilemap_renders[tilemap.render_id().as_ref().unwrap().0].clone()
        });
    }

    fn rerender_animated_tiles(&mut self, tilemap: &Tilemap) {
        let render_id = tilemap.render_id().unwrap();
        let render = &self.tilemap_renders[render_id.0];
        let mut quads = vec![];

        let tilemap_material = tilemap.material();
        let albedo_map_texture_metadata = &self.texture_metadata[&tilemap_material.albedo_map];

        for (tile_index, tile) in tilemap
            .tiles()
            .iter()
            .enumerate()
            .filter(|&tile| matches!(tile, (_, &Some(Tile::AnimatedTile(_)))))
            .map(|(tile_index, tile)| match tile {
                Some(Tile::AnimatedTile(animated_tile)) => (tile_index, animated_tile),
                _ => panic!(),
            })
        {
            let tile_x = tile_index % tilemap.size().width;
            let tile_y = tile_index / tilemap.size().height;
            quads.push(QuadDescription {
                size: Size2::new(
                    tilemap.tile_size().width as f32,
                    tilemap.tile_size().height as f32,
                ),
                color: Color::WHITE,
                material: MaterialDescription {
                    albedo_map_description: TextureDescription {
                        identifier: albedo_map_texture_metadata.texture_id,
                        texture_region: tile.animation_state.keyframes
                            [tile.animation_state.current_keyframe]
                            .normalize(
                                albedo_map_texture_metadata.width,
                                albedo_map_texture_metadata.height,
                            ),
                    },
                    normal_map_description: match &tilemap_material.normal_map {
                        Some(normal_map_identifier) => TextureDescription {
                            identifier: self.texture_metadata[normal_map_identifier].texture_id,
                            texture_region: tile.animation_state.keyframes
                                [tile.animation_state.current_keyframe]
                                .normalize(
                                    albedo_map_texture_metadata.width,
                                    albedo_map_texture_metadata.height,
                                ),
                        },
                        None => TextureDescription::default_normal_map_description(
                            &self.texture_metadata,
                        ),
                    },
                },
                transform: Transform2D {
                    translation: (
                        (tile_x * tilemap.tile_size().width as usize) as f32,
                        (tile_y * tilemap.tile_size().height as usize) as f32,
                        0,
                    ),
                    ..Default::default()
                },
            })
        }

        self.graphics_impl.pre_draw_quads(render, &quads);
    }

    fn render_entire_tilemap(
        &mut self,
        asset_store: &mut AssetStore,
        tilemap: &mut Tilemap,
        tilemap_size: &Size2<usize>,
        tile_size: &Size2<u32>,
        destination_quad: &QuadDescription,
    ) {
        let tilemap_material = tilemap.material();
        self.load_material_in_vram_if_required(asset_store, tilemap_material);

        let albedo_map_texture_metadata = &self.texture_metadata[&tilemap_material.albedo_map];

        let mut quads = vec![];
        for (tile_index, tile) in tilemap.tiles().iter().enumerate() {
            let tile_x = tile_index % tilemap_size.width;
            let tile_y = tile_index / tilemap_size.height;
            if let Some(tile) = tile {
                let tile_texture_region = match tile {
                    Tile::StaticTile(static_tile) => &static_tile.texture_region,
                    Tile::AnimatedTile(animated_tile) => {
                        &animated_tile.animation_state.keyframes
                            [animated_tile.animation_state.current_keyframe]
                    }
                };

                quads.push(QuadDescription {
                    size: Size2::new(tile_size.width as f32, tile_size.height as f32),
                    color: Color::WHITE,
                    material: MaterialDescription {
                        albedo_map_description: TextureDescription {
                            identifier: albedo_map_texture_metadata.texture_id,
                            texture_region: tile_texture_region.normalize(
                                albedo_map_texture_metadata.width,
                                albedo_map_texture_metadata.height,
                            ),
                        },
                        normal_map_description: match &tilemap_material.normal_map {
                            Some(normal_map_identifier) => TextureDescription {
                                identifier: self.texture_metadata[normal_map_identifier].texture_id,
                                texture_region: tile_texture_region.normalize(
                                    albedo_map_texture_metadata.width,
                                    albedo_map_texture_metadata.height,
                                ),
                            },
                            None => TextureDescription::default_normal_map_description(
                                &self.texture_metadata,
                            ),
                        },
                    },
                    transform: Transform2D {
                        translation: (
                            (tile_x * tile_size.width as usize) as f32,
                            (tile_y * tile_size.height as usize) as f32,
                            0,
                        ),
                        ..Default::default()
                    },
                })
            }
        }

        self.graphics_impl.pre_draw_quads(&destination_quad, &quads);
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
                    albedo_map_description: TextureDescription {
                        identifier: self.texture_metadata[&font_texture].texture_id,
                        texture_region: TextureRegion {
                            x: (font_region.x + glyph_region.x) / texture.width as f32,
                            y: (font_region.y + glyph_region.y) / texture.height as f32,
                            width: glyph_region.width / texture.width as f32,
                            height: glyph_region.height / texture.height as f32,
                        },
                    },
                    normal_map_description: TextureDescription::default_normal_map_description(
                        &self.texture_metadata,
                    ),
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
        if !self.texture_metadata.contains_key(texture_identifier) {
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
        let texture_id = self.graphics_impl.load_texture_in_vram(texture);

        self.texture_metadata.insert(
            texture.identifier.clone(),
            TextureMetadata {
                texture_id,
                width: texture.size.0,
                height: texture.size.1,
            },
        );
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

    pub fn set_polygon_mode(&mut self, polygon_mode: PolygonMode) {
        self.graphics_impl.set_polygon_mode(polygon_mode);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.graphics_impl
            .on_window_resized(Size2::from((width, height)));
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

#[derive(Copy, Clone)]
pub struct RenderId(pub usize);

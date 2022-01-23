use crate::draw_command::{Command, DrawPreRenderCommand, DrawQuadCommand, PreDrawQuadsCommand};
use crate::font::DEFAULT_FONT_IDENTIFIER;
use crate::geometry::Vertex;
use crate::primitives::Quad;
use crate::texture::{MISSING_TEXTURE_IDENTIFIER, WHITE_TEXTURE_IDENTIFIER};
use crate::{
    bitmap_font::font_loader, font, texture, texture_atlas_loader, texture_loader, Active,
    AnimatedSprite, BitmapFont, Color, GBufferComponent, GraphicsError, Material,
    MaterialDescription, OrthographicCamera, PolygonMode, RectangleShape, Size2, Sprite,
    TextureAtlas, TextureData, TextureMetadata, TextureRegion, Tile, Tilemap, WGPUState, Window,
    WindowSize, DEFAULT_NORMAL_MAP_IDENTIFIER,
};
use nalgebra::{Matrix4, Vector3};
use std::any::TypeId;
use std::collections::HashMap;
use tuber_core::asset::{AssetStore, GenericLoader};
use tuber_core::transform::{IntoMatrix4, Transform2D};
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::Opt;
use tuber_ecs::query::accessors::R;
use tuber_ecs::{EntityIndex, Parent};

pub struct Graphics {
    wgpu_state: Option<WGPUState>,
    texture_metadata: HashMap<String, TextureMetadata>,
}

impl Graphics {
    pub fn new() -> Self {
        let texture_metadata = HashMap::new();
        Self {
            wgpu_state: None,
            texture_metadata,
        }
    }

    pub fn initialize(&mut self, window: Window, window_size: WindowSize) {
        self.wgpu_state = Some(WGPUState::new(window, window_size));
        self.load_texture_in_vram(&font::create_default_bitmap_font_texture());
        self.load_texture_in_vram(&texture::create_white_texture());
        self.load_texture_in_vram(&texture::create_placeholder_texture());
        self.load_texture_in_vram(&texture::create_normal_map_texture());
    }

    pub fn render(&mut self) {
        self.wgpu_state.as_mut().unwrap().render();
    }

    pub fn draw_ui_rectangle(
        &mut self,
        rectangle: &RectangleShape,
        transform_matrix: Matrix4<f32>,
    ) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::DrawUIQuad(DrawQuadCommand {
                quad: Quad {
                    top_left: Vertex {
                        position: [0.0, 0.0, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    bottom_left: Vertex {
                        position: [0.0, rectangle.height, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    top_right: Vertex {
                        position: [rectangle.width, 0.0, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    bottom_right: Vertex {
                        position: [rectangle.width, rectangle.height, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                },
                world_transform: transform_matrix,
                material: MaterialDescription {
                    albedo_map_id: self.texture_metadata[WHITE_TEXTURE_IDENTIFIER].texture_id,
                    normal_map_id: self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
                },
            }));
    }

    pub fn draw_rectangle(&mut self, rectangle: &RectangleShape, transform_matrix: Matrix4<f32>) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::DrawQuad(DrawQuadCommand {
                quad: Quad {
                    top_left: Vertex {
                        position: [0.0, 0.0, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    bottom_left: Vertex {
                        position: [0.0, rectangle.height, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    top_right: Vertex {
                        position: [rectangle.width, 0.0, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                    bottom_right: Vertex {
                        position: [rectangle.width, rectangle.height, 0.0],
                        texture_coordinates: [0.0, 0.0],
                        color: rectangle.color.into(),
                    },
                },
                world_transform: transform_matrix,
                material: MaterialDescription {
                    albedo_map_id: self.texture_metadata[WHITE_TEXTURE_IDENTIFIER].texture_id,
                    normal_map_id: self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
                },
            }));
    }

    pub fn draw_sprite(
        &mut self,
        sprite: &Sprite,
        transform_matrix: Matrix4<f32>,
        asset_manager: &mut AssetStore,
    ) -> Result<(), GraphicsError> {
        self.load_texture_in_vram_if_required(asset_manager, &sprite.material.albedo_map);

        let albedo_map_metadata = match self.texture_metadata.get(&sprite.material.albedo_map) {
            Some(albedo_map_medata) => albedo_map_medata,
            None => &self.texture_metadata[MISSING_TEXTURE_IDENTIFIER],
        };

        let normal_map_metadata = match &sprite.material.normal_map {
            Some(normal_map) => self.texture_metadata.get(normal_map),
            None => None,
        };

        let texture_region = sprite
            .texture_region
            .normalize(albedo_map_metadata.width, albedo_map_metadata.height);

        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::DrawQuad(DrawQuadCommand {
                quad: Quad {
                    top_left: Vertex {
                        position: [0.0, 0.0, 0.0],
                        texture_coordinates: [texture_region.x, texture_region.y],
                        ..Default::default()
                    },
                    bottom_left: Vertex {
                        position: [0.0, sprite.height, 0.0],
                        texture_coordinates: [
                            texture_region.x,
                            texture_region.y + texture_region.height,
                        ],
                        ..Default::default()
                    },
                    top_right: Vertex {
                        position: [sprite.width, 0.0, 0.0],
                        texture_coordinates: [
                            texture_region.x + texture_region.width,
                            texture_region.y,
                        ],
                        ..Default::default()
                    },
                    bottom_right: Vertex {
                        position: [sprite.width, sprite.height, 0.0],
                        texture_coordinates: [
                            texture_region.x + texture_region.width,
                            texture_region.y + texture_region.height,
                        ],
                        ..Default::default()
                    },
                },
                world_transform: transform_matrix,
                material: MaterialDescription {
                    albedo_map_id: albedo_map_metadata.texture_id,
                    normal_map_id: normal_map_metadata
                        .or(Some(&self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER]))
                        .unwrap()
                        .texture_id,
                },
            }));

        Ok(())
    }

    pub fn draw_animated_sprite(
        &mut self,
        animated_sprite: &AnimatedSprite,
        transform_matrix: Matrix4<f32>,
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

        let albedo_map_id = self.texture_metadata[&animated_sprite.material.albedo_map].texture_id;
        let normal_map_id = match &animated_sprite.material.normal_map {
            Some(normal_map) => self.texture_metadata[normal_map].texture_id,
            None => self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
        };

        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::DrawQuad(DrawQuadCommand {
                quad: Quad {
                    top_left: Vertex {
                        position: [0.0, 0.0, 0.0],
                        texture_coordinates: [
                            normalized_texture_region.x,
                            normalized_texture_region.y,
                        ],
                        ..Default::default()
                    },
                    bottom_left: Vertex {
                        position: [0.0, animated_sprite.height, 0.0],
                        texture_coordinates: [
                            normalized_texture_region.x,
                            normalized_texture_region.y + normalized_texture_region.height,
                        ],
                        ..Default::default()
                    },
                    top_right: Vertex {
                        position: [animated_sprite.width, 0.0, 0.0],
                        texture_coordinates: [
                            normalized_texture_region.x + normalized_texture_region.width,
                            normalized_texture_region.y,
                        ],
                        ..Default::default()
                    },
                    bottom_right: Vertex {
                        position: [animated_sprite.width, animated_sprite.height, 0.0],
                        texture_coordinates: [
                            normalized_texture_region.x + normalized_texture_region.width,
                            normalized_texture_region.y + normalized_texture_region.height,
                        ],
                        ..Default::default()
                    },
                },
                world_transform: transform_matrix,
                material: MaterialDescription {
                    albedo_map_id,
                    normal_map_id,
                },
            }));

        Ok(())
    }

    pub fn draw_tilemap(
        &mut self,
        asset_store: &mut AssetStore,
        tilemap: &mut Tilemap,
        transform_matrix: Matrix4<f32>,
    ) {
        if tilemap.render_id().is_none() {
            let tilemap_size = tilemap.size().clone();
            let tile_size = tilemap.tile_size().clone();

            let tilemap_size_pixel = Size2::new(
                tilemap_size.width as u32 * tile_size.width,
                tilemap_size.height as u32 * tile_size.height,
            );

            let render_id = self
                .wgpu_state
                .as_mut()
                .unwrap()
                .allocate_pre_render(tilemap_size_pixel);

            self.render_entire_tilemap(render_id, asset_store, tilemap, &tilemap_size, &tile_size);
            tilemap.set_render_id(render_id);
        } else if tilemap
            .tiles()
            .iter()
            .any(|tile| matches!(tile, &Some(Tile::AnimatedTile(_))))
        {
            self.rerender_animated_tiles(tilemap);
        }

        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::DrawPreRender(DrawPreRenderCommand {
                render_id: tilemap.render_id().unwrap(),
                size: Size2::new(
                    tilemap.size().width as f32 * tilemap.tile_size().width as f32,
                    tilemap.size().height as f32 * tilemap.tile_size().height as f32,
                ),
                world_transform: transform_matrix,
            }));
    }

    fn rerender_animated_tiles(&mut self, tilemap: &Tilemap) {
        let render_id = tilemap.render_id().unwrap();
        let tile_size = tilemap.tile_size();
        let tilemap_material = tilemap.material();
        let albedo_map_texture_metadata = &self.texture_metadata[&tilemap_material.albedo_map];
        let mut draw_quad_commands = vec![];
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
            let texture_region =
                &tile.animation_state.keyframes[tile.animation_state.current_keyframe].normalize(
                    albedo_map_texture_metadata.width,
                    albedo_map_texture_metadata.height,
                );

            draw_quad_commands.push(DrawQuadCommand {
                quad: Quad {
                    top_left: Vertex {
                        position: [0.0, 0.0, 0.0],
                        color: Default::default(),
                        texture_coordinates: [texture_region.x, texture_region.y],
                    },
                    bottom_left: Vertex {
                        position: [0.0, tile_size.height as f32, 0.0],
                        color: Default::default(),
                        texture_coordinates: [
                            texture_region.x,
                            texture_region.y + texture_region.height,
                        ],
                    },
                    top_right: Vertex {
                        position: [tile_size.width as f32, 0.0, 0.0],
                        color: Default::default(),
                        texture_coordinates: [
                            texture_region.x + texture_region.width,
                            texture_region.y,
                        ],
                    },
                    bottom_right: Vertex {
                        position: [tile_size.width as f32, tile_size.height as f32, 0.0],
                        color: Default::default(),
                        texture_coordinates: [
                            texture_region.x + texture_region.width,
                            texture_region.y + texture_region.height,
                        ],
                    },
                },
                world_transform: Matrix4::identity().append_translation(&Vector3::new(
                    (tile_x * tilemap.tile_size().width as usize) as f32,
                    (tile_y * tilemap.tile_size().height as usize) as f32,
                    0.0,
                )),
                material: MaterialDescription {
                    albedo_map_id: albedo_map_texture_metadata.texture_id,
                    normal_map_id: match &tilemap_material.normal_map {
                        Some(normal_map_identifier) => {
                            self.texture_metadata[normal_map_identifier].texture_id
                        }
                        None => self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
                    },
                },
            });
        }

        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::PreDrawQuads(PreDrawQuadsCommand {
                render_id,
                draw_quad_commands,
            }));
    }

    fn render_entire_tilemap(
        &mut self,
        render_id: RenderId,
        asset_store: &mut AssetStore,
        tilemap: &mut Tilemap,
        tilemap_size: &Size2<usize>,
        tile_size: &Size2<u32>,
    ) {
        let tilemap_material = tilemap.material();
        self.load_material_in_vram_if_required(asset_store, tilemap_material);

        let albedo_map_texture_metadata = &self.texture_metadata[&tilemap_material.albedo_map];

        let mut draw_quad_commands = vec![];
        for (tile_index, tile) in tilemap.tiles().iter().enumerate() {
            let tile_x = tile_index % tilemap_size.width;
            let tile_y = tile_index / tilemap_size.height;
            if let Some(tile) = tile {
                let texture_region = match tile {
                    Tile::StaticTile(static_tile) => &static_tile.texture_region,
                    Tile::AnimatedTile(animated_tile) => {
                        &animated_tile.animation_state.keyframes
                            [animated_tile.animation_state.current_keyframe]
                    }
                };

                let texture_region = texture_region.normalize(
                    albedo_map_texture_metadata.width,
                    albedo_map_texture_metadata.height,
                );

                draw_quad_commands.push(DrawQuadCommand {
                    quad: Quad {
                        top_left: Vertex {
                            position: [0.0, 0.0, 0.0],
                            color: Default::default(),
                            texture_coordinates: [texture_region.x, texture_region.y],
                        },
                        bottom_left: Vertex {
                            position: [0.0, tile_size.height as f32, 0.0],
                            color: Default::default(),
                            texture_coordinates: [
                                texture_region.x,
                                texture_region.y + texture_region.height,
                            ],
                        },
                        top_right: Vertex {
                            position: [tile_size.width as f32, 0.0, 0.0],
                            color: Default::default(),
                            texture_coordinates: [
                                texture_region.x + texture_region.width,
                                texture_region.y,
                            ],
                        },
                        bottom_right: Vertex {
                            position: [tile_size.width as f32, tile_size.height as f32, 0.0],
                            color: Default::default(),
                            texture_coordinates: [
                                texture_region.x + texture_region.width,
                                texture_region.y + texture_region.height,
                            ],
                        },
                    },
                    world_transform: Matrix4::identity().append_translation(&Vector3::new(
                        (tile_x * tilemap.tile_size().width as usize) as f32,
                        (tile_y * tilemap.tile_size().height as usize) as f32,
                        0.0,
                    )),
                    material: MaterialDescription {
                        albedo_map_id: albedo_map_texture_metadata.texture_id,
                        normal_map_id: match &tilemap_material.normal_map {
                            Some(normal_map_identifier) => {
                                self.texture_metadata[normal_map_identifier].texture_id
                            }
                            None => self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER].texture_id,
                        },
                    },
                });
            }
        }

        self.wgpu_state
            .as_mut()
            .unwrap()
            .command_buffer_mut()
            .add(Command::PreDrawQuads(PreDrawQuadsCommand {
                render_id,
                draw_quad_commands,
            }));
    }

    pub fn draw_text(
        &mut self,
        text: &str,
        font_identifier: &Option<String>,
        transform_matrix: Matrix4<f32>,
        asset_store: &mut AssetStore,
    ) {
        {
            if let Some(font_identifier) = font_identifier {
                let _ = asset_store.load::<BitmapFont>(font_identifier);
            }
        }

        let default_bitmap_font = font::default_bitmap_font();
        let font = match font_identifier {
            Some(font_identifier) => asset_store
                .stored_asset::<BitmapFont>(dbg!(font_identifier))
                .unwrap(),
            None => &default_bitmap_font,
        };

        let font_texture_atlas = match font_identifier {
            Some(_) => Some(
                asset_store
                    .stored_asset::<TextureAtlas>(font.font_atlas().unwrap())
                    .unwrap(),
            ),
            None => None,
        };

        let font_region = match font_identifier {
            Some(font_identifier) => font_texture_atlas
                .unwrap()
                .texture_region(font_identifier)
                .expect("Font region not found"),
            None => TextureRegion::new(0.0, 0.0, 128.0, 32.0),
        };

        let font_texture = match font_identifier {
            Some(_) => font.font_atlas_texture().unwrap(),
            None => DEFAULT_FONT_IDENTIFIER,
        };

        let texture = &self.texture_metadata[font_texture];

        let transform_matrix = transform_matrix;
        let mut offset_x = transform_matrix.column(3).x;
        let mut offset_y = transform_matrix.column(3).y;
        for character in text.chars() {
            if character == '\n' {
                offset_y += (font.line_height() + font.line_spacing()) as f32;
                offset_x = transform_matrix.column(3).x;
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

            let mut glyph_transform_matrix = Matrix4::identity();
            glyph_transform_matrix.append_translation_mut(&Vector3::new(offset_x, offset_y, 0.0));

            let glyph_texture_coordinates = TextureRegion {
                x: font_region.x + glyph_region.x,
                y: font_region.y + glyph_region.y,
                width: glyph_region.width,
                height: glyph_region.height,
            }
            .normalize(texture.width, texture.height);

            self.wgpu_state
                .as_mut()
                .unwrap()
                .command_buffer_mut()
                .add(Command::DrawUIQuad(DrawQuadCommand {
                    quad: Quad {
                        top_left: Vertex {
                            position: [0.0, 0.0, 0.0],
                            color: Color::WHITE.into(),
                            texture_coordinates: [
                                glyph_texture_coordinates.x,
                                glyph_texture_coordinates.y,
                            ],
                        },
                        bottom_left: Vertex {
                            position: [0.0, glyph_region.height, 0.0],
                            color: Color::WHITE.into(),
                            texture_coordinates: [
                                glyph_texture_coordinates.x,
                                glyph_texture_coordinates.y + glyph_texture_coordinates.height,
                            ],
                        },
                        top_right: Vertex {
                            position: [glyph_region.width, 0.0, 0.0],
                            color: Color::WHITE.into(),
                            texture_coordinates: [
                                glyph_texture_coordinates.x + glyph_texture_coordinates.width,
                                glyph_texture_coordinates.y,
                            ],
                        },
                        bottom_right: Vertex {
                            position: [glyph_region.width, glyph_region.height, 0.0],
                            color: Color::WHITE.into(),
                            texture_coordinates: [
                                glyph_texture_coordinates.x + glyph_texture_coordinates.width,
                                glyph_texture_coordinates.y + glyph_texture_coordinates.height,
                            ],
                        },
                    },
                    world_transform: glyph_transform_matrix,
                    material: MaterialDescription {
                        albedo_map_id: self.texture_metadata[font_texture].texture_id,
                        normal_map_id: self.texture_metadata[DEFAULT_NORMAL_MAP_IDENTIFIER]
                            .texture_id,
                    },
                }));

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
        let texture_id = self
            .wgpu_state
            .as_mut()
            .unwrap()
            .load_texture_in_vram(texture);

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
        self.update_camera(camera_id, &camera, camera_transform.into_matrix4());

        for (_, (rectangle_shape, transform, parent)) in
            ecs.query::<(R<RectangleShape>, R<Transform2D>, Opt<R<Parent>>)>()
        {
            let mut parent_transform = Matrix4::<f32>::identity();
            let mut parent = parent;
            while let Some(parent_ref) = &parent {
                let parent_id = parent_ref.0;
                let (_, (transform, p)) = ecs
                    .query_one_by_id::<(R<Transform2D>, Opt<R<Parent>>)>(parent_id)
                    .unwrap();
                parent_transform *= transform.into_matrix4();
                parent = p;
            }

            self.draw_rectangle(
                &rectangle_shape,
                parent_transform * transform.into_matrix4(),
            );
        }
        for (_, (sprite, transform, parent)) in
            ecs.query::<(R<Sprite>, R<Transform2D>, Opt<R<Parent>>)>()
        {
            let mut parent_transform = Matrix4::<f32>::identity();
            let mut parent = parent;
            while let Some(parent_ref) = &parent {
                let parent_id = parent_ref.0;
                let (_, (transform, p)) = ecs
                    .query_one_by_id::<(R<Transform2D>, Opt<R<Parent>>)>(parent_id)
                    .unwrap();
                parent_transform *= transform.into_matrix4();
                parent = p;
            }

            self.draw_sprite(
                &sprite,
                parent_transform * transform.into_matrix4(),
                asset_store,
            )
            .unwrap();
        }
        for (_, (animated_sprite, transform, parent)) in
            ecs.query::<(R<AnimatedSprite>, R<Transform2D>, Opt<R<Parent>>)>()
        {
            let mut parent_transform = Matrix4::<f32>::identity();
            let mut parent = parent;
            while let Some(parent_ref) = &parent {
                let parent_id = parent_ref.0;
                let (_, (transform, p)) = ecs
                    .query_one_by_id::<(R<Transform2D>, Opt<R<Parent>>)>(parent_id)
                    .unwrap();
                parent_transform *= transform.into_matrix4();
                parent = p;
            }

            self.draw_animated_sprite(
                &animated_sprite,
                parent_transform * transform.into_matrix4(),
                asset_store,
            )
            .unwrap();
        }

        self.render();
    }

    pub fn update_camera(
        &mut self,
        camera_id: EntityIndex,
        camera: &OrthographicCamera,
        transform_matrix: Matrix4<f32>,
    ) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .update_camera(camera_id, camera, transform_matrix);
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .set_clear_color(clear_color);
    }

    pub fn set_rendered_g_buffer_component(&mut self, g_buffer_component: GBufferComponent) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .set_rendered_g_buffer_component(g_buffer_component);
    }

    pub fn set_polygon_mode(&mut self, polygon_mode: PolygonMode) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .set_polygon_mode(polygon_mode);
    }

    pub fn on_window_resized(&mut self, width: u32, height: u32) {
        self.wgpu_state
            .as_mut()
            .unwrap()
            .resize(Size2::from((width, height)));
    }

    pub fn loaders() -> Vec<(TypeId, GenericLoader)> {
        vec![
            (TypeId::of::<TextureData>(), Box::new(texture_loader)),
            (TypeId::of::<TextureAtlas>(), Box::new(texture_atlas_loader)),
            (TypeId::of::<BitmapFont>(), Box::new(font_loader)),
        ]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RenderId(pub usize);

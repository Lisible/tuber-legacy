use std::collections::HashMap;

use image::GenericImageView;
use log::info;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Origin3d, Queue, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TextureAspect, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDescriptor, TextureViewDimension,
};

pub struct TextureStore {
    texture_bind_group_layout: BindGroupLayout,
    texture_bind_groups: HashMap<String, BindGroup>,
}

impl TextureStore {
    pub fn new(device: &Device) -> Self {
        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D2,
                            sample_type: TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        Self {
            texture_bind_group_layout,
            texture_bind_groups: HashMap::new(),
        }
    }

    pub fn load_texture_from_image_data(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture_identifier: &str,
        image_data: &[u8],
    ) {
        let image = image::load_from_memory(image_data).unwrap();
        let dimensions = image.dimensions();
        let texture_data = image.as_rgba8().unwrap();
        self.load_texture(
            device,
            queue,
            texture_identifier,
            texture_data.as_ref(),
            dimensions.0,
            dimensions.1,
        )
    }

    pub fn load_texture(
        &mut self,
        device: &Device,
        queue: &Queue,
        texture_identifier: &str,
        texture_data: &[u8],
        texture_width: u32,
        texture_height: u32,
    ) {
        info!(
            "Loading texture \"{}\" from RGBA8 data into V-RAM",
            texture_identifier
        );
        let texture_rgba = texture_data;
        let texture_size = Extent3d {
            width: texture_width,
            height: texture_height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            texture_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * texture_width),
                rows_per_image: std::num::NonZeroU32::new(texture_height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &self.texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&texture_sampler),
                },
            ],
        });

        self.texture_bind_groups
            .insert(texture_identifier.into(), texture_bind_group);
    }

    pub fn texture_bind_group_layout(&self) -> &BindGroupLayout {
        &self.texture_bind_group_layout
    }

    pub fn texture_bind_group(&self, texture_identifier: &str) -> Option<&BindGroup> {
        self.texture_bind_groups.get(texture_identifier)
    }
}

use wgpu::{AddressMode, Device, FilterMode, Sampler};

use crate::low_level::primitives::TextureId;
use crate::texture::{TextureData, TextureSize};
use crate::types::Size2;

const BYTES_PER_PIXEL: usize = 4;

pub(crate) fn create_texture_from_data(
    device: &Device,
    queue: &wgpu::Queue,
    texture_id: TextureId,
    texture_data: &TextureData,
) -> wgpu::Texture {
    create_texture(
        device,
        queue,
        texture_id,
        texture_data.size,
        &texture_data.bytes,
        texture_data.srgb,
    )
}

fn create_texture(
    device: &Device,
    queue: &wgpu::Queue,
    texture_id: TextureId,
    size: TextureSize,
    data: &[u8],
    srgb: bool,
) -> wgpu::Texture {
    let texture_label = create_wgpu_texture_label(texture_id);
    let texture_size = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&texture_label),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: if srgb {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        },
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: std::num::NonZeroU32::new(BYTES_PER_PIXEL as u32 * size.0),
            rows_per_image: std::num::NonZeroU32::new(size.1),
        },
        texture_size,
    );

    texture
}

pub fn create_texture_descriptor(
    label: Option<&'static str>,
    size: Size2<u32>,
    texture_format: wgpu::TextureFormat,
) -> wgpu::TextureDescriptor {
    wgpu::TextureDescriptor {
        label,
        size: wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
    }
}

pub fn create_g_buffer_texture_descriptor(
    label: &'static str,
    size: Size2<u32>,
) -> wgpu::TextureDescriptor {
    create_texture_descriptor(Some(label), size, wgpu::TextureFormat::Bgra8UnormSrgb)
}

pub fn create_default_sampler(device: &Device) -> Sampler {
    create_sampler(
        device,
        AddressMode::ClampToEdge,
        FilterMode::Nearest,
        FilterMode::Nearest,
        FilterMode::Nearest,
    )
}

pub fn create_sampler(
    device: &Device,
    address_mode: AddressMode,
    min_filter: FilterMode,
    mag_filter: FilterMode,
    mipmap_filter: FilterMode,
) -> Sampler {
    device.create_sampler(&wgpu::SamplerDescriptor {
        label: None,
        address_mode_u: address_mode,
        address_mode_v: address_mode,
        address_mode_w: address_mode,
        mag_filter,
        min_filter,
        mipmap_filter,
        ..Default::default()
    })
}

fn create_wgpu_texture_label(texture_id: TextureId) -> String {
    "wgputexture_".to_owned() + &*texture_id.to_string()
}

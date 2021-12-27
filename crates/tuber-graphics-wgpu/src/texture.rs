use tuber_graphics::texture::{TextureData, TextureSize};
use wgpu::TextureDescriptor;

const BYTES_PER_PIXEL: usize = 4;

pub struct Texture {
    size: wgpu::Extent3d,
    handle: wgpu::Texture,
}

pub(crate) fn create_texture_from_data(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture_data: &TextureData,
) -> wgpu::Texture {
    create_texture(
        device,
        queue,
        &texture_data.identifier,
        texture_data.size,
        &texture_data.bytes,
    )
}

fn create_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    identifier: &str,
    size: TextureSize,
    data: &[u8],
) -> wgpu::Texture {
    let texture_identifier = create_wgpu_texture_identifier(identifier);
    let texture_size = wgpu::Extent3d {
        width: size.0,
        height: size.1,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&TextureDescriptor {
        label: Some(&texture_identifier),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
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

fn create_wgpu_texture_identifier(texture_identifier: &str) -> String {
    "wgputexture_".to_owned() + texture_identifier
}

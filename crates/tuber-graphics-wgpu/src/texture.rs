use wgpu::TextureDescriptor;
use tuber_graphics::texture::TextureData;

pub struct Texture {
    size: wgpu::Extent3d,
    handle: wgpu::Texture,
}

pub(crate) fn create_texture_from_data(device: &wgpu::Device, texture_data: TextureData) -> wgpu::Texture {
    let texture_identifier = "texture_".to_owned() + &*texture_data.identifier;
    let texture = device.create_texture(&TextureDescriptor {
        label: Some(&texture_identifier),
        size: wgpu::Extent3d {
            width: texture_data.size.0,
            height: texture_data.size.1,
            depth_or_array_layers: 1
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING  | wgpu::TextureUsages::COPY_DST
    });
    texture.
}

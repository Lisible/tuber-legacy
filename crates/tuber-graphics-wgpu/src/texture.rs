use tuber_graphics::texture::{TextureData, TextureSize};

const BYTES_PER_PIXEL: usize = 4;

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

    let texture = device.create_texture(&wgpu::TextureDescriptor {
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

pub fn create_texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("texture_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler {
                    filtering: true,
                    comparison: false,
                },
                count: None,
            },
        ],
    })
}

pub fn create_texture_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    texture_sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(texture_sampler),
            },
        ],
    })
}

fn create_wgpu_texture_identifier(texture_identifier: &str) -> String {
    "wgputexture_".to_owned() + texture_identifier
}

use std::marker::PhantomData;

use crate::GraphicsResult;
use crate::textures::*;
use crate::wgpu::*;

#[derive(Default)]
pub struct Resources {
    textures: Vec<Texture>,
}

impl Resources {
    fn create_texture(&mut self, device: &WGPUDevice, size: TextureSize, format: TextureFormat) -> GraphicsResult<Handle<Texture>> {
        let wgpu_texture = device.create_texture(&WGPUTextureDescriptor {
            label: None,
            size: size.into(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: WGPUTextureDimension::D2,
            format: format.into(),
            usage: (TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST),
        });

        self.textures.push(Texture::new(wgpu_texture));

        Ok(Handle::<Texture>::new(self.textures.len() - 1))
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Handle<T> {
    id: usize,
    marker: PhantomData<T>,
}

impl<T> Handle<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
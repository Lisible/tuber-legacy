use std::marker::PhantomData;

use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::Device;
use wgpu::Queue;

use crate::low_level::utils::create_uniform_buffer;

pub struct UniformBuffer<T> {
    label: String,
    buffer: Buffer,
    count: usize,
    capacity: usize,
    _phantom: PhantomData<T>,
}

impl<T> UniformBuffer<T> {
    /// Creates a new uniform buffer with a given capacity
    pub fn new(device: &Device, label: &str, capacity: usize) -> Self {
        let buffer = create_uniform_buffer(
            device,
            label,
            (capacity * std::mem::size_of::<T>()) as BufferAddress,
        );

        Self {
            label: label.to_string(),
            buffer,
            count: 0,
            capacity,
            _phantom: PhantomData,
        }
    }

    /// Appends uniform variables to the buffer
    ///
    /// # Panics
    ///
    /// Panics if the buffer doesn't have enough capacity
    pub fn append_uniforms(&mut self, queue: &Queue, uniforms: &[T])
    where
        T: bytemuck::Pod,
    {
        assert!(
            self.count < self.capacity,
            "UniformBuffer label=`{}` is filled",
            self.label
        );
        assert!(
            self.count + uniforms.len() <= self.capacity,
            "UniformBuffer label=`{}` would overflow",
            self.label
        );

        queue.write_buffer(
            &self.buffer,
            (self.count * std::mem::size_of::<T>()) as BufferAddress,
            bytemuck::cast_slice(uniforms),
        );

        self.count += uniforms.len();
    }

    /// Clears the buffer
    pub fn clear(&mut self) {
        self.count = 0;
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

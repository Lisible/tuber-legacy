use std::cmp::max;
use std::ops::RangeBounds;

use wgpu::{Buffer, BufferAddress, BufferSlice, CommandEncoder, Device, Queue};

use crate::low_level::primitives::Index;
use crate::low_level::utils::create_index_buffer;

pub struct IndexBuffer {
    label: String,
    buffer: Buffer,
    count: usize,
    capacity: usize,
}

impl IndexBuffer {
    pub fn with_capacity(device: &Device, label: &str, capacity: usize) -> Self {
        let buffer = create_index_buffer(device, label, capacity);

        Self {
            label: label.to_string(),
            buffer,
            count: 0,
            capacity,
        }
    }

    pub fn append_indices(
        &mut self,
        command_encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        indices: &[Index],
        index_count: usize,
    ) {
        self.ensure_capacity(device, command_encoder, self.count + indices.len());

        queue.write_buffer(
            &self.buffer,
            (self.count * std::mem::size_of::<Index>()) as BufferAddress,
            bytemuck::cast_slice(&indices),
        );

        self.count += index_count;
    }

    pub fn slice(&self, range: impl RangeBounds<BufferAddress>) -> BufferSlice {
        self.buffer.slice(range)
    }

    pub fn ensure_capacity(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        target_capacity: usize,
    ) {
        if self.capacity >= target_capacity {
            return;
        }

        let new_capacity = max(self.capacity * 2, target_capacity);
        let new_buffer = create_index_buffer(device, &self.label, new_capacity);

        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            (self.count * std::mem::size_of::<Index>()) as BufferAddress,
        );

        self.capacity = new_capacity;
        self.buffer = new_buffer;
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

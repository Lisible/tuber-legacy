use crate::geometry::Vertex;
use crate::low_level::utils::create_vertex_buffer;
use std::cmp::max;
use std::ops::RangeBounds;
use wgpu::{Buffer, BufferAddress, BufferSlice, CommandEncoder, Device, Queue};

pub struct VertexBuffer {
    label: String,
    buffer: Buffer,
    count: usize,
    capacity: usize,
}

impl VertexBuffer {
    pub fn with_capacity(device: &Device, label: &str, capacity: usize) -> Self {
        let buffer = create_vertex_buffer(device, label, capacity);

        Self {
            label: label.to_string(),
            buffer,
            count: 0,
            capacity,
        }
    }

    pub fn append_vertices(
        &mut self,
        command_encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        vertices: &[Vertex],
    ) {
        self.ensure_capacity(device, command_encoder, self.count + vertices.len());

        queue.write_buffer(
            &self.buffer,
            (self.count * std::mem::size_of::<Vertex>()) as BufferAddress,
            bytemuck::cast_slice(vertices),
        );

        self.count += vertices.len();
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
        let new_buffer = create_vertex_buffer(device, &self.label, new_capacity);

        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            (self.capacity * std::mem::size_of::<Vertex>()) as BufferAddress,
        );

        self.capacity = new_capacity;
        self.buffer = new_buffer;
    }

    pub fn clear(&mut self) {
        self.count = 0;
    }
}

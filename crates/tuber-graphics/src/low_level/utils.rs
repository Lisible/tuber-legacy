use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device, Label};

use crate::geometry::Vertex;
use crate::primitives::Index;

pub fn create_uniform_buffer(device: &Device, label: &str, size: BufferAddress) -> Buffer {
    create_copyable_buffer(device, label, size, BufferUsages::UNIFORM)
}

pub fn create_vertex_buffer(device: &Device, label: &str, capacity: usize) -> Buffer {
    create_copyable_buffer(
        device,
        label,
        (capacity * std::mem::size_of::<Vertex>()) as BufferAddress,
        BufferUsages::VERTEX,
    )
}

pub fn create_index_buffer(device: &Device, label: &str, capacity: usize) -> Buffer {
    create_copyable_buffer(
        device,
        label,
        (capacity * std::mem::size_of::<Index>()) as BufferAddress,
        BufferUsages::INDEX,
    )
}

pub fn create_copyable_buffer(
    device: &Device,
    label: &str,
    size: BufferAddress,
    usage: BufferUsages,
) -> Buffer {
    create_buffer(
        device,
        Some(label),
        size,
        usage | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
    )
}

fn create_buffer(
    device: &Device,
    label: Label,
    size: BufferAddress,
    usage: BufferUsages,
) -> Buffer {
    device.create_buffer(&BufferDescriptor {
        label,
        size,
        usage,
        mapped_at_creation: false,
    })
}

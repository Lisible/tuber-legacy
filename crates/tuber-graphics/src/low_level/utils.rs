use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupLayout, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferAddress, BufferDescriptor, BufferUsages, Device, Label,
};

pub fn create_uniform_bind_group<UniformType>(
    device: &Device,
    label: &str,
    layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    device.create_bind_group(&BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: 0,
                size: wgpu::BufferSize::new(std::mem::size_of::<UniformType>() as u64),
            }),
        }],
    })
}

pub fn create_global_uniform_bind_group_layout(device: &Device, label: &str) -> BindGroupLayout {
    device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some(label),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

pub fn create_global_uniform_bind_group(
    device: &Device,
    label: &str,
    bind_group_layout: &BindGroupLayout,
    buffer: &Buffer,
) -> BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}

pub fn create_global_uniform_buffer<GlobalUniformType>(
    device: &Device,
    label: &str,
    global_uniform: GlobalUniformType,
) -> Buffer
where
    GlobalUniformType: bytemuck::Pod + bytemuck::Zeroable,
{
    device.create_buffer_init(&BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(&[global_uniform]),
        usage: BufferUsages::UNIFORM,
    })
}

pub fn create_uniform_buffer(device: &Device, label: &str, size: BufferAddress) -> Buffer {
    create_copyable_buffer(device, label, size, wgpu::BufferUsages::UNIFORM)
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

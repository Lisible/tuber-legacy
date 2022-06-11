use std::cmp::max;
use std::marker::PhantomData;

use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry,
    BindingResource, BindingType, Buffer, BufferAddress, BufferBinding, BufferBindingType,
    BufferSize, CommandEncoder, Device, Queue, ShaderStages,
};

use crate::low_level::utils::create_uniform_buffer;

const BUFFER_SUFFIX: &str = "_buffer";
const BIND_GROUP_SUFFIX: &str = "_bind_group";
const BIND_GROUP_LAYOUT_SUFFIX: &str = "_bind_group_layout";

pub struct UniformBuffer<UniformType> {
    label: String,
    buffer: Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
    uniform_count: usize,
    capacity: usize,
    uniform_offset: usize,
    _phantom: PhantomData<UniformType>,
}

impl<UniformType> UniformBuffer<UniformType>
where
    UniformType: bytemuck::Pod,
{
    pub fn new(device: &Device, label: &str, default_capacity: usize) -> Self {
        let uniform_offset = device.limits().min_uniform_buffer_offset_alignment as usize;
        let buffer = Self::create_buffer(
            device,
            &format!("{}{}", label, BUFFER_SUFFIX),
            default_capacity,
            uniform_offset,
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(&format!("{}{}", label, BIND_GROUP_LAYOUT_SUFFIX)),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(uniform_offset as BufferAddress),
                },
                count: None,
            }],
        });

        let bind_group =
            Self::create_bind_group(device, label, uniform_offset, &buffer, &bind_group_layout);

        Self {
            label: label.to_string(),
            buffer,
            bind_group,
            bind_group_layout,
            uniform_count: 0,
            capacity: default_capacity,
            uniform_offset,
            _phantom: PhantomData,
        }
    }

    fn create_bind_group(
        device: &Device,
        label: &str,
        uniform_offset: usize,
        buffer: &Buffer,
        bind_group_layout: &BindGroupLayout,
    ) -> BindGroup {
        device.create_bind_group(&BindGroupDescriptor {
            label: Some(&format!("{}{}", label, BIND_GROUP_SUFFIX)),
            layout: bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(BufferBinding {
                    buffer,
                    offset: 0,
                    size: BufferSize::new(uniform_offset as BufferAddress),
                }),
            }],
        })
    }

    pub fn append_uniforms(
        &mut self,
        command_encoder: &mut CommandEncoder,
        device: &Device,
        queue: &Queue,
        uniforms: &[UniformType],
    ) {
        let uniform_buffer_offset = self.uniform_count * self.uniform_offset;
        self.ensure_capacity(device, command_encoder, self.uniform_count + uniforms.len());

        queue.write_buffer(
            &self.buffer,
            uniform_buffer_offset as BufferAddress,
            bytemuck::cast_slice(uniforms),
        );

        self.uniform_count += uniforms.len();
    }

    pub fn clear(&mut self) {
        self.uniform_count = 0;
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn ensure_capacity(
        &mut self,
        device: &Device,
        command_encoder: &mut CommandEncoder,
        target_capacity: usize,
    ) {
        if self.capacity > target_capacity {
            return;
        }

        let new_capacity = max(self.capacity * 2, target_capacity);
        let new_buffer =
            Self::create_buffer(device, &self.label, new_capacity, self.uniform_offset);

        command_encoder.copy_buffer_to_buffer(
            &self.buffer,
            0,
            &new_buffer,
            0,
            (self.capacity * self.uniform_offset) as BufferAddress,
        );

        self.capacity = new_capacity;
        self.buffer = new_buffer;
        self.bind_group = Self::create_bind_group(
            device,
            &self.label,
            self.uniform_offset,
            &self.buffer,
            &self.bind_group_layout,
        );
    }

    pub fn count(&self) -> usize {
        self.uniform_count
    }

    pub fn current_offset(&self) -> usize {
        self.count() * self.uniform_offset
    }

    fn create_buffer(
        device: &Device,
        label: &str,
        capacity: usize,
        uniform_offset: usize,
    ) -> Buffer {
        create_uniform_buffer(
            device,
            &format!("{}{}", label, BUFFER_SUFFIX),
            capacity as BufferAddress * uniform_offset as BufferAddress,
        )
    }
}

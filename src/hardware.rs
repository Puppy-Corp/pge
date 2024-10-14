use std::ops::Range;
use std::sync::Arc;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> Buffer;
}

#[derive(Debug)]
pub struct Buffer {
    buffer: wgpu::Buffer,
    queue: Arc<wgpu::Queue>,
    bind_group: wgpu::BindGroup,
}

impl Buffer {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn slice(&self, range: Range<u64>) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(range)
    }

    pub fn full(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(..)
    }

    pub fn write(&self, data: &[u8]) {
        self.queue.write_buffer(&self.buffer, 0, data);
    }
}

pub struct WgpuHardware {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
}

impl WgpuHardware {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
        }
    }
}

impl Hardware for WgpuHardware {
    fn create_buffer(&mut self, name: &str) -> Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(name),
            size: 10_000_000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: Some("Buffer Bind Group"),
        });
        Buffer {
            buffer,
            queue: self.queue.clone(),
            bind_group,
        }
    }
}



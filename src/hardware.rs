use std::ops::Range;
use std::sync::Arc;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> Buffer;
}

#[derive(Debug)]
pub struct Buffer {
    buffer: wgpu::Buffer,
    queue: Arc<wgpu::Queue>,
}

impl Buffer {
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
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDEX,
            mapped_at_creation: false,
        });
        Buffer {
            buffer,
            queue: self.queue.clone(),
        }
    }
}



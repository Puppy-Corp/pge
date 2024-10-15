use std::ops::Range;
use std::sync::Arc;

use crate::wgpu_types::TextureBuffer;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> Buffer;
    fn create_texture(&mut self, name: &str, data: &[u8]) -> Texture;
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

#[derive(Debug)]
pub struct Texture {
    texture: wgpu::Texture,
    queue: Arc<wgpu::Queue>,
    bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
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
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
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

    fn create_texture(&mut self, name: &str, data: &[u8]) -> Texture {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: Default::default(),
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            size
        );
    
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
    
        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &TextureBuffer::create_bind_group_layout(&self.device),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });
        Texture {
            texture,
            queue: self.queue.clone(),
            bind_group: texture_bind_group,
        }
    }
}



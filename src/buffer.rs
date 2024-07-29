use std::sync::Arc;

use wgpu::BufferUsages;

use crate::wgpu_types::BufferRecipe;


pub struct Bindablebuffer {
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub size: u64,
}

impl Bindablebuffer {
	pub fn new<T: BufferRecipe>(device: &wgpu::Device, size: u64) -> Bindablebuffer {
		let layout = T::create_bind_group_layout(&device);
		let buffer = T::create_buffer(&device, size);
		let bind_group = T::create_bind_group(&device, &buffer, &layout);

		Bindablebuffer {
			buffer,
			bind_group,
			size: size,
		}
	}
}

pub struct Buffer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    buffer: Option<wgpu::Buffer>,
	bind_group: Option<wgpu::BindGroup>,
	bind_group_layout: Option<wgpu::BindGroupLayout>,
    size: u64,
	pub has_bind_group: bool,
}

impl Buffer {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Buffer {
        Buffer {
            device,
            queue,
            buffer: None,
            size: 0,
			bind_group: None,
			bind_group_layout: None,
			has_bind_group: false,
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        if data.len() as u64 > self.size {
            self.resize_and_copy(data.len() as u64);
        }

        if let Some(buffer) = &self.buffer {
            self.queue.write_buffer(buffer, 0, data);
        } else {
            panic!("this should not happen");
        }
    }

    fn resize_and_copy(&mut self, new_size: u64) {
        let new_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            size: new_size,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            label: None,
            mapped_at_creation: false,
        });

        if let Some(old_buffer) = &self.buffer {
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("resize_and_copy_encoder"),
            });

            encoder.copy_buffer_to_buffer(old_buffer, 0, &new_buffer, 0, self.size);

            let command_buffer = encoder.finish();
            self.queue.submit(Some(command_buffer));
        }

        self.buffer = Some(new_buffer);
        self.size = new_size;
		
		if self.has_bind_group {
			self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
				layout: self.bind_group_layout.as_ref().unwrap(),
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { 
							buffer: self.buffer.as_ref().unwrap(), 
							offset: 0, 
							size: None 
						})
					}
				],
				label: None,
			}));
		} else {
			self.bind_group = None;
			self.bind_group_layout = None;
		}
    }

    pub fn get_size(&self) -> u64 {
        self.size
    }

	pub fn bind_group(&self) -> &wgpu::BindGroup {
		self.bind_group.as_ref().unwrap()
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		self.buffer.as_ref().unwrap()
	}
}

#[derive(Debug, Clone, Default)]
pub struct DirtyBuffer {
	pub name: String,
    pub data: Vec<u8>,
    pub dirty: bool,
    pub offset: usize,
}

impl DirtyBuffer {
    pub fn new(name: &str) -> Self {
        Self {
			name: name.to_string(),
            data: Vec::new(),
            dirty: false,
            offset: 0,
        }
    }

    // pub fn clear(&mut self) {
    //     self.data.clear();
    //     self.dirty = true;
    //     self.offset = 0;
    // }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        if self.offset + slice.len() > self.data.len() {
			log::info!("[{}] data is bigger offset: {} slice.len: {} data.len: {}", self.name, self.offset, slice.len(), self.data.len());
            self.data.resize(self.offset + slice.len(), 0);
            self.dirty = true;
        }

        let data_changed = &self.data[self.offset..self.offset + slice.len()] != slice;
        if data_changed {
            self.data[self.offset..self.offset + slice.len()].copy_from_slice(slice);
            self.dirty = true;
        }
        self.offset += slice.len();
    }

    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buffer = DirtyBuffer::new();
        assert_eq!(buffer.data, Vec::<u8>::new());
        assert_eq!(buffer.dirty, false);
        assert_eq!(buffer.offset, 0);
    }

    #[test]
    fn test_clear_buffer() {
        let mut buffer = DirtyBuffer::new();
        buffer.extend_from_slice(&[1, 2, 3]);
        buffer.clear();
        assert_eq!(buffer.data, Vec::<u8>::new());
        assert_eq!(buffer.dirty, true);
        assert_eq!(buffer.offset, 0);
    }

    #[test]
    fn test_extend_from_slice() {
        let mut buffer = DirtyBuffer::new();
        buffer.extend_from_slice(&[1, 2, 3]);
        assert_eq!(buffer.data, vec![1, 2, 3]);
        assert_eq!(buffer.dirty, true);
        assert_eq!(buffer.offset, 3);
    }

    #[test]
    fn test_extend_from_slice_with_offset() {
        let mut buffer = DirtyBuffer::new();
        buffer.extend_from_slice(&[1, 2, 3]);
        buffer.reset_offset(1);
        buffer.extend_from_slice(&[4, 5]);
        assert_eq!(buffer.data, vec![1, 4, 5]);
        assert_eq!(buffer.dirty, true);
        assert_eq!(buffer.offset, 3);
    }

    #[test]
    fn test_extend_from_slice_no_change() {
        let mut buffer = DirtyBuffer::new();
        buffer.extend_from_slice(&[1, 2, 3]);
        buffer.reset_offset(0);
        buffer.dirty = false;
        buffer.extend_from_slice(&[1, 2, 3]);
        assert_eq!(buffer.data, vec![1, 2, 3]);
        assert_eq!(buffer.dirty, false);
        assert_eq!(buffer.offset, 3);
    }

    #[test]
    fn test_extend_from_slice_partial_change() {
        let mut buffer = DirtyBuffer::new();
        buffer.extend_from_slice(&[1, 2, 3]);
        buffer.reset_offset(1);
        buffer.extend_from_slice(&[4, 3]);
        assert_eq!(buffer.data, vec![1, 4, 3]);
        assert_eq!(buffer.dirty, true);
        assert_eq!(buffer.offset, 3);
    }

    #[test]
    fn test_set_offset() {
        let mut buffer = DirtyBuffer::new();
        buffer.reset_offset(5);
        assert_eq!(buffer.offset, 5);
    }
}
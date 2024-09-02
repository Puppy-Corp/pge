use std::sync::Arc;
use crate::wgpu_types::{BindableBufferRecipe, BufferRecipe};

pub struct BindableBuffer<B> {
	pub buffer: Buffer<B>,
	pub bind_group: wgpu::BindGroup,
	pub bind_group_layout: wgpu::BindGroupLayout,
}

impl<B> BindableBuffer<B>
where
	B: BindableBufferRecipe + BufferRecipe,
{
	pub fn new(name: String, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buffer = Buffer::new(name, device, queue);
		let bind_group_layout = B::create_bind_group_layout(&buffer.device);
		let bind_group = B::create_bind_group(&buffer.device, buffer.buffer(), &bind_group_layout);
		Self {
			buffer: buffer,
			bind_group_layout,
			bind_group,
		}
	}

	pub fn write(&mut self, data: &[u8]) {
		let reseized = self.buffer.write(data);
		
		if reseized {
			let layout = B::create_bind_group_layout(&self.buffer.device);
			self.bind_group = B::create_bind_group(&self.buffer.device, self.buffer.buffer(), &layout);
			self.bind_group_layout = layout;
		}
	}

	pub fn bind_group(&self) -> &wgpu::BindGroup {
		&self.bind_group
	}

	pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		&self.bind_group_layout
	}
}

pub struct Buffer<B> {
	name: String,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	buffer: wgpu::Buffer,
	size: u64,
	_marker: std::marker::PhantomData<B>,
}

impl<B: BufferRecipe> Buffer<B> {
	pub fn new(name: String, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buffer = B::create_buffer(&device, 1024);
		Self {
			name,
			device,
			queue,
			buffer,
			size: 1024,
			_marker: std::marker::PhantomData,
		}
	}

	pub fn write(&mut self, data: &[u8]) -> bool {
		let mut resized = false;

		if data.len() as u64 > self.size {
			self.resize_and_copy(data.len() as u64);
			resized = true;
		}

		self.queue.write_buffer(&self.buffer, 0, data);

		resized
	}

	fn resize_and_copy(&mut self, new_size: u64) {
		let new_size = new_size.max(1024);
		log::info!("[{}] resizing buffer from {} to {}", self.name, self.size, new_size);
		let new_buffer = B::create_buffer(&self.device, new_size);

		let mut encoder = self
			.device
			.create_command_encoder(&wgpu::CommandEncoderDescriptor {
				label: Some("resize_and_copy_encoder"),
			});

		encoder.copy_buffer_to_buffer(&self.buffer, 0, &new_buffer, 0, self.size);

		let command_buffer = encoder.finish();
		self.queue.submit(Some(command_buffer));

		self.buffer = new_buffer;
		self.size = new_size;
	}

	pub fn get_size(&self) -> u64 {
		self.size
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

#[derive(Debug, Clone, Default)]
pub struct DirtyBuffer {
    pub name: String,
    data: Vec<u8>,
    pub dirty: bool,
    offset: usize,
	pub bindable: bool
}

impl DirtyBuffer {
    /// Creates a new DirtyBuffer with the given name.
    pub fn new(name: &str, bindable: bool) -> Self {
        Self {
            name: name.to_string(),
            data: Vec::new(),
            dirty: false,
            offset: 0,
			bindable
        }
    }

    /// Returns the current length of valid data in the buffer.
    pub fn len(&self) -> usize {
        self.offset
    }

    /// Extends the buffer with data from the given slice.
    /// Marks the buffer as dirty if any changes are made.
    pub fn extend_from_slice(&mut self, slice: &[u8]) {
        if self.offset + slice.len() > self.data.len() {
            log::info!(
                "[{}] data is bigger offset: {} slice.len: {} data.len: {}",
                self.name,
                self.offset,
                slice.len(),
                self.data.len()
            );
            self.data.resize(self.offset + slice.len(), 0);
            self.dirty = true;
        }

        let current_slice = &self.data[self.offset..self.offset + slice.len()];
        if current_slice != slice {
            self.data[self.offset..self.offset + slice.len()].copy_from_slice(slice);
            self.dirty = true;
        }
        self.offset += slice.len();
    }

    /// Resets the offset to zero without modifying the underlying data.
    pub fn reset_offset(&mut self) {
        self.offset = 0;
    }

    /// Clears the buffer, removing all data and marking it as dirty.
    pub fn clear(&mut self) {
        self.data.clear();
        self.dirty = true;
        self.offset = 0;
    }

    /// Returns a slice of the valid data in the buffer.
    pub fn data(&self) -> &[u8] {
        &self.data[..self.offset]
    }
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_buffer() {
		let buffer = DirtyBuffer::new("test", false);
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_clear_buffer() {
		let mut buffer = DirtyBuffer::new("test", false);
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.clear();
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_extend_from_slice() {
		let mut buffer = DirtyBuffer::new("test", false);
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 3);
	}

	#[test]
	fn test_extend_from_slice_no_change() {
		let mut buffer = DirtyBuffer::new("test", false);
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.reset_offset();
		buffer.dirty = false;
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 3);
	}
}

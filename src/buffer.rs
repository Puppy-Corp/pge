use std::sync::Arc;
use crate::wgpu_types::{BindableBufferRecipe, BufferRecipe};

pub struct BindableBuffer<B> {
	pub buffer: Buffer<B>,
	pub bind_group: Option<wgpu::BindGroup>,
	pub bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl<B> BindableBuffer<B>
where
	B: BindableBufferRecipe + BufferRecipe,
{
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buffer = Buffer::new(device, queue);
		Self {
			buffer: buffer,
			bind_group_layout: None,
			bind_group: None,
		}
	}

	pub fn write(&mut self, data: &[u8]) {
		let reseized = self.buffer.write(data);
		
		if reseized {
			let layout = B::create_bind_group_layout(&self.buffer.device);
			self.bind_group = Some(B::create_bind_group(&self.buffer.device, self.buffer.buffer(), &layout));
			self.bind_group_layout = Some(layout);
		}
	}

	pub fn bind_group(&self) -> &wgpu::BindGroup {
		self.bind_group.as_ref().unwrap()
	}

	pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
		self.bind_group_layout.as_ref().unwrap()
	}
}

pub struct Buffer<B> {
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	buffer: Option<wgpu::Buffer>,
	size: u64,
	_marker: std::marker::PhantomData<B>,
}

impl<B: BufferRecipe> Buffer<B> {
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		Self {
			device,
			queue,
			buffer: None,
			size: 0,
			_marker: std::marker::PhantomData,
		}
	}

	pub fn write(&mut self, data: &[u8]) -> bool {
		let mut resized = false;

		if data.len() as u64 > self.size {
			self.resize_and_copy(data.len() as u64);
			resized = true;
		}

		if let Some(buffer) = &self.buffer {
			self.queue.write_buffer(buffer, 0, data);
		} else {
			panic!("this should not happen");
		}

		resized
	}

	fn resize_and_copy(&mut self, new_size: u64) {
		let new_buffer = B::create_buffer(&self.device, new_size);

		if let Some(old_buffer) = &self.buffer {
			let mut encoder = self
				.device
				.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label: Some("resize_and_copy_encoder"),
				});

			encoder.copy_buffer_to_buffer(old_buffer, 0, &new_buffer, 0, self.size);

			let command_buffer = encoder.finish();
			self.queue.submit(Some(command_buffer));
		}

		self.buffer = Some(new_buffer);
		self.size = new_size;
	}

	pub fn get_size(&self) -> u64 {
		self.size
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		self.buffer.as_ref().unwrap()
	}
}

#[derive(Debug, Clone, Default)]
pub struct DirtyBuffer {
	pub name: String,
	data: Vec<u8>,
	pub dirty: bool,
	offset: usize,
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

	pub fn len(&self) -> usize {
		self.offset
	}

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

	pub fn clear(&mut self) {
		self.data.clear();
		self.dirty = true;
		self.offset = 0;
	}

	pub fn data(&self) -> &[u8] {
		&self.data[..self.offset]
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_buffer() {
		let buffer = DirtyBuffer::new("test");
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_clear_buffer() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.clear();
		assert_eq!(buffer.data, Vec::<u8>::new());
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 0);
	}

	#[test]
	fn test_extend_from_slice() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, true);
		assert_eq!(buffer.offset, 3);
	}

	#[test]
	fn test_extend_from_slice_no_change() {
		let mut buffer = DirtyBuffer::new("test");
		buffer.extend_from_slice(&[1, 2, 3]);
		buffer.reset_offset();
		buffer.dirty = false;
		buffer.extend_from_slice(&[1, 2, 3]);
		assert_eq!(buffer.data, vec![1, 2, 3]);
		assert_eq!(buffer.dirty, false);
		assert_eq!(buffer.offset, 3);
	}
}

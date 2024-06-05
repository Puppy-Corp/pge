use std::collections::HashMap;
use std::sync::Arc;
use crate::wgpu_types::BufferRecipe;

#[derive(Debug, Clone, Copy)]
pub struct Slot {
	pub offset: usize,
	pub size: usize
}

pub struct Buffer {
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	slots: Vec<Slot>,
	buffer: wgpu::Buffer
}

impl Buffer {
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buff = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});

		Self {
			device,
			queue,
			slots: Vec::new(),
			buffer: buff
		}
	}

	pub fn store(&self, data: &[u8]) -> Slot {
		let slot = Slot {
			offset: 0,
			size: data.len()
		};

		self.queue.write_buffer(&self.buffer, 0, data);

		slot
	}

	pub fn write(&self, slot: Slot, data: &[u8]) {
		self.queue.write_buffer(&self.buffer, slot.offset as u64, data);
	}

	pub fn bind_group(&self) {

	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

pub struct StaticStagingBuffer<T> {
	blocks: Vec<T>,
	id_block_map: HashMap<usize, usize>,
	free_list: Vec<usize>,
	write_commands: Vec<usize>
}

impl<T> StaticStagingBuffer<T>
where
	T: Sized + bytemuck::Pod + bytemuck::Zeroable
{
	pub fn new(size: usize) -> Self {
		Self {
			blocks: Vec::with_capacity(size),
			id_block_map: HashMap::new(),
			free_list: Vec::new(),
			write_commands: Vec::new()
		}
	}

	pub fn store(&mut self, id: usize, item: T) -> usize {
		match self.id_block_map.get(&id) {
			Some(&index) => {
				self.blocks[index] = item;
				self.write_commands.push(index);
				index
			},
			None => {
				if let Some(index) = self.free_list.pop() {
					self.blocks[index] = item;
					self.id_block_map.insert(id, index);
					self.write_commands.push(index);
					index
				} else {
					self.blocks.push(item);
					let index = self.blocks.len() - 1;
					self.id_block_map.insert(id, index);
					self.write_commands.push(index);
					index
				}
			}
		}
	}

	pub fn store_at_inx(&mut self, index: usize, item: T) {
		self.blocks[index] = item;
		self.write_commands.push(index);
	}

	pub fn get(&self, id: usize) -> Option<&T> {
		if let Some(&index) = self.id_block_map.get(&id) {
			Some(&self.blocks[index])
		} else {
			None
		}
	}

	pub fn get_inx(&self, id: &usize) -> Option<usize> {
		self.id_block_map.get(&id).copied()
	}

	pub fn iter(&self) -> StaticBufferIterator<T> {
		StaticBufferIterator {
			data: &self.blocks,
			pointers: &self.write_commands
		}
	}

	pub fn merge_write_commands(&mut self) {

	}

	pub fn clear_write_commands(&mut self) {
		self.write_commands.clear();
	}
}

pub struct StaticBufferIterator<'a, T> {
	data: &'a [T],
	pointers: &'a [usize]
}

impl<'a, T> Iterator for StaticBufferIterator<'a, T> {
	type Item = (usize, &'a T);

	fn next(&mut self) -> Option<Self::Item> {
		if self.pointers.len() == 0 {
			return None;
		}

		let pointer = self.pointers[0];
		self.pointers = &self.pointers[1..];

		let data = &self.data[pointer];

		Some((pointer * std::mem::size_of::<T>(), data))
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Pointer {
	pub offset: usize,
	pub size: usize,
}

pub struct DynamicStagingBuffer {
	write_commands: Vec<Pointer>,
	pointers: HashMap<usize, Pointer>,
	free_list: Vec<Pointer>,
	largest_free: usize,
	data: Vec<u8>
}

impl DynamicStagingBuffer {
	pub fn new(size: usize) -> Self {
		Self {
			write_commands: Vec::new(),
			pointers: HashMap::new(),
			free_list: Vec::new(),
			largest_free: 0,
			data: Vec::with_capacity(size)
		}
	}

	fn alloc(&mut self, size: usize) -> Pointer {
        // Check if there is a free block that is large enough
        if let Some(index) = self.free_list.iter().position(|p| p.size >= size) {
            let mut free_block = self.free_list.remove(index);

            // If the free block is larger than needed, split it
            if free_block.size > size {
                let remaining_size = free_block.size - size;
                free_block.size = size;
                self.free_list.push(Pointer {
                    offset: free_block.offset + size,
                    size: remaining_size,
                });
            }

            self.largest_free = self.free_list.iter().map(|p| p.size).max().unwrap_or(0);
            return free_block;
        }

        let offset = self.data.len();
        self.largest_free += size;

        Pointer {
            offset,
            size,
        }
    }

	/// Stores data in the buffer and generates write commands.
	/// Checks if new data can fit into old location and 
	/// Moves it if it can't otherwise it updates the pointer.
	pub fn store(&mut self, id: usize, data: &[u8]) -> Pointer {
		match self.pointers.get(&id) {
			Some(p) => {
				if data.len() > p.size {
					let new_pointer = self.alloc(data.len());
					self.write_commands.push(new_pointer);
					return new_pointer.clone();
				}

				let existing_data = &mut self.data[p.offset..p.offset + p.size];

				if existing_data != data {
					existing_data.copy_from_slice(data);
					self.write_commands.push(Pointer {
						offset: p.offset,
						size: data.len()
					});
				}

				return *p;
			},
			None => {
				let pointer = self.alloc(data.len());
				self.data.extend(data);
				self.pointers.insert(id, pointer);
				self.write_commands.push(pointer);
				pointer
			}
		}
	}

	pub fn get(&self, id: usize) -> Option<&[u8]> {
		if let Some(pointer) = self.pointers.get(&id) {
			Some(&self.data[pointer.offset..pointer.offset + pointer.size])
		} else {
			None
		}
	}

	pub fn delete(&mut self, id: usize) {
		if let Some(pointer) = self.pointers.remove(&id) {
			self.free_list.push(pointer);
		}
	}

	fn merge_write_commands(&mut self) {
		if self.write_commands.len() == 0 {
			return;
		}

		self.write_commands.sort_by_key(|p| p.offset);

		let mut merged = Vec::new();
		let mut current = self.write_commands[0];

		for pointer in self.write_commands.iter().skip(1) {
			if current.offset + current.size == pointer.offset {
				current.size += pointer.size;
			} else {
				merged.push(current);
				current = *pointer;
			}
		}

		merged.push(current);
		self.write_commands = merged;
	}

	pub fn clear_write_commands(&mut self) {
		self.write_commands.clear();
	}

	pub fn iter(&self) -> WriteIterator {
		WriteIterator {
			data: &self.data,
			pointers: self.write_commands.as_slice()
		}
	}
}

pub struct WriteIterator<'a> {
	data: &'a [u8],
	pointers: &'a [Pointer],
}

impl<'a> Iterator for WriteIterator<'a> {
	type Item = (usize, &'a [u8]);

	fn next(&mut self) -> Option<Self::Item> {
		if self.pointers.len() == 0 {
			return None;
		}

		let pointer = &self.pointers[0];
		self.pointers = &self.pointers[1..];

		let data = &self.data[pointer.offset..pointer.offset + pointer.size];

		Some((pointer.offset, data))
	}
}

pub struct StaticBufferManager<T> {
	pub device: Arc<wgpu::Device>,
	pub queue: Arc<wgpu::Queue>,
	pub bind_group: wgpu::BindGroup,
	pub buffer: wgpu::Buffer,
	pub layout: Arc<wgpu::BindGroupLayout>,
	pub staging_buffer: StaticStagingBuffer<T>
}

impl<T> StaticBufferManager<T>
where
	T: Sized + bytemuck::Pod + bytemuck::Zeroable + BufferRecipe
{
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let layout = Arc::new(T::create_bind_group_layout(&device));
		let buffer = T::create_buffer(&device);
		let bind_group = T::create_bind_group(&device, &buffer, &layout);

		Self {
			device,
			queue,
			bind_group,
			buffer,
			layout,
			staging_buffer: StaticStagingBuffer::new(1024)
		}
	}

	pub fn store(&mut self, id: usize, item: T) -> usize {
		let index = self.staging_buffer.store(id, item);
		index
	}

	pub fn get_inx(&self, id: &usize) -> Option<usize> {
		self.staging_buffer.get_inx(id)
	}

	pub fn flush(&mut self) {
		// self.staging_buffer.merge_write_commands();
		for (offset, data) in self.staging_buffer.iter() {
			let bytes = bytemuck::bytes_of(data);
			println!("write offset: {}, bytes: {:?}", offset, bytes);
			self.queue.write_buffer(&self.buffer, offset as u64, bytes);
		}
		self.staging_buffer.clear_write_commands();
	}

	pub fn bind_group_layout(&self) -> Arc<wgpu::BindGroupLayout> {
		self.layout.clone()
	}

	pub fn bind_group(&self) -> &wgpu::BindGroup {
		&self.bind_group
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

pub struct DynamicVertexBuffer {
	pub device: Arc<wgpu::Device>,
	pub queue: Arc<wgpu::Queue>,
	pub buffer: wgpu::Buffer,
	pub staging_buffer: DynamicStagingBuffer
}

impl DynamicVertexBuffer
{
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Dynamic Vertex Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});
		Self {
			device,
			queue,
			buffer,
			staging_buffer: DynamicStagingBuffer::new(1024)
		}
	}

	pub fn store(&mut self, id: usize, data: &[u8]) -> Pointer {
		self.staging_buffer.store(id, data)
	}

	pub fn flush(&mut self) {
		// self.staging_buffer.merge_write_commands();
		for (offset, data) in self.staging_buffer.iter() {
			println!("write offset: {}, bytes: {:?}", offset, data);
			self.queue.write_buffer(&self.buffer, offset as u64, data);
		}
		self.staging_buffer.clear_write_commands();
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

pub struct FixedVertexBuffer<T> {
	pub device: Arc<wgpu::Device>,
	pub queue: Arc<wgpu::Queue>,
	pub buffer: wgpu::Buffer,
	pub staging_buffer: StaticStagingBuffer<T>
}

impl<T> FixedVertexBuffer<T>
where
	T: Sized + bytemuck::Pod + bytemuck::Zeroable
{
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Fixed Vertex Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false
		});
		Self {
			device,
			queue,
			buffer,
			staging_buffer: StaticStagingBuffer::new(1024)
		}
	}

	pub fn store(&mut self, id: usize, item: T) -> usize {
		self.staging_buffer.store(id, item)
	}

	pub fn flush(&mut self) {
		// self.staging_buffer.merge_write_commands();
		for (offset, data) in self.staging_buffer.iter() {
			let bytes = bytemuck::bytes_of(data);
			println!("write offset: {}, bytes: {:?}", offset, bytes);
			self.queue.write_buffer(&self.buffer, offset as u64, bytes);
		}
		self.staging_buffer.clear_write_commands();
	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_allocation() {
		let mut buff = DynamicStagingBuffer::new(1024);
		let pointer1 = buff.store(1, &[1, 2, 3, 4]);
		assert_eq!(pointer1.offset, 0);
		assert_eq!(pointer1.size, 4);
		let pointer2 =  buff.store(2, &[5, 6, 7, 8]);
		assert_eq!(pointer2.offset, 4);
		assert_eq!(pointer2.size, 4);
		let data = buff.get(1).unwrap();
		assert_eq!(data, &[1, 2, 3, 4]);
		let data = buff.get(2).unwrap();
		assert_eq!(data, &[5, 6, 7, 8]);

		buff.merge_write_commands();

		let iter: Vec<_> = buff.iter().collect();

		assert_eq!(iter.len(), 1);
		let (offset, data) = iter[0];
		assert_eq!(offset, 0);
		assert_eq!(data, &[1, 2, 3, 4, 5, 6, 7, 8]);
	}
}
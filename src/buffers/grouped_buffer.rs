use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;
use bytemuck::Pod;
use wgpu::Device;
use crate::wgpu_types::WgpuBuffer;

use super::types::WriteCommand;

pub struct GroupedVertexBuffer {
	device: Arc<wgpu::Device>,
	buffer: wgpu::Buffer,
	staging_buffer: GroupedBuffer
}

impl GroupedVertexBuffer {
	pub fn new<T: Clone + WgpuBuffer + Pod>(device: Arc<Device>) -> Self {
		let buffer = T::create_buffer(&device, 1024);

		Self {
			device,
			buffer,
			staging_buffer: GroupedBuffer::new()
		}
	}

	pub fn store<T: Pod>(&mut self, group: u32, id: u32, item: &T) -> Range<u32> {
		self.staging_buffer.store(group, id, item)
	}

	pub fn wgpu_buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}

pub struct GroupedBuffer {
    buffer: Vec<u8>,
    groups: HashMap<u32, HashMap<u32, Range<u32>>>,
    modified_ranges: HashSet<Range<u32>>,
}

impl GroupedBuffer {
    pub fn new() -> Self {
        GroupedBuffer {
            buffer: Vec::new(),
            groups: HashMap::new(),
            modified_ranges: HashSet::new(),
        }
    }

    pub fn store<T: Pod>(&mut self, group: u32, id: u32, item: &T) -> Range<u32> {
        println!("store group: {}, id: {}", group, id);
        let size = std::mem::size_of::<T>();

        let group_map = self.groups.entry(group).or_insert_with(HashMap::new);
        
        if let Some(range) = group_map.get(&id) {
            let start = range.start as usize;
            let end = range.end as usize;
            let item_bytes = bytemuck::bytes_of(item);
            self.buffer[start..end].copy_from_slice(item_bytes);
            self.modified_ranges.insert(range.clone());
            range.clone()
        } else {
            let start = self.buffer.len() as u32;
            let end = start + size as u32;
            let range = start..end;
            let item_bytes = bytemuck::bytes_of(item);
            self.buffer.extend_from_slice(item_bytes);
            group_map.insert(id, range.clone());
            self.modified_ranges.insert(range.clone());
            range
        }
    }

	pub fn remove(&mut self, group: u32, id: u32) {
		if let Some(group_map) = self.groups.get_mut(&group) {
			if let Some(range) = group_map.remove(&id) {
				// Zero out the removed range
				for i in range.clone() {
					self.buffer[i as usize] = 0;
				}
				// Mark the range as modified
				self.modified_ranges.insert(range);
			}
		}
	}

	pub fn generate_write_commands(&mut self) -> Vec<WriteCommand> {
		let mut commands = Vec::new();

		for range in self.modified_ranges.drain() {
			let command = WriteCommand {
				offset: range.start as usize,
				data: self.buffer[range.start as usize..range.end as usize].to_vec(),
			};
			commands.push(command);
		}

		commands
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_mesh_with_two_instances() {
		#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
		#[repr(C)]
		struct Instance {
			position: [f32; 3],
		}

		let mesh_one_id = 0;
		let mesh_two_id = 1;

		let mut buffer = GroupedBuffer::new();

		let range = buffer.store(mesh_one_id, 1, &Instance { position: [1.0, 2.0, 3.0] });
		assert_eq!(range, 0..12);
		let range = buffer.store(mesh_one_id, 2, &Instance { position: [4.0, 5.0, 6.0] });
		assert_eq!(range, 0..24);
		let range = buffer.store(mesh_two_id, 1, &Instance { position: [7.0, 8.0, 9.0] });
		assert_eq!(range, 24..36);
		let range = buffer.store(mesh_two_id, 2, &Instance { position: [10.0, 11.0, 12.0] });
		assert_eq!(range, 24..48);
	}
}
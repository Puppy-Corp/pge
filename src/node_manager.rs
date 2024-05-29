use std::collections::HashMap;

use crate::internal::WriteCommand;
use crate::wgpu_types::NodeTransform;

#[derive(Debug, Default, Clone, Copy)]
pub struct NodeMetadata {
	pub id: usize,
	pub model: glam::Mat4,
	pub parent_id: Option<usize>
}

impl NodeMetadata {
	pub fn to_bytes(&self) -> Vec<u8> {
		let n = NodeTransform {
			_padding: [0; 3],
			model: self.model.to_cols_array_2d(),
			parent_index: self.parent_id.map_or(-1, |p| p as i32),
		};
		bytemuck::bytes_of(&n).to_vec()
	}
}

pub struct NodeManager {
    nodes: Vec<NodeMetadata>,
    id_map: HashMap<usize, usize>, // Maps global ID to local buffer index
    free_list: Vec<usize>,       // List of free slots in the buffer
	changed: Vec<usize>
}

impl NodeManager {
	pub fn new() -> Self {
		Self {
			nodes: Vec::new(),
			id_map: HashMap::new(),
			free_list: Vec::new(),
			changed: Vec::new()
		}
	}

	pub fn upsert_node(&mut self, node: NodeMetadata) {
		if let Some(index) = self.id_map.get(&node.id) {
			self.nodes[*index] = node;
			self.changed.push(*index);
		} else {
			let node_id = node.id;
			if let Some(free_index) = self.free_list.pop() {
				self.nodes[free_index] = node;
				self.id_map.insert(node_id, free_index);
				self.changed.push(free_index);
			} else {
				self.nodes.push(node);
				let inx = self.nodes.len() - 1;
				self.id_map.insert(node_id, inx);
				self.changed.push(inx);
			}
		}
	}

	pub fn delete_node(&mut self, id: usize) {
		if let Some(index) = self.id_map.get(&id) {
			self.free_list.push(*index);
			self.id_map.remove(&id);
		}
	}

	pub fn get_write_commands(&self) -> Vec<WriteCommand>{
		let mut commands = Vec::new();
		for inx in &self.changed {
			let node = &self.nodes[*inx];
			let data = node.to_bytes();
			commands.push(WriteCommand {
				start: *inx * std::mem::size_of::<NodeMetadata>(),
				data
			});
		}
		commands
	}
}
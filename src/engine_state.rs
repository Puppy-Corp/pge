use std::collections::HashMap;
use std::collections::HashSet;

use bytemuck::bytes_of;
use thunderdome::Index;

use crate::physics::physics_update;
use crate::renderer::DrawCall;
use crate::spatial_grid::SpatialGrid;
use crate::wgpu_types::*;
use crate::Node;
use crate::State;

pub struct EngineState {
	pub state: State,
	grid: SpatialGrid,
	nodes: HashMap<Index, Node>,
	instances: HashMap<Index, RawInstance>,
	meshes: HashSet<Index>,
	cameras: HashMap<Index, RawCamera>,
	pub draw_calls: Vec<DrawCall>,
	pub all_instances_data: Vec<u8>,
	pub all_positions_data: Vec<u8>,
	pub all_normals_data: Vec<u8>,
	pub all_indices_data: Vec<u8>,
	pub all_nodes_data: Vec<u8>,
	pub all_cameras_data: Vec<u8>,
}

impl EngineState {
	pub fn new() -> Self {
		Self {
			state: State::default(),
			grid: SpatialGrid::new(10.0, 100),
			nodes: HashMap::new(),
			draw_calls: Vec::new(),
			instances: HashMap::new(),
			meshes: HashSet::new(),
			cameras: HashMap::new(),
			all_instances_data: Vec::new(),
			all_positions_data: Vec::new(),
			all_normals_data: Vec::new(),
			all_indices_data: Vec::new(),
			all_nodes_data: Vec::new(),
			all_cameras_data: Vec::new(),
		}
	}

	pub fn update_buffers(&mut self) {
		self.draw_calls.clear();
		self.all_instances_data.clear();
		self.all_positions_data.clear();
		self.all_normals_data.clear();
		self.all_indices_data.clear();
		self.all_nodes_data.clear();
		self.all_cameras_data.clear();
		let mut instance_count = 0;

		let mut mesh_instances: HashMap<Index, Vec<RawInstance>> = HashMap::new();
		let mut node_indexes: HashMap<Index, i32> = HashMap::new();

		for (node_inx, (node_id, node)) in self.state.nodes.iter().enumerate() {
			let model = glam::Mat4::from_quat(node.rotation) * glam::Mat4::from_translation(node.translation) * glam::Mat4::from_scale(node.scale);
			let raw_node = RawNode {
				model: model.to_cols_array_2d(),
				parent_index: -1,
				_padding: [0; 3]
			};

			// if !self.nodes.contains(&node_id) {
			// 	self.nodes.insert(node_id, node.clone());
			// 	log::info!("new node: {:?}", node_id);
			// 	self.grid.add_node(node_id, rect);
			// 	println!("new nodex_ix: {}  node_id: {:?} node_name: {:?} node: {:?}", node_inx, node_id, node.name, raw_node);
			// }

			match self.nodes.get(&node_id) {
				Some(old_node) => {
					if let Some(collision_shape) = &node.collision_shape {
						if old_node.translation != node.translation {
							self.grid.move_node(node_id, collision_shape.aabb(node.translation))
						}
					}

					self.nodes.insert(node_id, node.clone());
				},
				None => {
					log::info!("new node node_id: {:?} node: {:?}", node_id, node);
					self.nodes.insert(node_id, node.clone());

					if let Some(collision_mesh) = &node.collision_shape {
						self.grid.add_node(node_id, collision_mesh.aabb(node.translation))
					}
				}
			}

			node_indexes.insert(node_id, node_inx as i32);
			self.all_nodes_data.extend_from_slice(bytes_of(&raw_node));
			
			if let Some(mesh_id) = node.mesh {
				let instance = RawInstance {
					node_index: node_inx as i32
				};

				match self.instances.get(&mesh_id) {
					Some(instance) => {
						self.instances.insert(mesh_id, *instance);
					},
					None => {
						log::info!("new instance mesh_id: {:?} instance: {:?}", mesh_id, instance);
						self.instances.insert(mesh_id, instance);
					}
				}

				mesh_instances.entry(mesh_id).or_insert(Vec::new()).push(instance);
			}
		}

		for (mesh_id, mesh) in &self.state.meshes {
			// println!("mesh_id {:?}", mesh_id);
			let positions_start = self.all_positions_data.len() as u64;
			self.all_positions_data.extend_from_slice(bytemuck::cast_slice(&mesh.positions));
			let positions_end = self.all_positions_data.len() as u64;
			let indices_start = self.all_indices_data.len() as u64;
			self.all_indices_data.extend_from_slice(bytemuck::cast_slice(&mesh.indices));
			let indices_end = self.all_indices_data.len() as u64;
			let normals_start = self.all_normals_data.len() as u64;
			self.all_normals_data.extend_from_slice(bytemuck::cast_slice(&mesh.normals));
			let normals_end = self.all_normals_data.len() as u64;

			let instances: &Vec<RawInstance> = match mesh_instances.get(&mesh_id) {
				Some(instances) => instances,
				None => continue,
			};

			let instance_start = instance_count;
			self.all_instances_data.extend_from_slice(bytemuck::cast_slice(instances));
			instance_count += instances.len() as u32;
			let instance_end = instance_count;

			let draw_instruction = DrawCall {
				position_range: positions_start..positions_end,
				normal_range: normals_start..normals_end,
				index_range: indices_start..indices_end,
				indices_range: 0..mesh.indices.len() as u32,
				instances_range: instance_start..instance_end
			};

			match self.meshes.contains(&mesh_id) {
				true => {},
				false => {
					log::info!("new mesh mesh_id: {:?} mesh: {:?}", mesh_id, mesh);
					log::info!("draw_instruction: {:?}", draw_instruction);
					log::info!("instances: {:?}", instances);
					self.meshes.insert(mesh_id);
				},
			}

			self.draw_calls.push(draw_instruction);
		}

		for (cam_id, cam) in &self.state.cameras {
			let node_inx = match cam.node_id {
				Some(id) => {
					match node_indexes.get(&id) {
						Some(inx) => *inx,
						None => continue,
					}
				}
				None => continue,
			};

			let cam = RawCamera {
				proj: glam::Mat4::perspective_rh(cam.fovy, cam.aspect, cam.znear, cam.zfar).to_cols_array_2d(),
				_padding: [0; 3],
				node_inx
			};

			match self.cameras.get(&cam_id) {
				Some(camera) => {
					self.cameras.insert(cam_id, *camera);
				},
				None => {
					log::info!("new camera cam_id: {:?} camera: {:?} node_inx: {}", cam_id, cam, node_inx);
					self.cameras.insert(cam_id, cam);
				}
			}

			self.all_cameras_data.extend_from_slice(bytes_of(&cam));
		}
	}

	pub fn physics_update(&mut self, dt: f32) {
		// update physics
		physics_update(&mut self.state, &mut self.grid, dt)
	}
}
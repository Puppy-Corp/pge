use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::time::Instant;

use bytemuck::bytes_of;
use glam::Vec2;
use glam::Vec3;
use thunderdome::Index;

use crate::compositor::UICompositor;
use crate::debug::ChangePrinter;
use crate::gltf::load_gltf;
use crate::physics::PhysicsSystem;
use crate::spatial_grid::SpatialGrid;
use crate::wgpu_types::*;
use crate::Node;
use crate::PrimitiveTopology;
use crate::State;
use crate::Texture;
use crate::AABB;

const REM_NODE_SLOT: u32 = 0;
const ADD_NODE_SLOT: u32 = 1;
const NODE_UPDATE_TIME_SLOT: u32 = 2;
const BROAD_PHASE_TIME_SLOT: u32 = 3;
const NARROW_PHASE_TIME_SLOT: u32 = 4;

#[derive(Debug, Clone)]
pub struct Gemometry {
	pub vertices: Vec<u8>,
	pub normals: Vec<u8>,
	pub tex_coords: Vec<u8>,
	pub indices: Vec<u8>,
}

impl Gemometry {
	pub fn new() -> Self {
		Self {
			vertices: Vec::new(),
			normals: Vec::new(),
			tex_coords: Vec::new(),
			indices: Vec::new(),
		}
	}

	pub fn clear(&mut self) {
		self.vertices.clear();
		self.normals.clear();
		self.tex_coords.clear();
		self.indices.clear();
	}
}

#[derive(Debug, Clone)]
pub struct DrawCall {
	pub texture: Option<Index>,
	pub position_range: Range<u64>,
	pub index_range: Range<u64>,
	pub normal_range: Range<u64>,
	pub tex_coords_range: Range<u64>,
	pub instances_range: Range<u32>,
	pub indices_range: Range<u32>,
}

pub struct EngineState {
	pub state: State,
	grid: SpatialGrid,
	nodes: HashMap<Index, Node>,
	instances: HashMap<Index, RawInstance>,
	meshes: HashSet<Index>,
	cameras: HashMap<Index, RawCamera>,
	printer: ChangePrinter,
	pub draw_calls: Vec<DrawCall>,
	pub all_instances_data: Vec<u8>,
	// pub all_positions_data: Vec<u8>,
	// pub all_tex_coords_data: Vec<u8>,
	// pub all_normals_data: Vec<u8>,
	// pub all_indices_data: Vec<u8>,
	pub all_nodes_data: Vec<u8>,
	pub all_cameras_data: HashMap<Index, Vec<u8>>,
	pub all_point_lights_data: Vec<u8>,
	pub triangles: Gemometry,
	pub move_nodes: Vec<(Index, AABB)>,
	rem_nodes: HashSet<Index>,
	add_nodes: Vec<(Index, AABB)>,
	assets: HashSet<String>,
	pub ui_compositors: HashMap<Index, UICompositor>,
	pub textures: HashMap<Index, Texture>,
	pub physics_system: PhysicsSystem,
}

impl EngineState {
	pub fn new() -> Self {
		Self {
			state: State::default(),
			grid: SpatialGrid::new(5.0),
			nodes: HashMap::new(),
			draw_calls: Vec::new(),
			instances: HashMap::new(),
			meshes: HashSet::new(),
			cameras: HashMap::new(),
			all_instances_data: Vec::new(),
			// all_positions_data: Vec::new(),
			// all_tex_coords_data: Vec::new(),
			// all_normals_data: Vec::new(),
			// all_indices_data: Vec::new(),
			all_nodes_data: Vec::new(),
			all_cameras_data: HashMap::new(),
			all_point_lights_data: Vec::new(),
			triangles: Gemometry::new(),
			printer: ChangePrinter::new(),
			move_nodes: Vec::new(),
			rem_nodes: HashSet::new(),
			add_nodes: Vec::new(),
			assets: HashSet::new(),
			physics_system: PhysicsSystem::new(),
			ui_compositors: HashMap::new(),
			textures: HashMap::new(),
		}
	}

	pub fn update_buffers(&mut self) {
		let timer = Instant::now();
		self.draw_calls.clear();
		self.all_instances_data.clear();
		// self.all_positions_data.clear();
		// self.all_tex_coords_data.clear();
		// self.all_normals_data.clear();
		// self.all_indices_data.clear();
		self.all_nodes_data.clear();
		self.all_cameras_data.clear();
		self.all_point_lights_data.clear();
		self.triangles.clear();
		let mut instance_count = 0;

		let mut mesh_instances: HashMap<Index, Vec<RawInstance>> = HashMap::new();
		let mut node_indexes: HashMap<Index, i32> = HashMap::new();

		for (node_inx, (node_id, node)) in self.state.nodes.iter().enumerate() {
			let model = glam::Mat4::from_translation(node.translation)
				* glam::Mat4::from_quat(node.rotation)
				* glam::Mat4::from_scale(node.scale);
			let raw_node = RawNode {
				model: model.to_cols_array_2d(),
				parent_index: -1,
				_padding: [0; 3]
			};

			match self.nodes.get(&node_id) {
				Some(old_node) => {
					if let Some(collision_shape) = &node.collision_shape {
						if old_node.translation != node.translation {
							self.rem_nodes.insert(node_id);
							self.add_nodes.push((node_id, collision_shape.aabb(node.translation)));
						}
					}

					self.nodes.insert(node_id, node.clone());
				},
				None => {
					log::info!("new node node_id: {:?} node: {:?}", node_id, node);
					self.nodes.insert(node_id, node.clone());

					if let Some(collision_mesh) = &node.collision_shape {
						self.add_nodes.push((node_id, collision_mesh.aabb(node.translation)));
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
			for primitive in &mesh.primitives {
				if primitive.topology == PrimitiveTopology::TriangleList {			
					if primitive.vertices.len() == 0 || primitive.indices.len() == 0 {
						continue;
					}

					let positions_start = self.triangles.vertices.len() as u64;
					self.triangles.vertices.extend_from_slice(bytemuck::cast_slice(&primitive.vertices));
					let positions_end = self.triangles.vertices.len() as u64;
					let normals_start = self.triangles.normals.len() as u64;
					self.triangles.normals.extend_from_slice(bytemuck::cast_slice(&primitive.normals));
					let normals_end = self.triangles.normals.len() as u64;
					let indices_start = self.triangles.indices.len() as u64;
					self.triangles.indices.extend_from_slice(bytemuck::cast_slice(&primitive.indices));
					let indices_end = self.triangles.indices.len() as u64;
					let tex_coords_start = self.triangles.tex_coords.len() as u64;
					self.triangles.tex_coords.extend_from_slice(bytemuck::cast_slice(&primitive.tex_coords));
					let tex_coords_end = self.triangles.tex_coords.len() as u64;
					
					let instances = match mesh_instances.get(&mesh_id) {
						Some(instances) => instances,
						None => {
							continue
						},
					};

					let instance_start = instance_count;
					self.all_instances_data.extend_from_slice(bytemuck::cast_slice(instances));
					instance_count += instances.len() as u32;
					let instance_end = instance_count;
		
					let draw_instruction = DrawCall {
						position_range: positions_start..positions_end,
						normal_range: normals_start..normals_end,
						index_range: indices_start..indices_end,
						indices_range: 0..primitive.indices.len() as u32,
						instances_range: instance_start..instance_end,
						tex_coords_range: tex_coords_start..tex_coords_end,
						texture: mesh.texture,
					};

					self.draw_calls.push(draw_instruction);
				}
			}
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

			let cam_model_matrix = match cam.node_id {
				Some(id) => {
					let node = match self.state.nodes.get(id) {
						Some(node) => node,
						None => continue,
					};

					glam::Mat4::from_translation(node.translation) * glam::Mat4::from_quat(node.rotation)
				},
				None => glam::Mat4::IDENTITY,
			};

			let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar) * cam_model_matrix.inverse();

			let cam = RawCamera {
				proj: model
					// .transpose()
					//.inverse()
					.to_cols_array_2d(),
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

			self.all_cameras_data.insert(cam_id, bytes_of(&cam).to_vec());
		}

		for (node_id, light) in &self.state.point_lights {
			let node_inx = match light.node_id {
				Some(id) => {
					match node_indexes.get(&id) {
						Some(inx) => *inx,
						None => continue,
					}
				}
				None => continue,
			};

			let light = RawPointLight {
				color: light.color.into(),
				intensity: light.intensity,
				node_inx
			};

			self.all_point_lights_data.extend_from_slice(bytes_of(&light));
		}

		if self.rem_nodes.len() > 0 {
			let rem_timer = Instant::now();
			self.grid.rem_nodes(&self.rem_nodes);
			self.printer.print(REM_NODE_SLOT, format!("rem nodes total time: {}ms", rem_timer.elapsed().as_millis()));
			self.rem_nodes.clear();
		}

		if self.add_nodes.len() > 0 {
			let add_timer = Instant::now();
			for (node_id, aabb) in self.add_nodes.drain(..) {
				self.grid.add_node(node_id, aabb);
			}
			self.printer.print(ADD_NODE_SLOT, format!("add nodes total time: {}ms", add_timer.elapsed().as_millis()));
		}
	}

	pub fn update_assets(&mut self) {
		let paths = self.state.assets_3d.iter().map(|p| p.1.path.clone()).collect::<Vec<String>>();

		for path in paths {
			if self.assets.contains(&path) {
				continue;
			}

			self.assets.insert(path.clone());

			load_gltf(&path, &mut self.state);
		}
	}

	pub fn physics_update(&mut self, dt: f32) {
		// update physics
		let timings = self.physics_system.physics_update(&mut self.state, &mut self.grid, dt);
		if timings.node_update_time > 3 {
			self.printer.print(NODE_UPDATE_TIME_SLOT, format!("node_update_time: {}", timings.node_update_time));
		}
		if timings.broad_phase_time > 10 {
			self.printer.print(BROAD_PHASE_TIME_SLOT, format!("broad_phase_time: {}", timings.broad_phase_time));
		}
		if timings.narrow_phase_time > 3 {
			self.printer.print(NARROW_PHASE_TIME_SLOT, format!("narrow_phase_time: {}", timings.narrow_phase_time));
		}
		if timings.resolve_collision_time > 0 {
			self.printer.print(NARROW_PHASE_TIME_SLOT, format!("resolve_collision_time: {}", timings.resolve_collision_time));
		}

		for (_, ray_cast) in &mut self.state.raycasts {
			ray_cast.intersects.clear();
		
			let node = match self.state.nodes.get(ray_cast.node_inx) {
				Some(node) => node,
				None => continue,
			};
		
			let start = node.translation;
			let end = start + node.rotation * Vec3::new(0.0, 0.0, 1.0) * ray_cast.len;
			let nodes = self.grid.get_line_ray_nodes(start, end);
		
			let mut intersections = Vec::new();
		
			for node_inx in nodes {
				if node_inx == ray_cast.node_inx {
					continue;
				}
		
				let aabb = match self.grid.get_node_rect(node_inx) {
					Some(aabb) => aabb,
					None => continue,
				};
		
				if let Some((tmin, _tmax)) = aabb.intersect_ray(start, end) {
					intersections.push((tmin, node_inx));
				}
			}
		
			// Sort the intersections by tmin
			intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
		
			// Store the sorted node indices
			ray_cast.intersects = intersections.into_iter().map(|(_, node_inx)| node_inx).collect();
		}		
	}

	pub fn update_guis(&mut self) {
		for (gui_id, gui) in &self.state.guis {
			let compositor = self.ui_compositors.entry(gui_id).or_insert(UICompositor::new());
			compositor.process(gui);
		}
	}
}
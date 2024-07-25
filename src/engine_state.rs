use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::time::Instant;

use bytemuck::bytes_of;
use glam::Vec2;
use glam::Vec3;
use thunderdome::Arena;
use thunderdome::Index;

use crate::compositor::UICompositor;
use crate::debug::ChangePrinter;
use crate::gltf::load_gltf;
use crate::internal_types::CamView;
use crate::physics::PhysicsSystem;
use crate::spatial_grid::SpatialGrid;
use crate::wgpu_types::*;
use crate::Mesh;
use crate::NodeParent;
use crate::PrimitiveTopology;
use crate::State;
use crate::Texture;
use crate::AABB;

const REM_NODE_SLOT: u32 = 0;
const ADD_NODE_SLOT: u32 = 1;
const NODE_UPDATE_TIME_SLOT: u32 = 2;
const BROAD_PHASE_TIME_SLOT: u32 = 3;
const NARROW_PHASE_TIME_SLOT: u32 = 4;

#[derive(Debug, Clone, Default)]
pub struct DirtyBuffer {
	pub data: Vec<u8>,
	pub dirty: bool,
}

impl DirtyBuffer {
	pub fn new() -> Self {
		Self {
			data: Vec::new(),
			dirty: false,
		}
	}

	pub fn clear(&mut self) {
		self.data.clear();
		self.dirty = true;
	}

	pub fn len(&self) -> usize {
		self.data.len()
	}

	pub fn extend_from_slice(&mut self, slice: &[u8]) {
		self.data.extend_from_slice(slice);
		self.dirty = true;
	}
}

#[derive(Debug, Clone, Default)]
pub struct Gemometry {
	pub vertices: DirtyBuffer,
	pub normals: DirtyBuffer,
	pub tex_coords: DirtyBuffer,
	pub indices: DirtyBuffer,
}

impl Gemometry {
	pub fn new() -> Self {
		Self {
			vertices: DirtyBuffer::new(),
			normals: DirtyBuffer::new(),
			tex_coords: DirtyBuffer::new(),
			indices: DirtyBuffer::new(),
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
	pub vertices: Range<u64>,
	pub indices: Range<u64>,
	pub normals: Range<u64>,
	pub tex_coords: Range<u64>,
	pub instances: Range<u32>,
	pub indices_range: Range<u32>,
}

pub struct UIDrawCalls {
	pub vertices: Vec<[f32; 3]>,
	pub indices: Vec<u16>,
	pub colors: Vec<[f32; 3]>
}

pub struct DrawCalls3D {
	pub camera: Vec<u8>,
	pub nodes: Vec<u8>,
	pub point_lights: Vec<u8>,
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32,
	pub calls: Vec<DrawCall>,
}

// #[derive(Debug, Clone)]
// pub struct WindowDrawData {
// 	pub ui: UIDrawCalls,
// 	pub draw_calls: DrawCalls3D,
// }

struct GuiBuffers {
	vertices: DirtyBuffer,
	indices: DirtyBuffer,
	colors: DirtyBuffer,
}

struct SceneData {
	nodes: Vec<Index>,

}

#[derive(Debug, Clone)]
struct NodeMetadata {
	model: glam::Mat4,
	scene_id: Index,
}

#[derive(Debug, Clone)]
struct CollisionNode {
	node_id: Index,
	aabb: AABB,
}

#[derive(Debug, Clone)]
struct MeshPointer {
	positions: Range<u64>,
	normals: Range<u64>,
	tex_coords: Range<u64>,
	indices: Range<u64>,
	indice_count: u32,
}

struct SceneDrawInstruction {
	draw_calls: Vec<DrawCall>,
}

#[derive(Debug, Clone)]
pub struct UIRenderArgs {
	pub ui: Index,
	pub views: Vec<CamView>,
}

#[derive(Debug, Clone)]
pub struct SceneCollection {
	collision_nodes: Vec<CollisionNode>,
	draw_calls: Vec<DrawCall>,
	grid: SpatialGrid,
	physics_system: PhysicsSystem,
}

#[derive(Debug, Clone)]
pub struct Buffer {
	pub data: Vec<u8>,
	pub dirty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EngineState {
	pub state: State,
	grids: HashMap<Index, SpatialGrid>,
	nodes: HashMap<Index, NodeMetadata>,
	instances: HashMap<Index, RawInstance>,
	meshes: HashMap<Index, Mesh>,
	cameras: HashMap<Index, RawCamera>,
	printer: ChangePrinter,
	pub all_instances_data: Vec<u8>,
	pub all_cameras_data: HashMap<Index, Vec<u8>>,
	pub camera_buffer: DirtyBuffer,
	camera_pointers: HashMap<Index, Range<u64>>,
	// camera_draw_calls: HashMap<Index, Vec<DrawCall>>,
	pub all_point_lights_data: Vec<u8>,
	pub triangles: Gemometry,
	pub move_nodes: Vec<(Index, AABB)>,
	// rem_nodes: HashSet<Index>,
	// add_nodes: Vec<(Index, AABB)>,
	assets: HashSet<String>,
	pub ui_compositors: HashMap<Index, UICompositor>,
	ui_render_args: HashMap<Index, UIRenderArgs>,
	pub textures: HashMap<Index, Texture>,
	// pub physics_system: PhysicsSystem,
	// pub window_draw_data: HashMap<Index, WindowDrawData>,
	mesh_pointers: HashMap<Index, MeshPointer>,
	mesh_nodes: HashMap<Index, Vec<Index>>,
	scene_collision_nodes: HashMap<Index, Vec<CollisionNode>>,
	scene_draw_calls: HashMap<Index, Vec<DrawCall>>,
	scene_lights: HashMap<Index, DirtyBuffer>,
	// scene_meshes: HashMap<Index, Vec<Index>>,
	scene_instance_buffers: HashMap<Index, DirtyBuffer>,
	scene_collections: HashMap<Index, SceneCollection>,
	pub buffers: Arena<Buffer>,
}

impl EngineState {
	pub fn new() -> Self {
		Default::default()
	}

	fn process_nodes(&mut self) {
		let mut processed_nodes: HashSet<Index> = HashSet::new();

		for (node_id, node) in &self.state.nodes {
			if processed_nodes.contains(&node_id) {
				continue;
			}

			let mut stack = vec![node_id];

			while let Some(node_id) = stack.last() {
				let node_id = *node_id;

				let node = match self.state.nodes.get(node_id) {
					Some(node) => node,
					None => continue,
				};

				match node.parent {
					NodeParent::Node(parent_node_id) => {
						match processed_nodes.contains(&parent_node_id) {
							true => {
								let parent = match self.nodes.get(&parent_node_id) {
									Some(model) => model,
									None => {
										stack.push(parent_node_id);
										continue;
									},
								};
								let model = parent.model * glam::Mat4::from_translation(node.translation)
									* glam::Mat4::from_quat(node.rotation)
									* glam::Mat4::from_scale(node.scale);

								let node_metadata = NodeMetadata {
									model,
									scene_id: parent.scene_id
								};

								if let Some(collision_shape) = &node.collision_shape {
									match self.nodes.get(&node_id) {
										Some(old) => {
											if old.model != model {
												let collision_node = CollisionNode {
													node_id,
													aabb: collision_shape.aabb(node.translation)
												};

												self.scene_collision_nodes.entry(parent.scene_id).or_insert(Vec::new()).push(collision_node);
											}
										},
										None => {
											let collision_node = CollisionNode {
												node_id,
												aabb: collision_shape.aabb(node.translation)
											};

											self.scene_collision_nodes.entry(parent.scene_id).or_insert(Vec::new()).push(collision_node);
										}
									}
								}

								self.nodes.insert(node_id, node_metadata);
							},
							false => {
								stack.push(parent_node_id);
								continue;
							}
						}
					},
					NodeParent::Scene(scene_id) => {
						let model = glam::Mat4::from_translation(node.translation)
							* glam::Mat4::from_quat(node.rotation)
							* glam::Mat4::from_scale(node.scale);
						let node = NodeMetadata {
							scene_id,
							model
						};
						self.nodes.insert(node_id, node);
					},
					NodeParent::Orphan => {},
				}

				if let Some(mesh_id) = node.mesh {
					self.mesh_nodes.entry(mesh_id).or_insert(Vec::new()).push(node_id);
				}

				stack.pop();
				processed_nodes.insert(node_id);
			}
		}
	}

	fn process_meshes(&mut self) {
		for (mesh_id, mesh) in &self.state.meshes {
			for primitive in &mesh.primitives {
				if primitive.topology == PrimitiveTopology::TriangleList {
					if primitive.vertices.len() == 0 || primitive.indices.len() == 0 {
						continue;
					}

					let positions_start = self.triangles.vertices.data.len() as u64;
					self.triangles.vertices.extend_from_slice(bytemuck::cast_slice(&primitive.vertices));
					let positions_end = self.triangles.vertices.len() as u64;
					let normals_start = self.triangles.normals.len() as u64;
					self.triangles.normals.extend_from_slice(bytemuck::cast_slice(&primitive.normals));
					let normals_end = self.triangles.normals.len() as u64;
					let indices_start = self.triangles.indices.len() as u64;
					self.triangles.indices.extend_from_slice(bytemuck::cast_slice(&primitive.indices));
					let indices_end = self.triangles.indices.len() as u64;
					let tex_coords_start = self.triangles.tex_coords.len() as u64;
					if primitive.tex_coords.len() > 0 {
						self.triangles.tex_coords.extend_from_slice(bytemuck::cast_slice(&primitive.tex_coords));
					} else {
						let tex_coords = vec![[0.0, 0.0]; primitive.vertices.len()];
						self.triangles.tex_coords.extend_from_slice(bytemuck::cast_slice(&tex_coords));
					}
					let tex_coords_end = self.triangles.tex_coords.len() as u64;

					let pointer = MeshPointer {
						positions: positions_start..positions_end,
						normals: normals_start..normals_end,
						tex_coords: tex_coords_start..tex_coords_end,
						indices: indices_start..indices_end,
						indice_count: primitive.indices.len() as u32,
					};
					self.mesh_pointers.insert(mesh_id, pointer);

					let node_ids = match self.mesh_nodes.get(&mesh_id) {
						Some(nodes) => nodes,
						None => {
							continue
						},
					};

					let mut checkpoints: HashMap<Index, Range<u32>> = HashMap::new();
					
					for node_id in node_ids {
						let node = match self.nodes.get(node_id) {
							Some(node) => node,
							None => {
								continue
							},
						};

						let instance = RawInstance {
							model: node.model.to_cols_array_2d(),
						};

						let buffer = self.scene_instance_buffers.entry(node.scene_id).or_insert(DirtyBuffer::new());
						let instance_start = buffer.len() as u32;
						buffer.extend_from_slice(bytemuck::bytes_of(&instance));
						let instance_end = buffer.len() as u32;
					
						let checkpoint = checkpoints.entry(node.scene_id).or_insert(instance_start..instance_end);
						checkpoint.end = instance_end;
					}

					for (scene_id, instances) in checkpoints {
						let draw_calls = self.scene_draw_calls.entry(scene_id).or_insert(Vec::new());

						draw_calls.push(DrawCall {
							texture: mesh.texture,
							vertices: positions_start..positions_end,
							indices: indices_start..indices_end,
							normals: normals_start..normals_end,
							tex_coords: tex_coords_start..tex_coords_end,
							instances,
							indices_range: 0..primitive.indices.len() as u32,
						});
					}
				}
			}
		}
	}

	fn process_cameras(&mut self) {
		for (cam_id, cam) in &self.state.cameras {
			let cam_node = match cam.node_id {
				Some(id) => {
					match self.nodes.get(&id) {
						Some(node) => node,
						None => continue,
					}
				},
				None => continue,
			};

			let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar) 
				* cam_node.model.inverse();

			let cam = RawCamera {
				model: model.to_cols_array_2d(),
			};

			match self.cameras.get(&cam_id) {
				Some(camera) => {
					self.cameras.insert(cam_id, *camera);
				},
				None => {
					log::info!("new camera cam_id: {:?} camera: {:?}", cam_id, cam);
					self.cameras.insert(cam_id, cam);
				}
			}

			let start = self.camera_buffer.len() as u64;
			self.camera_buffer.extend_from_slice(bytemuck::bytes_of(&cam));
			let end = self.camera_buffer.len() as u64;

			self.all_cameras_data.insert(cam_id, bytes_of(&cam).to_vec());
			self.camera_pointers.insert(cam_id, start..end);
		}
	}

	fn process_point_lights(&mut self) {
		for (_, light) in &self.state.point_lights {
			let node = match light.node_id {
				Some(id) => {
					match self.nodes.get(&id) {
						Some(node) => node,
						None => continue,
					}
				}
				None => continue,
			};

			let light = RawPointLight {
				color: light.color.into(),
				intensity: light.intensity,
				position: node.model.w_axis.truncate().into(),
			};

			self.scene_lights.entry(node.scene_id).or_insert(DirtyBuffer::new()).extend_from_slice(bytes_of(&light));
		}
	}

	fn process_scenes(&mut self) {
		for (scene_id, scene) in &self.state.scenes {
			self.grids.entry(scene_id).or_insert(SpatialGrid::new(5.0));
		}
	}

	fn process_assets(&mut self) {
		let paths = self.state.assets_3d.iter().map(|p| p.1.path.clone()).collect::<Vec<String>>();

		for path in paths {
			if self.assets.contains(&path) {
				continue;
			}

			self.assets.insert(path.clone());

			load_gltf(&path, &mut self.state);
		}
	}

	// pub fn update_guis(&mut self) {
	// 	for (gui_id, gui) in &self.state.guis {
	// 		let compositor = self.ui_compositors.entry(gui_id).or_insert(UICompositor::new());draw_calls
	// 		compositor.process(gui);
	// 	}
	// }


	fn process_gui(&mut self) {
		for (ui_id, gui) in &self.state.guis {
			let compositor = self.ui_compositors.entry(ui_id).or_insert(UICompositor::new());
			compositor.process(gui);

			let render_args = self.ui_render_args.entry(ui_id).or_insert(UIRenderArgs {
				ui: ui_id,
				views: Vec::new(),
			});

			render_args.views.extend_from_slice(compositor.views_3d.as_slice());
		}
	}

	// fn process_windows(&mut self) {
	// 	for (window_id, window) in &self.state.windows {
	// 		let a = WindowRenderArgs {
	// 			ui: window.ui,
	// 			views: 
	// 		}
	// 	}
	// }

	fn process_phycis(&mut self, dt: f32) {
		for (_, c) in &mut self.scene_collections {
			let timings = c.physics_system.physics_update(&mut self.state, &mut c.grid, dt);
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
				let nodes = c.grid.get_line_ray_nodes(start, end);
			
				let mut intersections = Vec::new();
			
				for node_inx in nodes {
					if node_inx == ray_cast.node_inx {
						continue;
					}
			
					let aabb = match c.grid.get_node_rect(node_inx) {
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
	}


	pub fn process(&mut self, dt: f32) {
		let timer = Instant::now();
		self.all_instances_data.clear();
		// self.all_positions_data.clear();
		// self.all_tex_coords_data.clear();
		// self.all_normals_data.clear();
		// self.all_indices_data.clear();
		self.all_cameras_data.clear();
		self.all_point_lights_data.clear();
		self.triangles.clear();

		self.process_nodes();
		self.process_meshes();
		self.process_cameras();
		self.process_point_lights();
		self.process_scenes();
		self.process_phycis(dt);

		// for (node_inx, (node_id, node)) in self.state.nodes.iter().enumerate() {
		// 	let model = glam::Mat4::from_translation(node.translation)
		// 		* glam::Mat4::from_quat(node.rotation)
		// 		* glam::Mat4::from_scale(node.scale);
		// 	let raw_node = RawNode {
		// 		model: model.to_cols_array_2d(),
		// 		parent_index: -1,
		// 		_padding: [0; 3]
		// 	};

		// 	match self.nodes.get(&node_id) {
		// 		Some(old_node) => {
		// 			if let Some(collision_shape) = &node.collision_shape {
		// 				if old_node.translation != node.translation {
		// 					self.rem_nodes.insert(node_id);
		// 					self.add_nodes.push((node_id, collision_shape.aabb(node.translation)));
		// 				}
		// 			}

		// 			self.nodes.insert(node_id, node.clone());
		// 		},
		// 		None => {
		// 			log::info!("new node node_id: {:?} node: {:?}", node_id, node);
		// 			self.nodes.insert(node_id, node.clone());

		// 			if let Some(collision_mesh) = &node.collision_shape {
		// 				self.add_nodes.push((node_id, collision_mesh.aabb(node.translation)));
		// 			}
		// 		}
		// 	}

		// 	node_indexes.insert(node_id, node_inx as i32);
		// 	self.all_nodes_data.extend_from_slice(bytes_of(&raw_node));
			
		// 	if let Some(mesh_id) = node.mesh {
		// 		let instance = RawInstance {
		// 			node_index: node_inx as i32
		// 		};
		// 		match self.instances.get(&mesh_id) {
		// 			Some(instance) => {
		// 				self.instances.insert(mesh_id, *instance);
		// 			},
		// 			None => {
		// 				log::info!("new instance mesh_id: {:?} instance: {:?}", mesh_id, instance);
		// 				self.instances.insert(mesh_id, instance);
		// 			}
		// 		}

		// 		mesh_instances.entry(mesh_id).or_insert(Vec::new()).push(instance);
		// 	}
		// }



		// if self.rem_nodes.len() > 0 {
		// 	let rem_timer = Instant::now();
		// 	self.grid.rem_nodes(&self.rem_nodes);
		// 	self.printer.print(REM_NODE_SLOT, format!("rem nodes total time: {}ms", rem_timer.elapsed().as_millis()));
		// 	self.rem_nodes.clear();
		// }

		// if self.add_nodes.len() > 0 {
		// 	let add_timer = Instant::now();
		// 	for (node_id, aabb) in self.add_nodes.drain(..) {
		// 		self.grid.add_node(node_id, aabb);
		// 	}
		// 	self.printer.print(ADD_NODE_SLOT, format!("add nodes total time: {}ms", add_timer.elapsed().as_millis()));
		// }
	}

	pub fn get_window_render_args(&self, window_id: Index) -> Option<&UIRenderArgs> {
		self.ui_render_args.get(&window_id)
	}

	pub fn get_camera_draw_calls(&self, camera_id: Index) -> Option<&Vec<DrawCall>> {
		let camera = self.state.cameras.get(camera_id)?;
		let scene_id = match camera.node_id {
			Some(node_id) => {
				let node = self.nodes.get(&node_id)?;
				node.scene_id
			},
			None => return None,
		};
		self.scene_draw_calls.get(&scene_id)
	}
}
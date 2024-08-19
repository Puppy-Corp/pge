use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::time::Instant;

use bytemuck::bytes_of;
use glam::*;

use crate::arena::ArenaId;
use crate::buffer::DirtyBuffer;
use crate::compositor::UICompositor;
use crate::cube;
use crate::debug::ChangePrinter;
use crate::gltf::load_gltf;
use crate::internal_types::CamView;
use crate::physics::PhysicsSystem;
use crate::spatial_grid::SpatialGrid;
use crate::wgpu_types::*;
use crate::Camera;
use crate::GUIElement;
use crate::Mesh;
use crate::Model3D;
use crate::Node;
use crate::NodeParent;
use crate::PointLight;
use crate::PrimitiveTopology;
use crate::Scene;
use crate::State;
use crate::Texture;
use crate::Window;
use crate::AABB;

const REM_NODE_SLOT: u32 = 0;
const ADD_NODE_SLOT: u32 = 1;
const NODE_UPDATE_TIME_SLOT: u32 = 2;
const BROAD_PHASE_TIME_SLOT: u32 = 3;
const NARROW_PHASE_TIME_SLOT: u32 = 4;
const NODES_COUNT_SLOT: u32 = 5;
const MESHES_COUNT_SLOT: u32 = 6;
const CAMERAS_SLOT: u32 = 7;
const POINT_LIGHTS_COUNT_SLOT: u32 = 8;
const SCENES_COUNT_SLOT: u32 = 9;
const DRAW_CALLS_SLOT: u32 = 10;
const MESH_NODES_SLOT: u32 = 11;
const UI_RENDER_ARGS_SLOT: u32 = 12;

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
			vertices: DirtyBuffer::new("vertices"),
			normals: DirtyBuffer::new("normals"),
			tex_coords: DirtyBuffer::new("tex_coords"),
			indices: DirtyBuffer::new("indices"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct DrawCall {
	pub texture: Option<ArenaId<Texture>>,
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
	pub colors: Vec<[f32; 3]>,
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

#[derive(Debug, Clone)]
struct NodeMetadata {
	model: glam::Mat4,
	scene_id: ArenaId<Scene>,
}

#[derive(Debug, Clone)]
struct CollisionNode {
	node_id: ArenaId<Node>,
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
pub struct View {
	pub camview: CamView,
	pub scene_id: ArenaId<Scene>,
}

#[derive(Debug, Clone)]
pub struct UIRenderArgs {
	pub ui: ArenaId<GUIElement>,
	pub views: Vec<View>,
}

#[derive(Debug, Clone)]
pub struct SceneCollection {
	grid: SpatialGrid,
	physics_system: PhysicsSystem,
}

#[derive(Debug, Clone, Default)]
pub struct EngineState {
	pub state: State,
	grids: HashMap<ArenaId<Scene>, SpatialGrid>,
	nodes: HashMap<ArenaId<Node>, NodeMetadata>,
	cameras: HashMap<ArenaId<Camera>, RawCamera>,
	printer: ChangePrinter,
	pub all_instances_data: Vec<u8>,
	pub camera_buffers: HashMap<ArenaId<Camera>, DirtyBuffer>,
	pub all_point_lights_data: Vec<u8>,
	pub triangles: Gemometry,
	_3d_models: HashMap<String, Model3D>,
	pub ui_compositors: HashMap<ArenaId<GUIElement>, UICompositor>,
	ui_render_args: HashMap<ArenaId<GUIElement>, UIRenderArgs>,
	mesh_pointers: HashMap<ArenaId<Mesh>, MeshPointer>,
	mesh_nodes: HashMap<ArenaId<Mesh>, Vec<ArenaId<Node>>>,
	scene_draw_calls: HashMap<ArenaId<Scene>, Vec<DrawCall>>,
	pub scene_point_lights: HashMap<ArenaId<Scene>, DirtyBuffer>,
	pub scene_instance_buffers: HashMap<ArenaId<Scene>, DirtyBuffer>,
	scene_collections: HashMap<ArenaId<Scene>, SceneCollection>,
	point_lights: HashMap<ArenaId<PointLight>, RawPointLight>,
}

impl EngineState {
	pub fn new() -> Self {
		Default::default()
	}

	fn process_nodes(&mut self) {
		self.printer.print(
			NODES_COUNT_SLOT,
			format!("nodes count: {}", self.state.nodes.len()),
		);
		let mut processed_nodes: HashSet<ArenaId<Node>> = HashSet::new();
		for (_, nodes) in &mut self.mesh_nodes {
			nodes.clear();
		}

		for (node_id, node) in &self.state.nodes {
			if processed_nodes.contains(&node_id) {
				continue;
			}

			let mut stack = vec![node_id];

			while let Some(node_id) = stack.last() {
				let node_id = *node_id;

				let node = match self.state.nodes.get(&node_id) {
					Some(node) => node,
					None => {
						panic!("Node with ID {:?} not found", node_id);
					},
				};
	
				let node_metadata = match node.parent {
					NodeParent::Node(parent_node_id) => {
						match processed_nodes.contains(&parent_node_id) {
							true => {
								let parent = match self.nodes.get(&parent_node_id) {
									Some(model) => model,
									None => {
										stack.push(parent_node_id);
										continue;
									}
								};

								let model = node.model_matrix();
								let model = parent.model * model;

								NodeMetadata {
									model,
									scene_id: parent.scene_id,
								}
							}
							false => {
								stack.push(parent_node_id);
								continue;
							}
						}
					}
					NodeParent::Scene(scene_id) => {
						let model = node.model_matrix();
						NodeMetadata { scene_id, model }
					}
					NodeParent::Orphan => {
						processed_nodes.insert(node_id);
						break;
					}
				};

				if let Some(collision_shape) = &node.collision_shape {
					let modify = match self.nodes.get(&node_id) {
						Some(old) => {
							if old.model != node_metadata.model {
								true
							} else {
								false
							}
						}
						None => {
							true
						}
					};

					if modify {
						let aabb = collision_shape.aabb(node.translation);

						let collection = self.scene_collections.entry(node_metadata.scene_id).or_insert(SceneCollection {
							grid: SpatialGrid::new(5.0),
							physics_system: PhysicsSystem::new(),
						});

						collection.grid.set_node(node_id, aabb);
					}
				}

				self.nodes.insert(node_id, node_metadata);

				if let Some(mesh_id) = node.mesh {
					self.mesh_nodes
						.entry(mesh_id)
						.or_insert(Vec::new())
						.push(node_id);
				}

				stack.pop();
				processed_nodes.insert(node_id);
			}
		}

		for (_, c) in &mut self.scene_collections {
			c.grid.retain_nodes(|node_id| processed_nodes.contains(&node_id));
		}
	}

	fn process_meshes(&mut self) {
		// log::info!("meshes: {:?}", self.state.meshes.len());
		self.printer.print(
			MESHES_COUNT_SLOT,
			format!("meshes count: {}", self.state.meshes.len()),
		);
		self.triangles.vertices.reset_offset();
		self.triangles.normals.reset_offset();
		self.triangles.tex_coords.reset_offset();
		self.triangles.indices.reset_offset();
		for (_, s) in &mut self.scene_instance_buffers {
			s.reset_offset();
		}
		for (_, s) in &mut self.scene_draw_calls {
			s.clear();
		}
		for (mesh_id, mesh) in &self.state.meshes {
			for primitive in &mesh.primitives {
				if primitive.topology == PrimitiveTopology::TriangleList {
					if primitive.vertices.len() == 0 || primitive.indices.len() == 0 {
						continue;
					}

					let vertices_start = self.triangles.vertices.len() as u64;
					self.triangles.vertices.extend_from_slice(bytemuck::cast_slice(&primitive.vertices));
					let vertices_end = self.triangles.vertices.len() as u64;
					let normals_start = self.triangles.normals.len() as u64;
					self.triangles.normals.extend_from_slice(bytemuck::cast_slice(&primitive.normals));
					let normals_end = self.triangles.normals.len() as u64;
					let indices_start = self.triangles.indices.len() as u64;
					self.triangles.indices.extend_from_slice(bytemuck::cast_slice(&primitive.indices));
					let indices_end = self.triangles.indices.len() as u64;
					let tex_coords_start = self.triangles.tex_coords.len() as u64;
					if primitive.tex_coords.len() > 0 {
						self.triangles
							.tex_coords
							.extend_from_slice(bytemuck::cast_slice(&primitive.tex_coords));
					} else {
						let tex_coords = vec![[0.0, 0.0]; primitive.vertices.len()];
						self.triangles
							.tex_coords
							.extend_from_slice(bytemuck::cast_slice(&tex_coords));
					}
					let tex_coords_end = self.triangles.tex_coords.len() as u64;

					let pointer = MeshPointer {
						positions: vertices_start..vertices_end,
						normals: normals_start..normals_end,
						tex_coords: tex_coords_start..tex_coords_end,
						indices: indices_start..indices_end,
						indice_count: primitive.indices.len() as u32,
					};
					self.mesh_pointers.insert(mesh_id, pointer);

					let node_ids = match self.mesh_nodes.get(&mesh_id) {
						Some(nodes) => nodes,
						None => continue,
					};

					let mut checkpoints: HashMap<ArenaId<Scene>, Range<u32>> = HashMap::new();
					

					for node_id in node_ids {
						let node = match self.nodes.get(node_id) {
							Some(node) => node,
							None => continue,
						};

						let instance = RawInstance {
							model: node.model.to_cols_array_2d(),
						};

						let buffer = self
							.scene_instance_buffers
							.entry(node.scene_id)
							.or_insert(DirtyBuffer::new("instances"));

						let instance_start = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;
						buffer.extend_from_slice(bytemuck::bytes_of(&instance));
						let instance_end = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;

						let checkpoint = checkpoints
							.entry(node.scene_id)
							.or_insert(instance_start..instance_end);
						checkpoint.end = instance_end;
					}

					for (scene_id, instances) in checkpoints {
						let draw_calls =
							self.scene_draw_calls.entry(scene_id).or_insert(Vec::new());

						// log::info!("draw_calls: {:?}", draw_calls.len());

						draw_calls.push(DrawCall {
							texture: mesh.texture,
							vertices: vertices_start..vertices_end,
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

		// log::info!("scene_draw_calls: {:?}", self.scene_draw_calls.len());
		self.printer.print(
			DRAW_CALLS_SLOT,
			format!("scene draw calls: {:?}", self.scene_draw_calls),
		);
	}

	fn process_cameras(&mut self) {
		for (_, buf) in &mut self.camera_buffers {
			buf.reset_offset();
		}

		for (cam_id, cam) in &self.state.cameras {
			let cam_node = match cam.node_id {
				Some(id) => match self.nodes.get(&id) {
					Some(node) => node,
					None => continue,
				},
				None => continue,
			};

			// let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar)
			//     * cam_node.model;

			//log::info!("cam_node: {:?}", cam_node);

			//let can_model = Mat4::from_translation(Vec3::new(0.0, 0.0, 2.0));
			let can_model = cam_node.model;

			let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar)
				* can_model.inverse();

			let cam = RawCamera {
				model: model.to_cols_array_2d(),
			};

			match self.cameras.get(&cam_id) {
				Some(camera) => {
					self.cameras.insert(cam_id, *camera);
				}
				None => {
					log::info!("can_node: {:?}", cam_node);
					log::info!("model: {:?}", model);
					log::info!("new camera cam_id: {:?} camera: {:?}", cam_id, cam);
					self.cameras.insert(cam_id, cam);
				}
			}

			let buffer = self
				.camera_buffers
				.entry(cam_id)
				.or_insert(DirtyBuffer::new("cameras"));

			let camdata = Mat4::IDENTITY;

			buffer.extend_from_slice(bytemuck::bytes_of(&cam));
			//buffer.extend_from_slice(bytemuck::bytes_of(&camdata.to_cols_array_2d()));
		}

		self.printer
			.print(CAMERAS_SLOT, format!("cameras: {:?}", self.cameras));
	}

	fn process_point_lights(&mut self) {
		for (_, s) in &mut self.scene_point_lights {
			s.reset_offset();
		}

		for (light_id, light) in &self.state.point_lights {
			//log::info!("light: {:?}", light);

			let node = match light.node_id {
				Some(id) => {
					match self.nodes.get(&id) {
						Some(node) => node,
						None => {
							log::warn!("Node with ID {:?} not found for light {:?}", id, light_id);
							continue;
						}
					}
				}
				None => {
					log::warn!("Light {:?} has no associated node ID", light_id);
					continue;
				}
			};

			// let light = RawPointLight {
			// 	color: light.color.into(),
			// 	intensity: light.intensity,
			// 	position: node.model.w_axis.truncate().into(),

			// };

			let pos = node.model.w_axis.truncate().into();
			let light = RawPointLight::new(light.color, light.intensity, pos);

			match self.point_lights.get(&light_id) {
				Some(old_light) => {
					if old_light != &light {
						//log::info!("point light modified {:?} {:?}", light_id, light);
						self.point_lights.insert(light_id, light);
					}
				},
				None => {
					log::info!("new point light {:?} {:?}", light_id, light);
					self.point_lights.insert(light_id, light);
				}
			}

			self.scene_point_lights.entry(node.scene_id).or_insert(DirtyBuffer::new("pointlight")).extend_from_slice(bytes_of(&light));
		}
	}

	fn process_scenes(&mut self) {
		for (scene_id, scene) in &self.state.scenes {
			self.grids.entry(scene_id).or_insert(SpatialGrid::new(5.0));
		}
	}

	fn process_assets(&mut self) {
		// let paths = self
		//     .state
		//     .assets_3d
		//     .iter()
		//     .map(|p| p.1.path.clone())
		//     .collect::<Vec<String>>();

		// for path in paths {
		//     if self._3d_models.contains(&path) {
		//         continue;
		//     }

		//     self.assets_3d.insert(path.clone());

		//     load_gltf(&path, &mut self.state);
		// }
	}

	fn process_ui(&mut self) {
		for (ui_id, gui) in &self.state.guis {
			let compositor = self
				.ui_compositors
				.entry(ui_id)
				.or_insert(UICompositor::new());
			compositor.process(gui);

			let render_args = self.ui_render_args.entry(ui_id).or_insert(UIRenderArgs {
				ui: ui_id,
				views: Vec::new(),
			});

			render_args.views.clear();

			for view in &compositor.views {
				let camera = match self.state.cameras.get(&view.camera_id) {
					Some(camera) => camera,
					None => continue,
				};

				let camera_node = match camera.node_id {
					Some(node_id) => match self.nodes.get(&node_id) {
						Some(node) => node,
						None => continue,
					},
					None => continue,
				};

				render_args.views.push(View {
					camview: view.clone(),
					scene_id: camera_node.scene_id,
				});
			}
		}

		self.printer.print(
			UI_RENDER_ARGS_SLOT,
			format!("ui_render_args: {:?}", self.ui_render_args),
		);
	}

	fn process_phycis(&mut self, dt: f32) {
		// for (scene_id, scene) in &self.state.scenes {
		// 	self.scene_collections.entry(scene_id).or_insert(SceneCollection {
		// 		moved_nodes: Vec::new(),
		// 		grid: SpatialGrid::new(5.0),
		// 		physics_system: PhysicsSystem::new(),
		// 	});
		// }

		for (_, c) in &mut self.scene_collections {
			let timings = c
				.physics_system
				.physics_update(&mut self.state, &mut c.grid, dt);
			if timings.node_update_time > 3 {
				self.printer.print(
					NODE_UPDATE_TIME_SLOT,
					format!("node_update_time: {}", timings.node_update_time),
				);
			}
			if timings.broad_phase_time > 10 {
				self.printer.print(
					BROAD_PHASE_TIME_SLOT,
					format!("broad_phase_time: {}", timings.broad_phase_time),
				);
			}
			if timings.narrow_phase_time > 3 {
				self.printer.print(
					NARROW_PHASE_TIME_SLOT,
					format!("narrow_phase_time: {}", timings.narrow_phase_time),
				);
			}
			if timings.resolve_collision_time > 0 {
				self.printer.print(
					NARROW_PHASE_TIME_SLOT,
					format!("resolve_collision_time: {}", timings.resolve_collision_time),
				);
			}

			for (_, ray_cast) in &mut self.state.raycasts {
				ray_cast.intersects.clear();

				let node = match self.state.nodes.get(&ray_cast.node_id) {
					Some(node) => node,
					None => continue,
				};

				let start = node.translation;
				let end = start + node.rotation * Vec3::new(0.0, 0.0, 1.0) * ray_cast.len;
				let nodes = c.grid.get_line_ray_nodes(start, end);

				let mut intersections = Vec::new();

				for node_inx in nodes {
					if node_inx == ray_cast.node_id {
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
				ray_cast.intersects = intersections
					.into_iter()
					.map(|(_, node_inx)| node_inx)
					.collect();
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
		self.all_point_lights_data.clear();

		self.process_nodes();
		self.process_meshes();
		self.process_cameras();
		self.process_point_lights();
		self.process_ui();
		self.process_scenes();
		self.process_phycis(dt);
	}

	pub fn get_window_render_args(&self, window_id: ArenaId<Window>) -> Option<&UIRenderArgs> {
		let window = match self.state.windows.get(&window_id) {
			Some(window) => window,
			None => return None,
		};

		let ui_id = match window.ui {
			Some(ui_id) => ui_id,
			None => return None,
		};

		self.ui_render_args.get(&ui_id)
	}

	pub fn get_camera_draw_calls(&self, camera_id: ArenaId<Camera>) -> Option<&Vec<DrawCall>> {
		let camera = self.state.cameras.get(&camera_id)?;
		let scene_id = match camera.node_id {
			Some(node_id) => {
				let node = self.nodes.get(&node_id)?;
				node.scene_id
			}
			None => return None,
		};
		self.scene_draw_calls.get(&scene_id)
	}
}


#[cfg(test)]
mod tests {
	use crate::Node;

use super::*;

	#[test]
	fn transform_nodes() {
		let mut state = EngineState::new(); 
		let scene_id = state.state.scenes.insert(Scene::new());
		let mut node = Node::new();
		node.translation = Vec3::new(0.0, 1.0, 0.0);
		node.parent = NodeParent::Scene(scene_id);

		let node_id = state.state.nodes.insert(node);

		state.process_nodes();

		println!("{:#?}", state);
	}
}
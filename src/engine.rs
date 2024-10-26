use crate::buffer::Buffer;
use crate::compositor::Compositor;
use crate::hardware;
use crate::hardware::Hardware;
use crate::hardware::PipelineHandle;
use crate::hardware::RenderEncoder;
use crate::hardware::TextureHandle;
use crate::hardware::WindowHandle;
use crate::internal_types::*;
use crate::physics::PhysicsSystem;
use crate::spatial_grid::SpatialGrid;
use crate::state::State;
use crate::types::*;
use crate::Arena;
use crate::ArenaId;
use crate::GUIElement;
use crate::Window;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct DrawCall {
	pub texture: Option<TextureHandle>,
	pub vertices: Range<u64>,
	pub indices: Range<u64>,
	pub normals: Range<u64>,
	pub tex_coords: Range<u64>,
	pub instances: Range<u32>,
	pub indices_range: Range<u32>,
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

struct GuiBuffers {
    vertices_buffer: Buffer,
    index_buffer: Buffer,
    color_buffer: Buffer,
    position_range: Range<u64>,
    index_range: Range<u64>,
    colors_range: Range<u64>,
    indices_range: Range<u32>,
}

impl GuiBuffers {
    pub fn new(hardware: &mut impl Hardware) -> Self {
        let vertices_buffer = Buffer::new(hardware.create_buffer("gui_vertices"));
		let index_buffer = Buffer::new(hardware.create_buffer("gui_indices"));
		let color_buffer = Buffer::new(hardware.create_buffer("gui_colors"));
        Self {
            vertices_buffer,
            index_buffer,
            color_buffer,
            position_range: 0..0,
            index_range: 0..0,
            colors_range: 0..0,
            indices_range: 0..0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SceneCollection {
	grid: SpatialGrid,
	physics_system: PhysicsSystem,
}

struct WindowContext {
	window_id: ArenaId<Window>,
	window: WindowHandle,
	pipeline: PipelineHandle,
}

struct NodeComputedMetadata {
	model: glam::Mat4,
	scene_id: ArenaId<Scene>,
}

pub struct Engine<A, H> {
    app: A,
    state: State,
	grids: HashMap<ArenaId<Scene>, SpatialGrid>,
    hardware: H,
    vertices_buffer: Buffer,
    tex_coords_buffer: Buffer,
    normal_buffer: Buffer,	
    index_buffer: Buffer,
    point_light_buffers: HashMap<ArenaId<Scene>, Buffer>,
    gui_buffers: HashMap<ArenaId<GUIElement>, GuiBuffers>,
    camera_buffers: HashMap<ArenaId<Camera>, Buffer>,
    default_texture: TextureHandle,
	default_point_lights: Buffer,
    scene_instance_buffers: HashMap<ArenaId<Scene>, Buffer>,
    scene_draw_calls: HashMap<ArenaId<Scene>, Vec<DrawCall>>,
	textures: HashSet<ArenaId<Texture>>,
    surfaces: Arena<hardware::Surface>,
    ui_compositors: HashMap<ArenaId<GUIElement>, Compositor>,
    ui_render_args: HashMap<ArenaId<GUIElement>, UIRenderArgs>,
	windows: Vec<WindowContext>,
	scene_collections: HashMap<ArenaId<Scene>, SceneCollection>,
	nodes: HashMap<ArenaId<Node>, NodeComputedMetadata>,
	mesh_nodes: HashMap<ArenaId<Mesh>, Vec<ArenaId<Node>>>,
}

impl<A, H> Engine<A, H>
where
    A: App,
    H: Hardware,
{
    pub fn new(mut app: A, mut hardware: H) -> Self {
        let data: [u8; 4] = [255, 100, 200, 255]; // pink
        let default_texture = hardware.create_texture("default_texture", &data);

        let vertices_buffer = Buffer::new(hardware.create_buffer("vertices"));
        let tex_coords_buffer = Buffer::new(hardware.create_buffer("tex_coords"));
        let normal_buffer = Buffer::new(hardware.create_buffer("normals"));
        let index_buffer = Buffer::new(hardware.create_buffer("indices"));

		let default_point_lights = Buffer::new(hardware.create_buffer("default_point_lights"));

        let mut state = State::default();
        app.on_create(&mut state);

        Self {
            app,
            state,
			grids: HashMap::new(),
            hardware,
            vertices_buffer,
            tex_coords_buffer,
            normal_buffer,
            index_buffer,
            point_light_buffers: HashMap::new(),
            gui_buffers: HashMap::new(),
            camera_buffers: HashMap::new(),
            default_texture,
            scene_instance_buffers: HashMap::new(),
			default_point_lights,
			textures: HashSet::new(),
            surfaces: Arena::new(),
            ui_compositors: HashMap::new(),
            scene_draw_calls: HashMap::new(),
            ui_render_args: HashMap::new(),
			windows: Vec::new(),
			scene_collections: HashMap::new(),
			nodes: HashMap::new(),
			mesh_nodes: HashMap::new(),
        }
    }

	fn process_nodes(&mut self) {
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

								NodeComputedMetadata {
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
						NodeComputedMetadata { scene_id, model }
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
		for (_, s) in &mut self.scene_draw_calls {
			s.clear();
		}
        
		for (mesh_id, mesh) in &self.state.meshes {
			for primitive in &mesh.primitives {
				if primitive.topology == PrimitiveTopology::TriangleList {
					if primitive.vertices.len() == 0 || primitive.indices.len() == 0 {
						continue;
					}

					let vertices_start = self.vertices_buffer.len();
                    self.vertices_buffer.write(bytemuck::cast_slice(&primitive.vertices));
					let vertices_end = self.vertices_buffer.len();

					let normals_start = self.normal_buffer.len();
					self.normal_buffer.write(bytemuck::cast_slice(&primitive.normals));
					let normals_end = self.normal_buffer.len();

					let indices_start = self.index_buffer.len();
					self.index_buffer.write(bytemuck::cast_slice(&primitive.indices));
					let indices_end = self.index_buffer.len();

					let tex_coords_start = self.tex_coords_buffer.len();
					if primitive.tex_coords.len() > 0 {
                        self.tex_coords_buffer.write(bytemuck::cast_slice(&primitive.tex_coords));
					} else {
						let tex_coords = vec![[0.0, 0.0]; primitive.vertices.len()];
						self.tex_coords_buffer.write(bytemuck::cast_slice(&tex_coords));
					}
					let tex_coords_end = self.tex_coords_buffer.len();
					let node_ids = match self.mesh_nodes.get(&mesh_id) {
						Some(ids) => ids,
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
						let buffer = self.scene_instance_buffers.entry(node.scene_id)
							.or_insert_with(|| Buffer::new(self.hardware.create_buffer(&format!("instances_{:?}", node.scene_id.index()))));

						let instance_start = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;
						buffer.write(bytemuck::bytes_of(&instance));
						let instance_end = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;

						let checkpoint = checkpoints
							.entry(node.scene_id)
							.or_insert(instance_start..instance_end);
						checkpoint.end = instance_end;
					}

					for (scene_id, instances) in checkpoints {
						let draw_calls =
							self.scene_draw_calls.entry(scene_id).or_insert(Vec::new());
						draw_calls.push(DrawCall {
							texture: None, // TODO: add texture
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
		self.vertices_buffer.flush(&mut self.hardware);
		self.tex_coords_buffer.flush(&mut self.hardware);
		self.normal_buffer.flush(&mut self.hardware);
		self.index_buffer.flush(&mut self.hardware);
		for (_, buffer) in &mut self.scene_instance_buffers {
			buffer.flush(&mut self.hardware);
		}
    }

    fn process_cameras(&mut self) {
		for (cam_id, cam) in &self.state.cameras {
			let node_id = match cam.node_id {
				Some(id) => id,
				None => continue,
			};
			let node = match self.nodes.get(&node_id) {
				Some(node) => node,
				None => continue,
			};
			let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar)
				* node.model.inverse();

			let cam = RawCamera {
				model: model.to_cols_array_2d(),
			};
			let buffer = self
				.camera_buffers
				.entry(cam_id)
				.or_insert_with(|| Buffer::new(self.hardware.create_buffer(&format!("camera_buffer_{:?}", cam_id.index()))));
			buffer.write(bytemuck::bytes_of(&cam));
		}
		for (_, buffer) in &mut self.camera_buffers {
			buffer.flush(&mut self.hardware);
		}
	}

    fn process_point_lights(&mut self) {
		for (_, light) in &self.state.point_lights {
            let node_id = match light.node_id {
                Some(id) => id,
                None => continue,
            };
			let node = match self.nodes.get(&node_id) {
				Some(node) => node,
				None => continue,
			};
			let scene_id = node.scene_id;
			let model = node.model;
			let pos = model.w_axis.truncate().into();
			let light = RawPointLight::new(light.color, light.intensity, pos);

			self.point_light_buffers.entry(scene_id).or_insert_with(|| {
				log::info!("Creating new point light buffer for scene ID: {:?}", scene_id);
				Buffer::new(self.hardware.create_buffer("pointlight"))
			}).write(bytemuck::bytes_of(&light));
		}
		for (_, buffer) in &mut self.point_light_buffers {
			buffer.flush(&mut self.hardware);
		}
	}

    fn process_ui(&mut self) {
		for (ui_id, gui) in &self.state.guis {
			let c: &mut Compositor = self
				.ui_compositors
				.entry(ui_id.clone())
				.or_insert_with(|| {
					log::info!("Creating new Compositor for UI ID: {:?}", ui_id); // Debug print
					Compositor::new()
				});
			c.process(gui);
	
			let buffers = self
				.gui_buffers
				.entry(ui_id)
				.or_insert_with(|| {
					log::info!("Creating new GuiBuffers for UI ID: {:?}", ui_id); // Debug print
					GuiBuffers::new(&mut self.hardware)
				});

            if c.positions.len() > 0 {
                let positions_data = bytemuck::cast_slice(&c.positions);
                let positions_data_len = positions_data.len() as u64;
                buffers.vertices_buffer.write(positions_data);
                buffers.position_range = 0..positions_data_len;
            }

            if c.indices.len() > 0 {
                let indices_data = bytemuck::cast_slice(&c.indices);
                let indices_data_len = indices_data.len() as u64;
                buffers.index_buffer.write(indices_data);
                buffers.index_range = 0..indices_data_len;
                buffers.indices_range = 0..c.indices.len() as u32;
            }

            if c.colors.len() > 0 {
                let colors_data = bytemuck::cast_slice(&c.colors);
                let colors_data_len = colors_data.len() as u64;
                buffers.color_buffer.write(colors_data);
                buffers.colors_range = 0..colors_data_len;
            }
			let render_args = self.ui_render_args.entry(ui_id.clone()).or_insert(UIRenderArgs {
				ui: ui_id.clone(),
				views: Vec::new(),
			});
			render_args.views.clear();
			for view in &c.views {
				let camera = match self.state.cameras.get(&view.camera_id) {
					Some(camera) => camera,
					None => continue,
				};
				let node_id = match &camera.node_id {
					Some(node_id) => node_id,
					None => continue,
				};
				let node = match self.nodes.get(node_id) {
					Some(node) => node,
					None => continue,
				};
				render_args.views.push(View {
					camview: view.clone(),
					scene_id: node.scene_id,
				});
			}
		}

		for (_, buffer) in &mut self.gui_buffers {
			buffer.vertices_buffer.flush(&mut self.hardware);
			buffer.index_buffer.flush(&mut self.hardware);
			buffer.color_buffer.flush(&mut self.hardware);
		}
	}

    fn get_window_render_args(&self, window_id: ArenaId<Window>) -> Option<&UIRenderArgs> {
		let window = match self.state.windows.get(&window_id) {
			Some(window) => window,
			None => return None,
		};

		let ui_id = match &window.ui {
			Some(ui_id) => ui_id,
			None => return None,
		};

		self.ui_render_args.get(&ui_id)
	}

	fn get_camera_draw_calls(&self, camera_id: ArenaId<Camera>) -> Option<&Vec<DrawCall>> {
		let camera = self.state.cameras.get(&camera_id)?;
		let scene_id = match &camera.node_id {
			Some(node_id) => {
				let node = match self.nodes.get(node_id) {
					Some(node) => node,
					None => return None,
				};
				node.scene_id
			}
			None => return None,
		};
		self.scene_draw_calls.get(&scene_id)
	}

    fn update_windows(&mut self) {
        for (window_id, window) in self.state.windows.iter_mut() {
			if self.windows.iter().any(|w| w.window_id == window_id) {
				continue;
			}
            let handle = self.hardware.create_window(&window);
			let pipeline = self.hardware.create_pipeline("pipeline", handle);
			self.windows.push(WindowContext {
				window_id,
				window: handle,
				pipeline,
			});
        }

        /*for (window_id, _) in self.prev_state.windows.iter() {
            if !self.state.windows.contains(&window_id) {
                //self.hardware.destroy_window(window_id);
            }
        }*/
    }

	fn process_scenes(&mut self) {
		for (scene_id, scene) in &self.state.scenes {
			self.grids.entry(scene_id).or_insert_with(|| SpatialGrid::new(5.0));
		}
	}

	fn process_physics(&mut self, dt: f32) {
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

			for (_, ray_cast) in &mut self.state.raycasts {
				ray_cast.intersects.clear();

				let node = match self.state.nodes.get(&ray_cast.node_id) {
					Some(node) => node,
					None => continue,
				};

				let start = node.translation;
				let end = start + node.rotation * glam::Vec3::new(0.0, 0.0, 1.0) * ray_cast.len;
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

    pub fn on_mouse_input(&mut self, window: WindowHandle, event: MouseEvent) {
		self.app.on_mouse_input(event, &mut self.state);
    }

    pub fn on_keyboard_input(&mut self, window: WindowHandle, key: KeyboardKey, action: KeyAction) {
		log::info!("Keyboard input: {:?} {:?}", key, action);
		self.app.on_keyboard_input(key, action, &mut self.state);
    }

    pub fn render(&mut self, dt: f32) {
		self.process_nodes();
		self.process_meshes();
		self.process_cameras();
		self.process_point_lights();
		self.process_ui();
		self.update_windows();
		self.process_scenes();
		self.process_physics(dt);
		self.app.on_process(&mut self.state, dt);
        for (window_id, _) in &self.state.windows {
			let ctx = self.windows.iter().find(|w| w.window_id == window_id).unwrap();
            let mut encoder = RenderEncoder::new();
            let args = match self.get_window_render_args(window_id) {
                Some(a) => a,
                None => {
                    panic!("Window render args not found");
                }
            };

            let pass = encoder.begin_render_pass();
			pass.set_pipeline(ctx.pipeline);
            for v in &args.views {
                let camera_buffer = match self.camera_buffers.get(&v.camview.camera_id) {
                    Some(b) => b,
                    None => {
                        panic!("Camera buffer not found");
                    }
                };

                let calls = match self.get_camera_draw_calls(v.camview.camera_id) {
                    Some(c) => c,
                    None => {
                        //panic!("Draw calls not found");
						continue;
                    }
                };

                let instance_buffer = match self.scene_instance_buffers.get(&v.scene_id) {
                    Some(b) => b,
                    None => {
                        panic!("Instance buffer not found");
                    }
                };

                let point_light_buffer = match self.point_light_buffers.get(&v.scene_id) {
                    Some(b) => b,
                    None => {
						&self.default_point_lights
					}
                };
                
                pass.bind_buffer(0, camera_buffer.handle);
                pass.bind_buffer(1, point_light_buffer.handle);

                for call in calls {
                    let texture = match call.texture {
                        Some(t) => t,
                        None => self.default_texture
                    };
                    pass.bind_texture(2, self.default_texture);
                    pass.set_vertex_buffer(0, self.vertices_buffer.slice(call.vertices.clone()));
                    pass.set_vertex_buffer(1, instance_buffer.full());
                    pass.set_vertex_buffer(2, self.normal_buffer.slice(call.normals.clone()));
                    pass.set_vertex_buffer(3, self.tex_coords_buffer.slice(call.tex_coords.clone()));
                    pass.set_index_buffer(self.index_buffer.slice(call.indices.clone()));
                    let indices = call.indices.clone();
                    let instances = call.instances.clone();
                    pass.draw_indexed(call.indices_range.clone(), instances.start as u32..instances.end as u32);
                }
            }
            self.hardware.render(encoder, ctx.window);
        }
    }
}
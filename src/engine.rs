use crate::buffer::Buffer;
use crate::compositor::Compositor;
use crate::hardware;
use crate::hardware::Hardware;
use crate::hardware::PipelineHandle;
use crate::hardware::RenderEncoder;
use crate::hardware::TextureHandle;
use crate::hardware::WindowHandle;
use crate::internal_types::*;
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

struct WindowContext {
	window_id: ArenaId<Window>,
	window: WindowHandle,
	pipeline: PipelineHandle,
}

pub struct Engine<A, H> {
    app: A,
    prev_state: State,
    state: State,
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
            prev_state: State::default(),
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

					let node_ids = self.state.get_mesh_nodes(mesh_id);

					let mut checkpoints: HashMap<ArenaId<Scene>, Range<u32>> = HashMap::new();
					

					for node_id in node_ids {
                        let transformation = self.state.get_node_transformation(node_id);
						let instance = RawInstance {
							model: transformation.to_cols_array_2d(),
						};
                        let scene_id = match self.state.get_node_scene(node_id) {
							Some(id) => id,
							None => continue,
						};

						let buffer = self.scene_instance_buffers.entry(scene_id)
							.or_insert_with(|| Buffer::new(self.hardware.create_buffer(&format!("instances_{:?}", scene_id))));

						let instance_start = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;
						buffer.write(bytemuck::bytes_of(&instance));
						let instance_end = buffer.len() as u32 / std::mem::size_of::<RawInstance>() as u32;

						let checkpoint = checkpoints
							.entry(scene_id)
							.or_insert(instance_start..instance_end);
						checkpoint.end = instance_end;
					}

					for (scene_id, instances) in checkpoints {
						let draw_calls =
							self.scene_draw_calls.entry(scene_id).or_insert(Vec::new());

						// log::info!("draw_calls: {:?}", draw_calls.len());

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
			let transformation = self.state.get_node_transformation(node_id);
			let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar)
				* transformation.inverse();

			let cam = RawCamera {
				model: model.to_cols_array_2d(),
			};

			let buffer = self
				.camera_buffers
				.entry(cam_id)
				.or_insert_with(|| Buffer::new(self.hardware.create_buffer(&format!("camera_buffer_{:?}", cam_id))));
			buffer.write(bytemuck::bytes_of(&cam));
		}
		for (_, buffer) in &mut self.camera_buffers {
			buffer.flush(&mut self.hardware);
		}
	}

    fn process_point_lights(&mut self) {
		for (light_id, light) in &self.state.point_lights {
            let node_id = match light.node_id {
                Some(id) => id,
                None => continue,
            };
            let scene_id = self.state.get_node_scene(node_id).unwrap();
            let transformation = self.state.get_node_transformation(node_id);
			let pos = transformation.w_axis.truncate().into();
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
                let scene_id = self.state.get_node_scene(node_id.clone()).unwrap();

				render_args.views.push(View {
					camview: view.clone(),
					scene_id,
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
			Some(node_id) => self.state.get_node_scene(node_id.clone()).unwrap(),
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

        for (window_id, _) in self.prev_state.windows.iter() {
            if !self.state.windows.contains(&window_id) {
                //self.hardware.destroy_window(window_id);
            }
        }

        /*match self
        .windows
        .values()
        .find(|window| window.window_id == window_id)
    {
        Some(w) => {
            if w.wininit_window.title() != window.title {
                w.wininit_window.set_title(&window.title);
            }
        }
        None => {
            let window_attributes =
                winit::window::Window::default_attributes().with_title(&window.title);
            let wininit_window = event_loop.create_window(window_attributes).unwrap();
            let wininit_window = Arc::new(wininit_window);
            let surface = Arc::new(self.instance.create_surface(wininit_window.clone()).unwrap());
            let pipeline = self.hardware.create_pipeline("pipeline", surface.clone(), wininit_window.inner_size());
            let wininit_window_id = wininit_window.id();
            let window_ctx = WindowContext {
                surface,
                window_id: window_id,
                wininit_window,
                pipeline,
                };
                self.windows.insert(wininit_window_id, window_ctx);
            }
        }*/
    }

    pub fn on_mouse_button_event(&mut self, window: WindowHandle, button: MouseButton, state: bool) {

    }

    pub fn on_cursor_moved(&mut self, window: WindowHandle, dx: f32, dy: f32) {

    }

    pub fn on_keyboard_input(&mut self, window_id: ArenaId<Window>, event: KeyboardKey, state: bool) {

    }

    fn does_window_exist(&self, arena_id: ArenaId<Window>) -> bool {
        false
    }

    pub fn render(&mut self, dt: f32) {
		self.state.prepare_cache();
		self.process_meshes();
		self.process_cameras();
		self.process_point_lights();
		self.process_ui();
		self.update_windows();
		
        for (window_id, _) in &self.state.windows {
			let ctx = self.windows.iter().find(|w| w.window_id == window_id).unwrap();
            let mut encoder = RenderEncoder::new();
            let args = match self.get_window_render_args(window_id) {
                Some(a) => a,
                None => {
                    //log::error!("Window render args not found");
					continue;
                }
            };

            let pass = encoder.begin_render_pass();
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
						log::info!("point light buffer not found");
						&self.default_point_lights
					}
                };

                pass.set_pipeline(ctx.pipeline);
                pass.bind_buffer(0, camera_buffer.clone());
                pass.bind_buffer(1, point_light_buffer.clone());

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
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use bytemuck::bytes_of;
use thunderdome::Index;
use wgpu::Backends;
use wgpu::Features;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::keyboard::Key;
use winit::keyboard::KeyCode;
use winit::window::WindowId;
use crate::buffer::*;
use crate::buffers::*;
use crate::cube;
use crate::physics::node_physics_update;
use crate::types::*;
use crate::renderer::*;
use crate::wgpu_types::*;

pub async fn run<T>(app: T) -> anyhow::Result<()>
where
	T: App
{
	let mut engine = Engine::new(app).await;
	let event_loop = EventLoop::new()?;
	Ok(event_loop.run_app(&mut engine)?)
}

#[derive(Debug)]
struct WindowContext<'a> {
	window_id: Index,
	renderer: Renderer<'a>,
	wininit_window: Arc<winit::window::Window>
}

struct Engine<'a, T> {
	i: usize,
	app: T,
	state: State,
	adapter: Arc<wgpu::Adapter>,
	instance: Arc<wgpu::Instance>,
	queue: Arc<wgpu::Queue>,
	device: Arc<wgpu::Device>,
	position_buffer: wgpu::Buffer,
	normal_buffer: wgpu::Buffer,
	tex_coord_buffer: DynamicVertexBuffer,
	index_buffer: wgpu::Buffer,
	draw_instructions: Vec<DrawInstruction>,
	// instaces: Arena<RawInstance>,
	windows: HashMap<WindowId, WindowContext<'a>>,
	instance_buffer: wgpu::Buffer,
	camera_buffer: FixedBuffer<RawCamera>,
	node_buffer: FixedBuffer<RawNode>,
	point_light_buffer: FixedBuffer<RawPointLight>,
	cameras: HashMap<Index, RawCamera>,
	instances: HashMap<Index, RawInstance>,
	nodes: HashMap<Index, RawNode>,
	meshes: HashSet<Index>,
	draw_instructions2: HashSet<Index>,
	last_on_process_time: Instant,
	last_physics_update_time: Instant
}

impl<'a, T> Engine<'a, T>
where
	T: App
{
	pub async fn new(app: T) -> Self {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let adapters = instance.enumerate_adapters(Backends::all());
		for adapter in adapters {
			println!("Adapter: {:?}", adapter.get_info());
		}
		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await.expect("Failed to find an appropriate adapter");
		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
			required_features: Features::VERTEX_WRITABLE_STORAGE,
			..Default::default()
		}, None)
			.await.expect("Failed to create device");
		
		let device = Arc::new(device);
		let queue = Arc::new(queue);
		let adapter = Arc::new(adapter);
		let instance = Arc::new(instance);

		let position_buffer = RawPositions::create_buffer(&device, 10_000);
		let normal_buffer = RawNormal::create_buffer(&device, 10_000);
		let tex_coord_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
		let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Index Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});
		let instance_buffer = RawInstance::create_buffer(&device, 10_000);
		let camera_buffer = FixedBuffer::new(device.clone(), queue.clone());
		let node_buffer = FixedBuffer::new(device.clone(), queue.clone());
		let point_light_buffer = FixedBuffer::new(device.clone(), queue.clone());

		Self {
			i: 0,
			app,
			state: State::default(),
			adapter,
			instance,
			queue,
			device,
			draw_instructions: Vec::new(),
			position_buffer,
			normal_buffer,
			tex_coord_buffer,
			index_buffer,
			instance_buffer,
			windows: HashMap::new(),
			camera_buffer,
			node_buffer,
			point_light_buffer,
			cameras: HashMap::new(),
			nodes: HashMap::new(),
			instances: HashMap::new(),
			meshes: HashSet::new(),
			draw_instructions2: HashSet::new(),
			last_on_process_time: Instant::now(),
			last_physics_update_time: Instant::now()
		}
	}

	pub fn update_buffers(&mut self) {
		self.app.on_process(&mut self.state, self.last_on_process_time.elapsed().as_secs_f32());
		self.last_on_process_time = Instant::now();
		
		self.draw_instructions.clear();
		let mut all_node_data = Vec::new();
		let mut all_instance_data: Vec<u8> = Vec::new();
		let mut all_position_data: Vec<u8> = Vec::new();
		let mut all_indices_data: Vec<u8> = Vec::new();
		let mut all_normal_data: Vec<u8> = Vec::new();

		let mut mesh_instances: HashMap<Index, Vec<RawInstance>> = HashMap::new();
		let mut node_indexes: HashMap<Index, i32> = HashMap::new();

		for (node_inx, (node_id, node)) in self.state.nodes.iter().enumerate() {
			let model = glam::Mat4::from_quat(node.rotation) * glam::Mat4::from_translation(node.translation) * glam::Mat4::from_scale(node.scale);
			let raw_node = RawNode {
				model: model.to_cols_array_2d(),
				parent_index: -1,
				_padding: [0; 3]
			};

			match self.nodes.get(&node_id) {
				Some(node) => {
					self.nodes.insert(node_id, *node);
				},
				None => {
					println!("new nodex_ix: {}  node_id: {:?} node: {:?}", node_inx, node_id, raw_node);
					self.nodes.insert(node_id, raw_node);
				}
			}

			node_indexes.insert(node_id, node_inx as i32);
			all_node_data.extend_from_slice(bytes_of(&raw_node));
			
			if let Some(mesh_id) = node.mesh {
				let instance = RawInstance {
					node_index: node_inx as i32
				};

				match self.instances.get(&mesh_id) {
					Some(instance) => {
						self.instances.insert(mesh_id, *instance);
					},
					None => {
						println!("new instance mesh_id: {:?} instance: {:?}", mesh_id, instance);
						self.instances.insert(mesh_id, instance);
					}
				}

				mesh_instances.entry(mesh_id).or_insert(Vec::new()).push(instance);
			}
		}

		for (mesh_id, mesh) in &self.state.meshes {
			// println!("mesh_id {:?}", mesh_id);
			let positions_start = all_position_data.len() as u64;
			all_position_data.extend_from_slice(bytemuck::cast_slice(&mesh.positions));
			let positions_end = all_position_data.len() as u64;
			let indices_start = all_indices_data.len() as u64;
			all_indices_data.extend_from_slice(bytemuck::cast_slice(&mesh.indices));
			let indices_end = all_indices_data.len() as u64;
			let normals_start = all_normal_data.len() as u64;
			all_normal_data.extend_from_slice(bytemuck::cast_slice(&mesh.normals));
			let normals_end = all_normal_data.len() as u64;

			let instances: &Vec<RawInstance> = match mesh_instances.get(&mesh_id) {
				Some(instances) => instances,
				None => continue,
			};

			let instance_start = all_instance_data.len() as u32;
			all_instance_data.extend_from_slice(bytemuck::cast_slice(instances));
			let instance_end = all_instance_data.len() as u32;

			let draw_instruction = DrawInstruction {
				position_range: positions_start..positions_end,
				normal_range: normals_start..normals_end,
				index_range: indices_start..indices_end,
				indices_range: 0..mesh.indices.len() as u32,
				instances_range: instance_start..instance_end
			};

			match self.meshes.contains(&mesh_id) {
				true => {},
				false => {
					println!("new mesh mesh_id: {:?} mesh: {:?}", mesh_id, mesh);
					println!("draw_instruction: {:?}", draw_instruction);
					println!("instances: {:?}", instances);
					self.meshes.insert(mesh_id);
				},
			}

			self.draw_instructions.push(draw_instruction);
		}

		let mut all_camera_data: Vec<u8> = Vec::new();
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
					println!("new camera cam_id: {:?} camera: {:?} node_inx: {}", cam_id, cam, node_inx);
					self.cameras.insert(cam_id, cam);
				}
			}

			all_camera_data.extend_from_slice(bytes_of(&cam));
		}

		if all_instance_data.len() > 0 {
			self.queue.write_buffer(&self.instance_buffer, 0, &all_instance_data);
		}
		if all_position_data.len() > 0 {
			self.queue.write_buffer(&self.position_buffer, 0, &all_position_data);
		}
		if all_normal_data.len() > 0 {
			self.queue.write_buffer(&self.normal_buffer, 0, &all_normal_data);
		}
		if all_indices_data.len() > 0 {
			self.queue.write_buffer(&self.index_buffer, 0, &all_indices_data);
		}
		if all_node_data.len() > 0 {
			self.queue.write_buffer(&self.node_buffer.buffer(), 0, &all_node_data);
		}
		if all_camera_data.len() > 0 {
			self.queue.write_buffer(&self.camera_buffer.buffer(), 0, &all_camera_data);
		}
	}

	fn update_physics(&mut self) {
		let dt = self.last_physics_update_time.elapsed().as_secs_f32();
		for (_, node) in &mut self.state.nodes {
			if node.physics.typ == PhycisObjectType::Dynamic {
				node_physics_update(node, dt)
			}
		}
		self.last_physics_update_time = Instant::now();
	}

	fn update_windows(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		for (window_id, window) in self.state.windows.iter_mut() {
			match self.windows.values().find(|window| window.window_id == window_id) {
				Some(w) => {
					if w.wininit_window.title() != window.title {
						w.wininit_window.set_title(&window.title);
					}
				},
				None => {
					let window_attributes = winit::window::Window::default_attributes()
						.with_title(&window.title);
					let wininit_window = event_loop.create_window(window_attributes).unwrap();
					let wininit_window = Arc::new(wininit_window);
					let renderer = Renderer::new(NewRendererArgs {
						window: wininit_window.clone(),
						instance: self.instance.clone(),
						adapter: self.adapter.clone(),
						queue: self.queue.clone(),
						device: self.device.clone()
					}).unwrap();

					let wininit_window_id = wininit_window.id();
					let window_ctx = WindowContext {
						window_id: window_id,
						wininit_window,
						renderer
					};
					self.windows.insert(wininit_window_id, window_ctx);
				},
			}
		}

		self.windows.retain(|_, w| self.state.windows.contains(w.window_id));

	}

	fn render_windows(&mut self) {
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		for (_, window_ctx) in self.windows.iter_mut() {
			let window = match self.state.windows.get(window_ctx.window_id) {
				Some(w) => w,
				None => continue,
			};

			let camera_id = match window.cam {
				Some(cam) => cam,
				None => continue,
			};

			let camera = match self.state.cameras.get(camera_id) {
				Some(cam) => cam,
				None => continue,
			};

			window_ctx.renderer.render(RenderArgs {
				node_buffer: &self.node_buffer,
				camera_buffer: &self.camera_buffer,
				point_light_buffer: &self.point_light_buffer,
				instance_buffer: &self.instance_buffer,
				index_buffer: &self.index_buffer,
				normal_buffer: &self.normal_buffer,
				positions_buffer: &self.position_buffer,
				encoder: &mut encoder,
				instructions: &mut self.draw_instructions.iter()
			}).unwrap();
		}
		self.queue.submit(std::iter::once(encoder.finish()));
	}
}

impl<'a, T> ApplicationHandler for Engine<'a, T>
where
	T: App
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		self.app.on_create(&mut self.state);
		self.update_windows(event_loop);
	}

	fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		// println!("about to wait {}", self.i);
		self.i += 1;

		let timer = Instant::now();
		let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
		event_loop
			.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)));

		// self.render_every_window();

		self.update_physics();
		self.update_buffers();
		self.render_windows();
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: winit::event::WindowEvent,
	) {
		// println!("window event");

		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				println!("redraw requested for window {:?}", window_id);
				match self.windows.get(&window_id) {
					Some(window) => {
						// let renderer = &window.renderer;
						// let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
						// 	label: Some("Render Encoder")
						// });
						// let args = RenderArgs {
						// 	encoder: &mut encoder,
						// 	camera_bind_group: &self.camera_buffer.bind_group(),
						// 	node_bind_group: &self.node_buffer.bind_group(),
						// 	positions_buffer: &self.position_buffer.buffer(),
						// 	index_buffer: &self.index_buffer.buffer(),
						// 	instance_buffer: &self.instance_buffer.buffer(),
						// 	instructions: &self.draw_instructions
						// };
						// match renderer.render(args) {
						// 	Ok(_) => {}
						// 	Err(err) => {
						// 		log::error!("Error rendering: {:?} window {:?}", err, window_id);
						// 	}
						// }
						// self.queue.submit(std::iter::once(encoder.finish()));
					}
					None => {
						log::error!("Window not found: {:?}", window_id);
					}
				}
			}
			WindowEvent::CursorMoved { device_id, position } => {
				if let Some(window_ctx) = self.windows.get(&window_id) {
					let size = &window_ctx.wininit_window.inner_size();
					let middle_x = size.width as f64 / 2.0;
					let middle_y = size.height as f64 / 2.0;
					let dx = position.x - middle_x;
					let dy = position.y - middle_y;
					let dx = dx as f32;
					let dy = dy as f32;
					self.app.on_mouse_input(MouseEvent::Moved { dx, dy }, &mut self.state);

					if let Some(window) = self.state.windows.get(window_ctx.window_id) {
						if window.lock_cursor {
							window_ctx.wininit_window.set_cursor_position(PhysicalPosition::new(middle_x, middle_y)).unwrap();
						}
					}
				}
			}
			WindowEvent::KeyboardInput { device_id, event, is_synthetic } => {
				match event {
					winit::event::KeyEvent {
						state,
						location,
						physical_key,
						repeat,
						..
					} => {
						// println!("keyboard input: {:?}", virtual_keycode);
						if !repeat {
							match physical_key {
								winit::keyboard::PhysicalKey::Code(code) => {
									if KeyCode::Escape == code {
										event_loop.exit();
									}

									match state {
										winit::event::ElementState::Pressed => self.app.on_keyboard_input(KeyboardKey::from(code), KeyAction::Pressed, &mut self.state),
										winit::event::ElementState::Released => self.app.on_keyboard_input(KeyboardKey::from(code), KeyAction::Released, &mut self.state),
									}
								},
								winit::keyboard::PhysicalKey::Unidentified(_) => {},
							}
						}
							
					}
				}
			}
			_ => {}
		}
	}
}

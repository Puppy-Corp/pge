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
use winit::window::WindowId;
use crate::buffer::*;
use crate::buffers::*;
use crate::cube;
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
	last_on_process_time: Instant
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
			last_on_process_time: Instant::now()
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






























// pub struct Engine {
// 	eventloop: EventLoop<Command>,
// 	tx: broadcast::Sender<Event>
// }

// impl Engine {
// 	pub fn new<F, Fut>(f: F) -> Self 
// 	where
// 		F: FnOnce(EngineHandle) -> Fut + Send + 'static,
// 		Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
// 	{
// 		let eventloop = EventLoop::<Command>::with_user_event().build().unwrap();
// 		let proxy = eventloop.create_proxy();
// 		// let (tx, rx) = mpsc::unbounded_channel();
// 		let (tx, _) = broadcast::channel(100);
// 		let handle = EngineHandle::new(proxy, tx.clone());
// 		tokio::spawn(async move {
// 			f(handle).await.unwrap();
// 		});

// 		Self {
// 			eventloop,
// 			tx
// 		}
// 	}

// 	pub async fn run(self) -> anyhow::Result<()> {
// 		let mut handler = EngineHandler::new(self.tx).await;
// 		Ok(self.eventloop.run_app(&mut handler)?)
// 	}
// }




// pub struct EngineHandler<'a> {
// 	windows: HashMap<WindowId, WindowContext<'a>>,
// 	device: Arc<wgpu::Device>,
// 	queue: Arc<wgpu::Queue>,
// 	adapter: Arc<wgpu::Adapter>,
// 	instance: Arc<wgpu::Instance>,
	
// 	time_buffer: wgpu::Buffer,
// 	key_frames_buffer: wgpu::Buffer,
// 	draw_instructions: Vec<DrawInstruction>,
// 	tx: broadcast::Sender<Event>,
// 	i: usize,
// 	since_last_frame: Instant,
// 	animation_pipeline: AnimationPipeline,
// 	animation_buffer: StaticBufferManager<RawAnimation>,
// 	keyframe_buffer: StaticBufferManager<RawKeyFrame>,
// 	node_transformations: TransformationAcumalator,
// 	node_transformation_buffer: StaticBufferManager<NodeTransformation>,
// 	position_buffer: DynamicVertexBuffer,
// 	normal_buffer: DynamicVertexBuffer,
// 	tex_coord_buffer: DynamicVertexBuffer,
// 	index_buffer: DynamicVertexBuffer,
// 	instance_buffer: FixedVertexBuffer<RawInstance>,
// 	camera_buffer: StaticBufferManager<RawCamera>,
// 	node_buffer: StaticBufferManager<RawNode>,
// 	point_light_buffer: StaticBufferManager<RawPointLight>
// }

// impl Drop for EngineHandler<'_> {
// 	fn drop(&mut self) {
// 		println!("dropping engine handler");
// 	}
// }

// impl<'a> EngineHandler<'a> {
// 	pub async fn new(tx: broadcast::Sender<Event>) -> Self {
// 		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
// 		let adapters = instance.enumerate_adapters(Backends::all());
// 		for adapter in adapters {
// 			println!("Adapter: {:?}", adapter.get_info());
// 		}
// 		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default())
// 			.await.expect("Failed to find an appropriate adapter");
// 		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
// 			required_features: Features::VERTEX_WRITABLE_STORAGE,
// 			..Default::default()
// 		}, None)
// 			.await.expect("Failed to create device");
		
// 		let device = Arc::new(device);
// 		let queue = Arc::new(queue);
// 		let adapter = Arc::new(adapter);
// 		let instance = Arc::new(instance);


// 		let animation_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
// 			label: Some("Animation Bind Group Layout"),
// 			entries: &[
// 				wgpu::BindGroupLayoutEntry {
// 					binding: 0,
// 					visibility: wgpu::ShaderStages::COMPUTE,
// 					ty: wgpu::BindingType::Buffer {
// 						ty: wgpu::BufferBindingType::Storage { read_only: true },
// 						has_dynamic_offset: false,
// 						min_binding_size: None,
// 					},
// 					count: None,
// 				},
// 				wgpu::BindGroupLayoutEntry {
// 					binding: 1,
// 					visibility: wgpu::ShaderStages::COMPUTE,
// 					ty: wgpu::BindingType::Buffer {
// 						ty: wgpu::BufferBindingType::Uniform,
// 						has_dynamic_offset: false,
// 						min_binding_size: None,
// 					},
// 					count: None,
// 				},
// 			],
// 		});
// 		let animation_bind_group_layout = Arc::new(animation_bind_group_layout);

// 		let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
// 			label: Some("Time Buffer"),
// 			size: 4,
// 			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
// 			mapped_at_creation: false
// 		});
// 		let key_frames_buffer = device.create_buffer(&wgpu::BufferDescriptor {
// 			label: Some("Key Frames Buffer"),
// 			size: 1024,
// 			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
// 			mapped_at_creation: false
// 		});
// 		let animation_buffer = StaticBufferManager::new(device.clone(), queue.clone());
// 		let keyframe_buffer = StaticBufferManager::new(device.clone(), queue.clone());

// 		let node_transformation_buffer = StaticBufferManager::new(device.clone(), queue.clone());
// 		let position_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
// 		let normal_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
// 		let tex_coord_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
// 		let index_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());

// 		let camera_buffer = StaticBufferManager::new(device.clone(), queue.clone());
// 		let instance_buffer = FixedVertexBuffer::new(device.clone(), queue.clone());
// 		let node_buffer = StaticBufferManager::new(device.clone(), queue.clone());
// 		let point_light_buffer = StaticBufferManager::new(device.clone(), queue.clone());

// 		let animation_pipeline = AnimationPipeline::new(AnimationPipelineArgs {
// 			instance: instance.clone(),
// 			queue: queue.clone(),
// 			device: device.clone(),
// 			adapter: adapter.clone(),
// 			animation_bind_group_layout: animation_buffer.bind_group_layout(),
// 			node_bind_group_layout: node_buffer.bind_group_layout(),
// 			change_node_bind_group_layout: node_transformation_buffer.bind_group_layout(),
// 		});

// 		Self {
// 			windows: HashMap::new(),
// 			device,
// 			queue,
// 			adapter,
// 			instance,
// 			draw_instructions: Vec::new(),
// 			tx,
// 			i: 0,
// 			since_last_frame: Instant::now(),
// 			time_buffer,
// 			key_frames_buffer,
// 			animation_pipeline,
// 			node_transformations: TransformationAcumalator::new(),
// 			animation_buffer,
// 			keyframe_buffer,
// 			node_transformation_buffer,
// 			position_buffer,
// 			normal_buffer,
// 			tex_coord_buffer,
// 			index_buffer,
// 			camera_buffer,
// 			instance_buffer,
// 			node_buffer,
// 			point_light_buffer,
// 		}
// 	}

// 	fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, gui_window: Window) {
// 		let window_attributes = winit::window::Window::default_attributes()
// 			.with_title(gui_window.title.as_str());
// 		let window = event_loop.create_window(window_attributes).unwrap();
// 		let window = Arc::new(window);
		
// 		let builder = RenderPipelineBuilder {
// 			queue: self.queue.clone(),
// 			adapter: self.adapter.clone(),
// 			device: self.device.clone(),
// 			window: window.clone(),
// 			instance: self.instance.clone(),
// 			node_bind_group_layout: self.node_buffer.bind_group_layout(),
// 			camera_bind_group_layout: self.camera_buffer.bind_group_layout(),
// 			point_light_bind_group_layout: self.point_light_buffer.bind_group_layout(),
// 		};
// 		let renderer = builder.build();

// 		self.windows.insert(window.id(), WindowContext {
// 			guid_id: gui_window.id,
// 			renderer,
// 			window,
// 			gui_window
// 		});
// 		self.render_every_window();
// 	}

// 	fn render_every_window(&mut self) {
// 		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
// 			label: Some("Render Encoder")
// 		});

// 		self.node_buffer.flush();
// 		self.point_light_buffer.flush();
// 		self.node_transformation_buffer.flush();
// 		self.instance_buffer.flush();
// 		self.position_buffer.flush();
// 		self.normal_buffer.flush();
// 		self.index_buffer.flush();
// 		self.camera_buffer.flush();

// 		let seconds = self.since_last_frame.elapsed().as_secs_f32();
// 		// println!("since last frame: {:?}", seconds);
// 		self.since_last_frame = Instant::now();
// 		self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[seconds]));

// 		for (node_id, mat) in self.node_transformations.get_items() {
// 			println!("writing node {:?} change {:?}", node_id, mat);
// 			let inx = self.node_buffer.get_inx(node_id).unwrap();
// 			println!("node inx: {:?}", inx);

// 			let changed_node = NodeTransformation {
// 				model: mat.to_cols_array_2d(),
// 				waiting: 1,
// 				_padding: [0; 3]
// 			};
// 			let changed_node = &[changed_node];
// 			let data = bytemuck::cast_slice(changed_node);
// 			let offset = inx * std::mem::size_of::<NodeTransformation>();
// 			let offset = offset as u64;
// 			println!("offset: {:?}", offset);

// 			self.queue.write_buffer(&self.node_transformation_buffer.buffer(), offset, data);
// 		}
// 		self.node_transformations.clear();

// 		self.animation_pipeline.animate(AnimateArgs {
// 			encoder: &mut encoder,
// 			animation_bind_group: &self.animation_buffer.bind_group(),
// 			node_bind_group: &self.node_buffer.bind_group(),
// 			change_node_bind_group: &self.node_transformation_buffer.bind_group(),
// 		});



// 		for (window_id, window) in self.windows.iter() {
// 			let args = RenderArgs {
// 				encoder: &mut encoder,
// 				camera_bind_group: &self.camera_buffer.bind_group(),
// 				node_bind_group: &self.node_buffer.bind_group(),
// 				point_light_bind_group: &self.point_light_buffer.bind_group(),
// 				positions_buffer: &self.position_buffer.buffer(),
// 				index_buffer: &self.index_buffer.buffer(),
// 				normal_buffer: &self.normal_buffer.buffer(),
// 				tex_coords_buffer: &self.tex_coord_buffer.buffer(),
// 				instance_buffer: &self.instance_buffer.buffer(),
// 				instructions: &self.draw_instructions
// 			};
// 			window.renderer.render(args).unwrap();
// 		}

// 		self.queue.submit(std::iter::once(encoder.finish()));
// 	}

// 	fn update_node(&mut self, node: &Node, parent_inx: Option<usize>) {
// 		// println!("node {} has parent {:?}", node.id, parent_id);
// 		// println!("translation: {:?}", node.translation);
// 		// println!("rotation: {:?}", node.rotation);
// 		println!("node: {} scale: {:?}", node.id, node.scale);
// 		let model = glam::Mat4::from_translation(node.translation) * glam::Mat4::from_quat(node.rotation) * glam::Mat4::from_scale(node.scale);
// 		// println!("model: {:?}", model.to_cols_array_2d());
// 		let n = RawNode {
// 			model: model.to_cols_array_2d(),
// 			parent_index: parent_inx.map_or(-1, |p| p as i32),
// 			_padding: [0; 3]
// 		};
// 		let node_index = self.node_buffer.store(node.id, n);
// 		println!("node index: {:?} data: {:?}", node_index, n);

// 		if let Some(mesh) = &node.mesh {
// 			// println!("write cube");
// 			println!("mesh: {:?}", mesh);

// 			let cube_positions_data = bytemuck::cast_slice(&mesh.positions);
// 			let pos_ptr = self.position_buffer.store(mesh.id, cube_positions_data);
// 			let cube_indices_data = bytemuck::cast_slice(&mesh.indices);
// 			let index_ptr = self.index_buffer.store(mesh.id, cube_indices_data);
// 			let normals_data = bytemuck::cast_slice(&mesh.normals);
// 			let normal_ptr = self.normal_buffer.store(mesh.id, normals_data);

// 			let instance = RawInstance {
// 				node_index: node_index as i32
// 			};
// 			let index = self.instance_buffer.store(node.id, instance);

// 			self.draw_instructions.push(DrawInstruction {
// 				index_range: index_ptr.offset as u64..index_ptr.offset as u64 + index_ptr.size as u64,
// 				position_range: pos_ptr.offset as u64..pos_ptr.offset as u64 + pos_ptr.size as u64,
// 				normal_range: normal_ptr.offset as u64..normal_ptr.offset as u64 + normal_ptr.size as u64,
// 				indices_range: 0..mesh.indices.len() as u32,
// 				instances_range: index as u32..index as u32 + 1
// 			});
// 		}

// 		if let Some(camera) = &node.camera {
// 			// let projection_matrix = glam::Mat4::perspective_rh(
// 			// 	camera.fovy, camera.aspect, camera.znear, camera.zfar);
// 			// let raw_camera = RawCamera {
// 			// 	proj: projection_matrix.to_cols_array_2d(),
// 			// 	_padding: [0; 3],
// 			// 	node_inx: node_index as i32
// 			// };
// 			// self.queue.write_buffer(&self.camera_buffer.buffer(), 0, bytemuck::cast_slice(&[raw_camera]));
// 			// self.camera_buffer.store(camera.id, raw_camera);
// 		}

// 		if let Some(point_light) = &node.point_light {
// 			// println!("{:?}", point_light);
// 			// let light = RawPointLight {
// 			// 	color: point_light.color,
// 			// 	intensity: point_light.intensity,
// 			// 	// _padding: 0.0,
// 			// 	node_inx: node_index as i32,
// 			// 	_padding2: [0.0; 3]
// 			// };
// 			let light = RawPointLight {
// 				intensity: 10.0,
// 				color: [1.0, 1.0, 1.0],
// 				node_inx: node_index as i32,
// 				// intensity: 1.0,
// 				// node_inx: node_index as i32,
// 				// _padding2: [0.0; 3]
// 			};
// 			println!("point light: {:?}", light);
// 			self.point_light_buffer.store(point_light.id, light);
// 		}

// 		for child in &node.children {
// 			println!("child id: {:?} parent_inx: {}", child.id, node_index);
// 			self.update_node(&child, Some(node_index));
// 		}
// 	}

// 	pub fn update_buffers(&mut self) {
// 		for 
// 	}

// 	fn send_keyboard_event(&self, event: KeyboardEvent) {
// 		self.tx.send(Event::InputEvent(InputEvent::KeyboardEvent(event))).unwrap();
// 	}

// 	fn send_mouse_event(&self, event: MouseEvent) {
// 		self.tx.send(Event::InputEvent(InputEvent::MouseEvent(event))).unwrap();
// 	}
// }

// impl ApplicationHandler<Command> for EngineHandler<'_> {
// 	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
// 		println!("resumed");

// 		let timer = Instant::now();
// 		let timer = timer.checked_add(Duration::from_millis(16)).unwrap();
// 		event_loop
// 			.set_control_flow(ControlFlow::WaitUntil(timer));
// 	}
// 	fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
// 		println!("exiting");
// 	}
// 	fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
// 		println!("suspended")
// 	}

// 	fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Command) {
// 		match event {
// 			Command::SaveWindow(gui_window) => {
// 				self.create_window(event_loop, gui_window);
// 			}
// 			Command::Exit => {
// 				println!("exit");
// 				event_loop.exit();
// 			}
// 			Command::SaveScene(scene) => {
// 				for node in scene.nodes {
// 					println!("scene node: {}", node.id);
// 					self.update_node(&node, None);
// 				}

// 				self.render_every_window();
// 			}
// 			Command::SetTransformation { node_id, transformation } => {
// 				println!("Setting transformation");
// 				// match self.node_offsets.get(&node_id) {
// 				// 	Some(offset) => {
// 				// 		let node_transfrom = NodeTransform {
// 				// 			model: transformation.data,
// 				// 			parent_index: -1
// 				// 		};
// 				// 		let data = &[node_transfrom];
// 				// 		let data = bytemuck::cast_slice(data);
// 				// 		self.nodes_buffer.write(Slot { offset: *offset as usize, size: data.len() }, data)
// 				// 	}
// 				// 	None => {

// 				// 	}
// 				// }
// 				// let node = self.nodes_buffer.get_mut::<Node>(node_id);
// 				// node.transformation = transformation;
// 			},
// 			Command::SetAnimation { node_id, animation } => {
// 				// println!("set animation node: {:?}", node_id);

// 				// TODO
// 				// let node_inx = self.node_staging_buffer.get_inx(&node_id).unwrap();
// 				// let value = animation.transform.to_cols_array_2d();
// 				// println!("value {:?}", value);
// 				// println!("node inx {:?}", node_inx);
// 				// let keyframe: Keyframe = Keyframe {
// 				// 	value,
// 				// 	is_running: 1,
// 				// 	node_inx: node_inx as u32
// 				// };
// 				// let keyframe = &[keyframe];
// 				// let data = bytemuck::cast_slice(keyframe);
// 				// self.queue.write_buffer(&self.key_frames_buffer, 0, data);
// 			},
// 			Command::ApplyTransformation { node_id, transformation } => {
// 				println!("apply transformation node: {:?}", node_id);
// 				// let inx = self.node_staging_buffer.get_inx(node_id).unwrap();
// 				// let node = self.node_staging_buffer.get(node_id).unwrap();
// 				// let new_transform = (transformation * glam::Mat4::from_cols_array_2d(&node.model)).to_cols_array_2d();
// 				// let node_transform = NodeTransform {
// 				// 	model: new_transform,
// 				// 	parent_index: node.parent_index,
// 				// 	_padding: [0; 3]
// 				// };
// 				self.node_transformations.accumulate(node_id, transformation);
// 				// self.node_transformations.
// 				// self.changed_node_stage_buffer.store_at_inx(inx, ChangedNode {
// 				// 	model: transformation.to_cols_array_2d(),
// 				// 	waiting: 1
// 				// });
// 				// self.flush_changed_node_staging_buffer();

// 				// let node_inx = self.node_staging_buffer.get_inx(node_id).unwrap();
// 				// let m = [
// 				// 	[1.0, 0.0, 0.0, 0.0],
// 				// 	[0.0, 1.0, 0.0, 0.0],
// 				// 	[0.0, 0.0, 1.0, 0.0],
// 				// 	[0.001, 0.001, 0.0, 1.0]
// 				// ];
// 				// let value = transformation.to_cols_array_2d();
// 				// println!("value {:?}", value);
// 				// let keyframe = Keyframe {
// 				// 	value,
// 				// 	is_running: 1,
// 				// 	node_inx: node_inx as u32
// 				// };
// 				// let keyframe = &[keyframe];
// 				// let data = bytemuck::cast_slice(keyframe);
// 				// self.queue.write_buffer(&self.key_frames_buffer, 0, data);
// 			}
// 		}
// 	}
// }
use core::time;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use log::error;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use wgpu::Backends;
use wgpu::BufferAddress;
use wgpu::Features;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;
use winit::window::WindowId;
use crate::animation_pipeline;
use crate::animation_pipeline::AnimateArgs;
use crate::animation_pipeline::AnimationPipeline;
use crate::animation_pipeline::AnimationPipelineArgs;
use crate::buffer::Buffer;
use crate::buffer::DynamicStagingBuffer;
use crate::buffer::Slot;
use crate::buffer::StaticBufferManager;
use crate::cube;
use crate::draw_queue::DrawQueue;
use crate::gui;
use crate::math::Mat4;
use crate::node_manager::NodeManager;
use crate::node_manager::NodeMetadata;
use crate::types::*;
use crate::wgpu_renderer::*;
use crate::wgpu_types::CameraUniform;
use crate::wgpu_types::Keyframe;
use crate::wgpu_types::NodeTransform;
use crate::wgpu_types::Position;
use crate::wgpu_types::RawInstance;
use crate::Window;

#[derive(Debug, Clone)]
pub enum Command {
	SaveWindow(Window),
	SaveScene(Scene),
	SetTransformation {
		node_id: usize,
		transformation: Mat4
	},
	SetAnimation {
		node_id: usize,
		animation: Animation
	},
	Exit
}

struct Inner {
	proxy: EventLoopProxy<Command>
}

impl Drop for Inner {
	fn drop(&mut self) {
		match self.proxy.send_event(Command::Exit) {
			Ok(_) => {}
			Err(err) => {
				error!("Error sending exit command: {:?}", err);
			}
		}
	}
}

pub struct EngineHandle {
	proxy: EventLoopProxy<Command>,
	inner: Inner,
	rx: broadcast::Receiver<Event>
}

impl EngineHandle {
	pub fn new(proxy: EventLoopProxy<Command>, rx: broadcast::Receiver<Event>) -> Self {
		let inner = Inner {
			proxy: proxy.clone()
		};
		Self {
			proxy,
			inner,
			rx
		}
	}

	pub fn save_scene(&self, scene: Scene) {
		self.proxy.send_event(Command::SaveScene(scene)).unwrap();
	}

	pub fn save_window(&self, window: &Window) {
		self.proxy.send_event(Command::SaveWindow(window.clone())).unwrap();
	}

	pub fn set_animation(&self, node_id: usize, animation: Animation) {
		self.proxy.send_event(Command::SetAnimation { node_id, animation }).unwrap();
	}

	pub async fn next_event(&mut self) -> Option<Event> {
		match self.rx.recv().await {
			Ok(event) => Some(event),
			Err(err) =>  {
				match err {
					RecvError::Closed => todo!(),
					RecvError::Lagged(_) => todo!(),
				}
			}
		}
	}
}

pub struct Engine {
	eventloop: EventLoop<Command>,
	tx: broadcast::Sender<Event>
}

impl Engine {
	pub fn new<F, Fut>(f: F) -> Self 
	where
		F: FnOnce(EngineHandle) -> Fut,
		Fut: Future<Output = ()> + Send + 'static,
	{
		let eventloop = EventLoop::<Command>::with_user_event().build().unwrap();
		let proxy = eventloop.create_proxy();
		// let (tx, rx) = mpsc::unbounded_channel();
		let (tx, _) = broadcast::channel(100);
		let handle = EngineHandle::new(proxy, tx.subscribe());
		tokio::spawn(f(handle));

		Self {
			eventloop,
			tx
		}
	}

	pub async fn run(self) -> anyhow::Result<()> {
		let mut handler = EngineHandler::new(self.tx).await;
		Ok(self.eventloop.run_app(&mut handler)?)
	}
}

#[derive(Debug)]
struct MeshPointer {
	incides: Slot,
	positions: Slot,
	normals: Slot,
	tex_coords: Range<u64>,
	indices_count: u32,
}

#[derive(Debug)]
struct WindowContext<'a> {
	guid_id: usize,
	renderer: Renderer<'a>,
	gui_window: gui::Window,
	window: Arc<winit::window::Window>
}



pub struct EngineHandler<'a> {
	windows: HashMap<WindowId, WindowContext<'a>>,
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	adapter: Arc<wgpu::Adapter>,
	instance: Arc<wgpu::Instance>,
	position_buffer: wgpu::Buffer,
	// normal_buffer: Buffer,
	// tex_coord_buffer: Buffer,
	indices_buffer: wgpu::Buffer,
	// mesh_pointers: HashMap<usize, MeshPointer>,
	node_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	node_bind_group: wgpu::BindGroup,
	node_buffer: wgpu::Buffer,
	camera_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	camera_bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	instance_buffer: wgpu::Buffer,
	time_buffer: wgpu::Buffer,
	key_frames_buffer: wgpu::Buffer,
	draw_instructions: Vec<DrawInstruction>,
	position_staging_buffer: DynamicStagingBuffer,
	index_staging_buffer: DynamicStagingBuffer,
	instance_staging_buffer: StaticBufferManager<RawInstance>,
	node_staging_buffer: StaticBufferManager<NodeTransform>,
	// draw_queue: DrawQueue
	tx: broadcast::Sender<Event>,
	i: usize,
	framer_timer: Instant,
	animation_pipeline: AnimationPipeline,
	animation_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	animation_bind_group: wgpu::BindGroup
}

impl Drop for EngineHandler<'_> {
	fn drop(&mut self) {
		println!("dropping engine handler");
	}
}

impl<'a> EngineHandler<'a> {
	pub async fn new(tx: broadcast::Sender<Event>) -> Self {
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

		let position_buffer = Position::create_buffer(&device, 500);
		let indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Indices Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});
		// let normal_buffer = Buffer::new(device.clone(), queue.clone());
		// let tex_coord_buffer = Buffer::new(device.clone(), queue.clone());

		let node_bind_group_layout = NodeTransform::create_bind_group_layout(&device);
		let node_bind_group_layout = Arc::new(node_bind_group_layout);
		let node_buffer = NodeTransform::create_buffer(&device);
		let node_bind_group = NodeTransform::create_bind_group(&device, &node_buffer, &node_bind_group_layout);

		let camera_bind_group_layout = CameraUniform::create_bind_group_layout(&device);
		let camera_buffer = CameraUniform::create_buffer(&device);
		let camera_bind_group = CameraUniform::create_bind_group(&device, &camera_buffer, &camera_bind_group_layout);

		let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Instance Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false
		});

		let animation_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Animation Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		});
		let animation_bind_group_layout = Arc::new(animation_bind_group_layout);

		let time_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Time Buffer"),
			size: 4,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});
		let key_frames_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Key Frames Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::STORAGE,
			mapped_at_creation: false
		});
		let animation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Animation Bind Group"),
			layout: &animation_bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &key_frames_buffer,
						offset: 0,
						size: None,
					}),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer: &time_buffer,
						offset: 0,
						size: None,
					}),
				},
			],
		});

		let animation_pipeline = AnimationPipeline::new(AnimationPipelineArgs {
			instance: instance.clone(),
			queue: queue.clone(),
			device: device.clone(),
			adapter: adapter.clone(),
			animation_bind_group_layout: animation_bind_group_layout.clone(),
			node_bind_group_layout: node_bind_group_layout.clone()
		});

		Self {
			windows: HashMap::new(),
			device,
			queue,
			adapter,
			instance,
			position_buffer,
			// normal_buffer,
			// tex_coord_buffer,
			indices_buffer,
			// mesh_pointers: HashMap::new(),
			draw_instructions: Vec::new(),
			camera_bind_group_layout: Arc::new(camera_bind_group_layout),
			camera_bind_group,
			camera_buffer,
			node_bind_group_layout,
			node_bind_group,
			node_buffer,
			instance_buffer,
			position_staging_buffer: DynamicStagingBuffer::new(1024),
			index_staging_buffer: DynamicStagingBuffer::new(1024),
			instance_staging_buffer: StaticBufferManager::new(1024),
			node_staging_buffer: StaticBufferManager::new(1024),
			tx,
			i: 0,
			framer_timer: Instant::now(),
			time_buffer,
			key_frames_buffer,
			animation_pipeline,
			animation_bind_group,
			animation_bind_group_layout
			// draw_queue: DrawQueue::new()
		}
	}

	fn create_window(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, gui_window: Window) {
		let window_attributes = winit::window::Window::default_attributes()
			.with_title(gui_window.title.as_str());
		let window = event_loop.create_window(window_attributes).unwrap();
		let window = Arc::new(window);
		
		let builder = RenderPipelineBuilder {
			queue: self.queue.clone(),
			adapter: self.adapter.clone(),
			device: self.device.clone(),
			window: window.clone(),
			instance: self.instance.clone(),
			node_bind_group_layout: self.node_bind_group_layout.clone(),
			camera_bind_group_layout: self.camera_bind_group_layout.clone(),
		};
		let renderer = builder.build();

		self.windows.insert(window.id(), WindowContext {
			guid_id: gui_window.id,
			renderer,
			window,
			gui_window
		});


		println!("windows {:?}", self.windows);
		println!("windows count {:?}", self.windows.len());

		self.render_every_window();
	}

	fn render_every_window(&mut self) {
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		let seconds = self.framer_timer.elapsed().as_secs_f32();
		self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[seconds]));

		self.animation_pipeline.animate(AnimateArgs {
			encoder: &mut encoder,
			animation_bind_group: &self.animation_bind_group,
			node_bind_group: &self.node_bind_group
		});



		for (window_id, window) in self.windows.iter() {
			let args = RenderArgs {
				encoder: &mut encoder,
				camera_bind_group: &self.camera_bind_group,
				node_bind_group: &self.node_bind_group,
				positions_buffer: &self.position_buffer,
				indices_buffer: &self.indices_buffer,
				instance_buffer: &self.instance_buffer,
				instructions: &self.draw_instructions
			};
			window.renderer.render(args).unwrap();
		}

		self.queue.submit(std::iter::once(encoder.finish()));
	}

	fn update_node(&mut self, node: &Node, parent_id: Option<usize>) {
		println!("node {} has parent {:?}", node.id, parent_id);
		println!("translation: {:?}", node.translation);
		println!("rotation: {:?}", node.rotation);
		let model = glam::Mat4::from_translation(node.translation) * glam::Mat4::from_quat(node.rotation);
		println!("model: {:?}", model.to_cols_array_2d());
		let n = NodeTransform {
			model: model.to_cols_array_2d(),
			parent_index: parent_id.map_or(-1, |p| p as i32),
			_padding: [0; 3]
		};
		let node_index = self.node_staging_buffer.store(node.id, n);
		println!("node index: {:?}", node_index);

		if let Some(mesh) = &node.mesh {
			println!("write cube");

			let cube_positions_data = bytemuck::cast_slice(&mesh.positions);
			let pos_ptr = self.position_staging_buffer.store(mesh.id, cube_positions_data);
			let cube_indices_data = bytemuck::cast_slice(&mesh.indices);
			let index_ptr = self.index_staging_buffer.store(mesh.id, cube_indices_data);

			let instance = RawInstance {
				node_index: node_index as i32
			};
			let index = self.instance_staging_buffer.store(node.id, instance);

			self.draw_instructions.push(DrawInstruction {
				index_range: index_ptr.offset as u64..index_ptr.offset as u64 + index_ptr.size as u64,
				position_range: pos_ptr.offset as u64..pos_ptr.offset as u64 + pos_ptr.size as u64,
				indices_range: 0..mesh.indices.len() as u32,
				instances_range: index as u32..index as u32 + 1
			});
		}

		if let Some(camera) = &node.camera {
			let projection_matrix = glam::Mat4::perspective_rh(
				camera.fovy, camera.aspect, camera.znear, camera.zfar);
			let camera_uniform = CameraUniform {
				proj: projection_matrix.to_cols_array_2d(),
				_padding: [0; 3],
				node_inx: node_index as i32
			};
			let camera_uniform_data = &[camera_uniform];
			let camera_uniform_data = bytemuck::cast_slice(camera_uniform_data);
			self.queue.write_buffer(&self.camera_buffer, 0, camera_uniform_data);
		}

		for child in &node.children {
			self.update_node(&child, Some(node.id));
		}
	}

	fn send_keyboard_event(&self, event: KeyboardEvent) {
		self.tx.send(Event::InputEvent(InputEvent::KeyboardEvent(event))).unwrap();
	}
}

impl ApplicationHandler<Command> for EngineHandler<'_> {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("resumed");

		let timer = Instant::now();
		let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
		event_loop
			.set_control_flow(ControlFlow::WaitUntil(timer));
	}
	fn exiting(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("exiting");
	}
	fn suspended(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("suspended")
	}

	fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: Command) {
		match event {
			Command::SaveWindow(gui_window) => {
				self.create_window(event_loop, gui_window);
			}
			Command::Exit => {
				println!("exit");
				event_loop.exit();
			}
			Command::SaveScene(scene) => {
				for node in scene.nodes {
					self.update_node(&node, None);
				}

				println!("write node buffer");
				for (offset, data) in self.node_staging_buffer.iter() {
					println!("write offset: {:?} data: {:?}", offset, data);
					self.queue.write_buffer(&self.node_buffer, offset as u64, bytemuck::cast_slice(&[*data]));
				}
				self.node_staging_buffer.clear_write_commands();

				println!("write instance buffer");
				for (offset, data) in self.instance_staging_buffer.iter() {
					println!("write offset: {:?} data: {:?}", offset, data);
					self.queue.write_buffer(&self.instance_buffer, offset as u64, bytemuck::cast_slice(&[*data]));
				}
				self.instance_staging_buffer.clear_write_commands();

				println!("write position buffer");
				for (offset, data) in self.position_staging_buffer.iter() {
					println!("write offset: {:?} data: {:?}", offset, data);
					self.queue.write_buffer(&self.position_buffer, offset as u64, data);
				}
				self.position_staging_buffer.clear_write_commands();

				println!("write index buffer");
				for (offset, data) in self.index_staging_buffer.iter() {
					println!("write offset: {:?} data: {:?}", offset, data);
					self.queue.write_buffer(&self.indices_buffer, offset as u64, data);
				}
				self.index_staging_buffer.clear_write_commands();
				println!("buffers written");

				self.render_every_window();
			}
			Command::SetTransformation { node_id, transformation } => {
				println!("Setting transformation");
				// match self.node_offsets.get(&node_id) {
				// 	Some(offset) => {
				// 		let node_transfrom = NodeTransform {
				// 			model: transformation.data,
				// 			parent_index: -1
				// 		};
				// 		let data = &[node_transfrom];
				// 		let data = bytemuck::cast_slice(data);
				// 		self.nodes_buffer.write(Slot { offset: *offset as usize, size: data.len() }, data)
				// 	}
				// 	None => {

				// 	}
				// }
				// let node = self.nodes_buffer.get_mut::<Node>(node_id);
				// node.transformation = transformation;
			},
			Command::SetAnimation { node_id, animation } => {
				println!("set animation node: {:?}", node_id);

				let node_inx = self.node_staging_buffer.get_inx(node_id).unwrap();
				let m = glam::Mat4::from_translation(glam::Vec3::new(0.01, 0.0, 0.0)).to_cols_array_2d();
				let keyframe = Keyframe {
					//animation_id: 0,
					is_running: 1,
					//node_id: node_inx as u32,
					//repeat: 1,
					//start_time: 1.0,
					//time: 0.0,
					value: m
				};
				self.queue.write_buffer(&self.key_frames_buffer, 0, bytemuck::cast_slice(&[keyframe]));

				// let node_transform = self.node_staging_buffer.get(node_id).unwrap();
				// let new_transform = (glam::Mat4::from_cols_array_2d(&node_transform.model) * animation.transform).to_cols_array_2d();
    
				// // Update the model matrix of the node_transform

				// let node_transform = NodeTransform {
				// 	model: new_transform,
				// 	parent_index: -1,
				// 	_padding: [0; 3]
				// };
				// self.node_staging_buffer.store(node_id, node_transform);
				// for (offset, n) in self.node_staging_buffer.iter() {
				// 	println!("write offset: {:?} data: {:?}", offset, n);
				// 	self.queue.write_buffer(&self.node_buffer, offset as u64, bytemuck::cast_slice(&[*n]));
				// }
				// self.node_staging_buffer.clear_write_commands();

				// self.render_every_window();
			}
		}
	}

	fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("about to wait {}", self.i);
		self.i += 1;

		let timer = Instant::now();
		let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
		event_loop
			.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(2000)));

		self.render_every_window();
	}

	fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
		println!("new events {}", self.i);
		
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: winit::event::WindowEvent,
	) {
		println!("window event");

		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				println!("redraw requested for window {:?}", window_id);
				match self.windows.get(&window_id) {
					Some(window) => {
						let renderer = &window.renderer;
						let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
							label: Some("Render Encoder")
						});
						let args = RenderArgs {
							encoder: &mut encoder,
							camera_bind_group: &self.camera_bind_group,
							node_bind_group: &self.node_bind_group,
							positions_buffer: &self.position_buffer,
							indices_buffer: &self.indices_buffer,
							instance_buffer: &self.instance_buffer,
							instructions: &self.draw_instructions
						};
						match renderer.render(args) {
							Ok(_) => {}
							Err(err) => {
								log::error!("Error rendering: {:?} window {:?}", err, window_id);
							}
						}
						self.queue.submit(std::iter::once(encoder.finish()));
					}
					None => {
						log::error!("Window not found: {:?}", window_id);
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
									match code {
										KeyCode::KeyW => {
											match state {
												winit::event::ElementState::Pressed => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::W,
														action: KeyAction::Pressed
													})
												}
												winit::event::ElementState::Released => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::W,
														action: KeyAction::Released
													})
												}
											}
										},
										KeyCode::KeyA => {
											match state {
												winit::event::ElementState::Pressed => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::A,
														action: KeyAction::Pressed
													})
												}
												winit::event::ElementState::Released => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::A,
														action: KeyAction::Released
													})
												}
											}
										},
										KeyCode::KeyS => {
											match state {
												winit::event::ElementState::Pressed => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::S,
														action: KeyAction::Pressed
													})
												}
												winit::event::ElementState::Released => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::S,
														action: KeyAction::Released
													})
												}
											}
										},
										KeyCode::KeyD => {
											match state {
												winit::event::ElementState::Pressed => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::D,
														action: KeyAction::Pressed
													})
												}
												winit::event::ElementState::Released => {
													self.send_keyboard_event(KeyboardEvent {
														key: KeyboardKey::D,
														action: KeyAction::Released
													})
												}
											}
										},
										_ => {}
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

		// let timer = Instant::now();
		// let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
		// event_loop
		// 		.set_control_flow(ControlFlow::WaitUntil(timer));
	}
}
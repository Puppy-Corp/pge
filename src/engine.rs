use core::time;
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::os::macos::raw;
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
use winit::dpi::PhysicalPosition;
use winit::dpi::Position;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;
use winit::window;
use winit::window::WindowId;
use crate::acumalator::TransformationAcumalator;
use crate::animation_pipeline;
use crate::animation_pipeline::AnimateArgs;
use crate::animation_pipeline::AnimationPipeline;
use crate::animation_pipeline::AnimationPipelineArgs;
use crate::buffer::Buffer;
use crate::buffer::DynamicStagingBuffer;
use crate::buffer::DynamicVertexBuffer;
use crate::buffer::FixedVertexBuffer;
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
use crate::wgpu_types::RawAnimation;
use crate::wgpu_types::RawCamera;
use crate::wgpu_types::Keyframe;
use crate::wgpu_types::RawKeyFrame;
use crate::wgpu_types::RawNode;
use crate::wgpu_types::Positions;
use crate::wgpu_types::RawInstance;
use crate::wgpu_types::NodeTransformation;
use crate::wgpu_types::RawPointLight;
use crate::Window;

#[derive(Debug, Clone)]
pub enum Command {
	SaveWindow(Window),
	SaveScene(Scene),
	SetTransformation {
		node_id: usize,
		transformation: Mat4
	},
	ApplyTransformation {
		node_id: usize,
		transformation: glam::Mat4
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

impl Clone for Inner {
	fn clone(&self) -> Self {
		Self {
			proxy: self.proxy.clone()
		}
	}
}

pub struct EngineHandle {
	proxy: EventLoopProxy<Command>,
	inner: Inner,
	tx: broadcast::Sender<Event>,
	rx: broadcast::Receiver<Event>
}

impl Clone for EngineHandle {
	fn clone(&self) -> Self {
		Self {
			proxy: self.proxy.clone(),
			inner: self.inner.clone(),
			rx: self.tx.subscribe(),
			tx: self.tx.clone()
		}
	}
}

impl EngineHandle {
	pub fn new(proxy: EventLoopProxy<Command>, tx: broadcast::Sender<Event>) -> Self {
		let inner = Inner {
			proxy: proxy.clone()
		};
		Self {
			proxy,
			inner,
			rx: tx.subscribe(),
			tx
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

	pub fn apply_transformation(&self, node_id: usize, transformation: glam::Mat4) {
		self.proxy.send_event(Command::ApplyTransformation { node_id, transformation }).unwrap();
	}

	pub fn rotate_node(&self, node_id: usize, dx: f32, dy: f32) {
		let transformation = glam::Mat4::from_rotation_x(-dy) * glam::Mat4::from_rotation_y(-dx);
		self.apply_transformation(node_id, transformation);
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
		let handle = EngineHandle::new(proxy, tx.clone());
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
	
	time_buffer: wgpu::Buffer,
	key_frames_buffer: wgpu::Buffer,
	draw_instructions: Vec<DrawInstruction>,
	tx: broadcast::Sender<Event>,
	i: usize,
	since_last_frame: Instant,
	animation_pipeline: AnimationPipeline,
	animation_buffer: StaticBufferManager<RawAnimation>,
	keyframe_buffer: StaticBufferManager<RawKeyFrame>,
	node_transformations: TransformationAcumalator,
	node_transformation_buffer: StaticBufferManager<NodeTransformation>,
	position_buffer: DynamicVertexBuffer,
	normal_buffer: DynamicVertexBuffer,
	tex_coord_buffer: DynamicVertexBuffer,
	index_buffer: DynamicVertexBuffer,
	instance_buffer: FixedVertexBuffer<RawInstance>,
	camera_buffer: StaticBufferManager<RawCamera>,
	node_buffer: StaticBufferManager<RawNode>,
	point_light_buffer: StaticBufferManager<RawPointLight>
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
		// let animation_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		// 	label: Some("Animation Bind Group"),
		// 	layout: &animation_bind_group_layout,
		// 	entries: &[
		// 		wgpu::BindGroupEntry {
		// 			binding: 0,
		// 			resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
		// 				buffer: &key_frames_buffer,
		// 				offset: 0,
		// 				size: None,
		// 			}),
		// 		},
		// 		wgpu::BindGroupEntry {
		// 			binding: 1,
		// 			resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
		// 				buffer: &time_buffer,
		// 				offset: 0,
		// 				size: None,
		// 			}),
		// 		},
		// 	],
		// });

		// let changed_node_bind_group_layout = NodeTransformation::create_bind_group_layout(&device);
		// let changed_node_buffer = NodeTransformation::create_buffer(&device);
		// let changed_node_bind_group = NodeTransformation::create_bind_group(&device, &changed_node_buffer, &changed_node_bind_group_layout);
		// let changed_node_bind_group_layout = Arc::new(changed_node_bind_group_layout);

		let animation_buffer = StaticBufferManager::new(device.clone(), queue.clone());
		let keyframe_buffer = StaticBufferManager::new(device.clone(), queue.clone());

		let node_transformation_buffer = StaticBufferManager::new(device.clone(), queue.clone());
		let position_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
		let normal_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
		let tex_coord_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());
		let index_buffer = DynamicVertexBuffer::new(device.clone(), queue.clone());

		let camera_buffer = StaticBufferManager::new(device.clone(), queue.clone());
		let instance_buffer = FixedVertexBuffer::new(device.clone(), queue.clone());
		let node_buffer = StaticBufferManager::new(device.clone(), queue.clone());
		let point_light_buffer = StaticBufferManager::new(device.clone(), queue.clone());

		let animation_pipeline = AnimationPipeline::new(AnimationPipelineArgs {
			instance: instance.clone(),
			queue: queue.clone(),
			device: device.clone(),
			adapter: adapter.clone(),
			animation_bind_group_layout: animation_buffer.bind_group_layout(),
			node_bind_group_layout: node_buffer.bind_group_layout(),
			change_node_bind_group_layout: node_transformation_buffer.bind_group_layout(),
		});

		Self {
			windows: HashMap::new(),
			device,
			queue,
			adapter,
			instance,
			draw_instructions: Vec::new(),
			tx,
			i: 0,
			since_last_frame: Instant::now(),
			time_buffer,
			key_frames_buffer,
			animation_pipeline,
			node_transformations: TransformationAcumalator::new(),
			animation_buffer,
			keyframe_buffer,
			node_transformation_buffer,
			position_buffer,
			normal_buffer,
			tex_coord_buffer,
			index_buffer,
			camera_buffer,
			instance_buffer,
			node_buffer,
			point_light_buffer,
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
			node_bind_group_layout: self.node_buffer.bind_group_layout(),
			camera_bind_group_layout: self.camera_buffer.bind_group_layout(),
			point_light_bind_group_layout: self.point_light_buffer.bind_group_layout(),
		};
		let renderer = builder.build();

		self.windows.insert(window.id(), WindowContext {
			guid_id: gui_window.id,
			renderer,
			window,
			gui_window
		});
		self.render_every_window();
	}

	fn render_every_window(&mut self) {
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		self.node_buffer.flush();
		self.point_light_buffer.flush();
		self.node_transformation_buffer.flush();
		self.instance_buffer.flush();
		self.position_buffer.flush();
		self.normal_buffer.flush();
		self.index_buffer.flush();
		self.camera_buffer.flush();

		let seconds = self.since_last_frame.elapsed().as_secs_f32();
		// println!("since last frame: {:?}", seconds);
		self.since_last_frame = Instant::now();
		self.queue.write_buffer(&self.time_buffer, 0, bytemuck::cast_slice(&[seconds]));

		for (node_id, mat) in self.node_transformations.get_items() {
			println!("writing node {:?} change {:?}", node_id, mat);
			let inx = self.node_buffer.get_inx(node_id).unwrap();
			println!("node inx: {:?}", inx);

			let changed_node = NodeTransformation {
				model: mat.to_cols_array_2d(),
				waiting: 1,
				_padding: [0; 3]
			};
			let changed_node = &[changed_node];
			let data = bytemuck::cast_slice(changed_node);
			let offset = inx * std::mem::size_of::<NodeTransformation>();
			let offset = offset as u64;
			println!("offset: {:?}", offset);

			self.queue.write_buffer(&self.node_transformation_buffer.buffer(), offset, data);
		}
		self.node_transformations.clear();

		self.animation_pipeline.animate(AnimateArgs {
			encoder: &mut encoder,
			animation_bind_group: &self.animation_buffer.bind_group(),
			node_bind_group: &self.node_buffer.bind_group(),
			change_node_bind_group: &self.node_transformation_buffer.bind_group(),
		});



		for (window_id, window) in self.windows.iter() {
			let args = RenderArgs {
				encoder: &mut encoder,
				camera_bind_group: &self.camera_buffer.bind_group(),
				node_bind_group: &self.node_buffer.bind_group(),
				point_light_bind_group: &self.point_light_buffer.bind_group(),
				positions_buffer: &self.position_buffer.buffer(),
				index_buffer: &self.index_buffer.buffer(),
				normal_buffer: &self.normal_buffer.buffer(),
				tex_coords_buffer: &self.tex_coord_buffer.buffer(),
				instance_buffer: &self.instance_buffer.buffer(),
				instructions: &self.draw_instructions
			};
			window.renderer.render(args).unwrap();
		}

		self.queue.submit(std::iter::once(encoder.finish()));
	}

	fn update_node(&mut self, node: &Node, parent_inx: Option<usize>) {
		// println!("node {} has parent {:?}", node.id, parent_id);
		// println!("translation: {:?}", node.translation);
		// println!("rotation: {:?}", node.rotation);
		let model = glam::Mat4::from_translation(node.translation) * glam::Mat4::from_quat(node.rotation);
		// println!("model: {:?}", model.to_cols_array_2d());
		let n = RawNode {
			model: model.to_cols_array_2d(),
			parent_index: parent_inx.map_or(-1, |p| p as i32),
			_padding: [0; 3]
		};
		let node_index = self.node_buffer.store(node.id, n);
		println!("node index: {:?} data: {:?}", node_index, n);

		if let Some(mesh) = &node.mesh {
			// println!("write cube");
			println!("mesh: {:?}", mesh);

			let cube_positions_data = bytemuck::cast_slice(&mesh.positions);
			let pos_ptr = self.position_buffer.store(mesh.id, cube_positions_data);
			let cube_indices_data = bytemuck::cast_slice(&mesh.indices);
			let index_ptr = self.index_buffer.store(mesh.id, cube_indices_data);
			let normals_data = bytemuck::cast_slice(&mesh.normals);
			let normal_ptr = self.normal_buffer.store(mesh.id, normals_data);

			let instance = RawInstance {
				node_index: node_index as i32
			};
			let index = self.instance_buffer.store(node.id, instance);

			self.draw_instructions.push(DrawInstruction {
				index_range: index_ptr.offset as u64..index_ptr.offset as u64 + index_ptr.size as u64,
				position_range: pos_ptr.offset as u64..pos_ptr.offset as u64 + pos_ptr.size as u64,
				normal_range: normal_ptr.offset as u64..normal_ptr.offset as u64 + normal_ptr.size as u64,
				indices_range: 0..mesh.indices.len() as u32,
				instances_range: index as u32..index as u32 + 1
			});
		}

		if let Some(camera) = &node.camera {
			let projection_matrix = glam::Mat4::perspective_rh(
				camera.fovy, camera.aspect, camera.znear, camera.zfar);
			let raw_camera = RawCamera {
				proj: projection_matrix.to_cols_array_2d(),
				_padding: [0; 3],
				node_inx: node_index as i32
			};
			self.queue.write_buffer(&self.camera_buffer.buffer(), 0, bytemuck::cast_slice(&[raw_camera]));
			self.camera_buffer.store(camera.id, raw_camera);
		}

		if let Some(point_light) = &node.point_light {
			// println!("{:?}", point_light);
			// let light = RawPointLight {
			// 	color: point_light.color,
			// 	intensity: point_light.intensity,
			// 	// _padding: 0.0,
			// 	node_inx: node_index as i32,
			// 	_padding2: [0.0; 3]
			// };
			let light = RawPointLight {
				intensity: 10.0,
				color: [1.0, 1.0, 1.0],
				node_inx: node_index as i32,
				// intensity: 1.0,
				// node_inx: node_index as i32,
				// _padding2: [0.0; 3]
			};
			println!("point light: {:?}", light);
			self.point_light_buffer.store(point_light.id, light);
		}

		for child in &node.children {
			println!("child id: {:?} parent_inx: {}", child.id, node_index);
			self.update_node(&child, Some(node_index));
		}
	}

	fn send_keyboard_event(&self, event: KeyboardEvent) {
		self.tx.send(Event::InputEvent(InputEvent::KeyboardEvent(event))).unwrap();
	}

	fn send_mouse_event(&self, event: MouseEvent) {
		self.tx.send(Event::InputEvent(InputEvent::MouseEvent(event))).unwrap();
	}
}

impl ApplicationHandler<Command> for EngineHandler<'_> {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("resumed");

		let timer = Instant::now();
		let timer = timer.checked_add(Duration::from_millis(16)).unwrap();
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
					println!("scene node: {}", node.id);
					self.update_node(&node, None);
				}

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
				// println!("set animation node: {:?}", node_id);

				// TODO
				// let node_inx = self.node_staging_buffer.get_inx(&node_id).unwrap();
				// let value = animation.transform.to_cols_array_2d();
				// println!("value {:?}", value);
				// println!("node inx {:?}", node_inx);
				// let keyframe: Keyframe = Keyframe {
				// 	value,
				// 	is_running: 1,
				// 	node_inx: node_inx as u32
				// };
				// let keyframe = &[keyframe];
				// let data = bytemuck::cast_slice(keyframe);
				// self.queue.write_buffer(&self.key_frames_buffer, 0, data);
			},
			Command::ApplyTransformation { node_id, transformation } => {
				println!("apply transformation node: {:?}", node_id);
				// let inx = self.node_staging_buffer.get_inx(node_id).unwrap();
				// let node = self.node_staging_buffer.get(node_id).unwrap();
				// let new_transform = (transformation * glam::Mat4::from_cols_array_2d(&node.model)).to_cols_array_2d();
				// let node_transform = NodeTransform {
				// 	model: new_transform,
				// 	parent_index: node.parent_index,
				// 	_padding: [0; 3]
				// };
				self.node_transformations.accumulate(node_id, transformation);
				// self.node_transformations.
				// self.changed_node_stage_buffer.store_at_inx(inx, ChangedNode {
				// 	model: transformation.to_cols_array_2d(),
				// 	waiting: 1
				// });
				// self.flush_changed_node_staging_buffer();

				// let node_inx = self.node_staging_buffer.get_inx(node_id).unwrap();
				// let m = [
				// 	[1.0, 0.0, 0.0, 0.0],
				// 	[0.0, 1.0, 0.0, 0.0],
				// 	[0.0, 0.0, 1.0, 0.0],
				// 	[0.001, 0.001, 0.0, 1.0]
				// ];
				// let value = transformation.to_cols_array_2d();
				// println!("value {:?}", value);
				// let keyframe = Keyframe {
				// 	value,
				// 	is_running: 1,
				// 	node_inx: node_inx as u32
				// };
				// let keyframe = &[keyframe];
				// let data = bytemuck::cast_slice(keyframe);
				// self.queue.write_buffer(&self.key_frames_buffer, 0, data);
			}
		}
	}

	fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		// println!("about to wait {}", self.i);
		self.i += 1;

		let timer = Instant::now();
		let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
		event_loop
			.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(16)));

		self.render_every_window();
	}

	fn new_events(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, cause: winit::event::StartCause) {
		// println!("new events {}", self.i);
		
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
				if let Some(window) = self.windows.get(&window_id) {
					let size = &window.window.inner_size();
					let middle_x = size.width as f64 / 2.0;
					let middle_y = size.height as f64 / 2.0;
					let dx = middle_x - position.x;
					let dy = middle_y - position.y;
					let dx = dx as f32;
					let dy = dy as f32;
					self.send_mouse_event(MouseEvent::Moved { dx, dy });
					window.window.set_cursor_position(PhysicalPosition::new(middle_x, middle_y)).unwrap();
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
	}
}
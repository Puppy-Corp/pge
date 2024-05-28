use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::sync::Arc;
use glam::Vec3;
use log::error;
use tokio::sync::oneshot::error;
use wgpu::Backends;
use wgpu::BufferAddress;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;
use crate::buffer::Buffer;
use crate::buffer::Slot;
use crate::cube;
use crate::gui;
use crate::math::Mat4;
use crate::types::*;
use crate::wgpu_renderer::*;
use crate::wgpu_types::CameraUniform;
use crate::wgpu_types::NodeTransform;
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
	inner: Inner
}

impl EngineHandle {
	pub fn new(proxy: EventLoopProxy<Command>) -> Self {
		let inner = Inner {
			proxy: proxy.clone()
		};
		Self {
			proxy,
			inner
		}
	}

	pub fn save_scene(&self, scene: Scene) {
		self.proxy.send_event(Command::SaveScene(scene)).unwrap();
	}

	pub fn save_window(&self, window: &Window) {
		self.proxy.send_event(Command::SaveWindow(window.clone())).unwrap();
	}
}

pub struct Engine {
	eventloop: EventLoop<Command>,
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
		let handle = EngineHandle::new(proxy);
		tokio::spawn(f(handle));

		Self {
			eventloop
		}
	}

	pub async fn run(self) -> anyhow::Result<()> {
		let mut handler = EngineHandler::new().await;
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
	position_buffer: Buffer,
	normal_buffer: Buffer,
	tex_coord_buffer: Buffer,
	indices_buffer: Buffer,
	nodes_buffer: Buffer,
	mesh_pointers: HashMap<usize, MeshPointer>,
	node_offsets: HashMap<usize, u64>,
	node_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	node_bind_group: wgpu::BindGroup,
	node_buffer: wgpu::Buffer,
	camera_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	camera_bind_group: wgpu::BindGroup,
	camera_buffer: wgpu::Buffer,
	instance_buffer: wgpu::Buffer,
	draw_instructions: Vec<DrawInstruction>,
}

impl Drop for EngineHandler<'_> {
	fn drop(&mut self) {
		println!("dropping engine handler");
	}
}

impl<'a> EngineHandler<'a> {
	pub async fn new() -> Self {
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
		let adapters = instance.enumerate_adapters(Backends::all());
		for adapter in adapters {
			println!("Adapter: {:?}", adapter.get_info());
		}
		let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions::default())
			.await.expect("Failed to find an appropriate adapter");
		let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor::default(), None)
			.await.expect("Failed to create device");
		
		let device = Arc::new(device);
		let queue = Arc::new(queue);
		let adapter = Arc::new(adapter);
		let instance = Arc::new(instance);

		let position_buffer = Buffer::new(device.clone(), queue.clone());
		let normal_buffer = Buffer::new(device.clone(), queue.clone());
		let tex_coord_buffer = Buffer::new(device.clone(), queue.clone());
		let indices_buffer = Buffer::new(device.clone(), queue.clone());
		let nodes_buffer = Buffer::new(device.clone(), queue.clone());

		let node_bind_group_layout = NodeTransform::create_bind_group_layout(&device);
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

		Self {
			windows: HashMap::new(),
			device,
			queue,
			adapter,
			instance,
			position_buffer,
			normal_buffer,
			tex_coord_buffer,
			indices_buffer,
			nodes_buffer,
			mesh_pointers: HashMap::new(),
			node_offsets: HashMap::new(),
			draw_instructions: Vec::new(),
			camera_bind_group_layout: Arc::new(camera_bind_group_layout),
			camera_bind_group,
			camera_buffer,
			node_bind_group_layout: Arc::new(node_bind_group_layout),
			node_bind_group,
			node_buffer,
			instance_buffer
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

		// let cube = cube(3.0);
		// println!("cute indices count: {}", cube.indices.len());
		// let indice_data = bytemuck::cast_slice(&cube.indices);
		// println!("indice data len: {:?}", indice_data.len());
		// let position_data = bytemuck::cast_slice(&cube.positions);
		// println!("position data len: {:?}", position_data.len());
		// // self.queue.write_buffer(&self.position_buffer.buffer(), 0, position_data);
		// // self.queue.write_buffer(&self.indices_buffer.buffer(), 0, indice_data);
		// self.position_buffer.store(position_data);
		// self.indices_buffer.store(indice_data);

		// self.draw_instructions.push(DrawInstruction {
		// 	// position_range: first_mesh.positions.offset as u64..first_mesh.positions.offset as u64 + first_mesh.positions.size as u64,
		// 	// indices_range: first_mesh.incides.offset as u64..first_mesh.incides.offset as u64 + first_mesh.incides.size as u64,
		// 	instances_range: 0..1,
		// 	indices_range: 0..(cube.indices.len() * 2) as u32,
		// 	index_range: 0..indice_data.len() as u64,
		// 	position_range: 0..position_data.len() as u64,
		// });

		// let args = RenderArgs {
		// 	camera_bind_group: &self.camera_bind_group,
		// 	node_bind_group: &self.node_bind_group,
		// 	positions_buffer: &self.position_buffer.buffer(),
		// 	indices_buffer: &self.indices_buffer.buffer(),
		// 	instructions: &self.draw_instructions
		// };
		// renderer.render(args).unwrap();

		let aspect = 16.0 / 9.0;
		let fovy = std::f32::consts::PI / 3.0; // 60 degrees field of view
		let znear = 0.1;
		let zfar = 100.0;
		let projection_matrix = glam::Mat4::perspective_rh(fovy, aspect, znear, zfar);

		let eye = glam::Vec3::new(3.0, 5.0, 6.0);
		let center = glam::Vec3::new(0.0, 0.0, 0.0);
		let up = glam::Vec3::new(0.0, 1.0, 0.0);

		// Compute the view and projection matrices
		let view_matrix = glam::Mat4::look_at_rh(eye, center, up);

		println!("write cameranode");
		let camera_node = NodeTransform {
			model: view_matrix.to_cols_array_2d(),
			parent_index: -1,
			_padding: [0; 3]
		};
		let camera_node_data = &[camera_node];
		let camera_node_data = bytemuck::cast_slice(camera_node_data);
		self.queue.write_buffer(&self.node_buffer, 0, camera_node_data);

		println!("write camera uniform");
		let camera_uniform = CameraUniform {
			proj: projection_matrix.to_cols_array_2d(),
			_padding: [0; 3],
			node_inx: 0
		};
		let camera_uniform_data = &[camera_uniform];
		let camera_uniform_data = bytemuck::cast_slice(camera_uniform_data);
		self.queue.write_buffer(&self.camera_buffer, 0, camera_uniform_data);

		println!("write cube");
		let cube = cube(1.0);
		let cube_positions_data = bytemuck::cast_slice(&cube.positions);
		self.position_buffer.store(cube_positions_data);
		let cube_indices_data = bytemuck::cast_slice(&cube.indices);
		self.indices_buffer.store(cube_indices_data);

		println!("write node transform");
		let cube_node_transform = NodeTransform {
			model: glam::Mat4::from_translation(glam::Vec3::new(0.0, 2.0, 0.0)).to_cols_array_2d(),
			parent_index: -1,
			_padding: [0; 3]
		};
		let cube_node_transform_data = &[cube_node_transform];
		let cube_node_transform_data = bytemuck::cast_slice(cube_node_transform_data);
		self.queue.write_buffer(&self.node_buffer, std::mem::size_of::<NodeTransform>() as BufferAddress, cube_node_transform_data);
		let instance = RawInstance {
			node_index: 1
		};

		println!("write instance");
		let instance_data = &[instance];
		let instance_data = bytemuck::cast_slice(instance_data);
		self.queue.write_buffer(&self.instance_buffer, 0, instance_data);

		self.draw_instructions.push(DrawInstruction {
			position_range: 0..cube_positions_data.len() as u64,
			index_range: 0..cube_indices_data.len() as u64,
			indices_range: 0..cube.indices.len() as u32,
			instances_range: 0..1
		});

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
		println!("rendering every window222 {:?}", self.windows);
		for (window_id, window) in self.windows.iter() {
			println!("WHAT IS THISSS ????");
			println!("positions_buffer {:?}", self.position_buffer.buffer());
			let args = RenderArgs {
				camera_bind_group: &self.camera_bind_group,
				node_bind_group: &self.node_bind_group,
				positions_buffer: &self.position_buffer.buffer(),
				indices_buffer: &self.indices_buffer.buffer(),
				instance_buffer: &self.instance_buffer,
				instructions: &self.draw_instructions
			};
			window.renderer.render(args).unwrap();
		}
	}
}

impl ApplicationHandler<Command> for EngineHandler<'_> {
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		println!("resumed");
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
				// println!("Saving scene {:?}", scene);
				// for node in scene.nodes {
				// 	if let Some(mesh) = node.mesh {
				// 		let positions = mesh.positions;
				// 		let normals = mesh.normals;
				// 		println!("mesh: {}", mesh.id);
				// 		println!("positions: {:?}", positions);
				// 		println!("indices: {:?}", mesh.indices);

				// 		match self.mesh_pointers.get(&mesh.id) {
				// 			Some(pointer) => {
				// 				println!("mesh pointer found");
				// 				self.position_buffer.write(pointer.positions, bytemuck::cast_slice(&positions));
				// 				self.normal_buffer.write(pointer.normals, bytemuck::cast_slice(&normals));
				// 				self.indices_buffer.write(pointer.incides, bytemuck::cast_slice(&mesh.indices));
				// 			}
				// 			None => {
				// 				println!("mesh pointer not found");
				// 				println!("positions len: {:?}", positions.len());
				// 				let positions_data = bytemuck::cast_slice(&positions);
				// 				println!("positions data len: {:?}", positions_data.len());
				// 				let positions = self.position_buffer.store(positions_data);
				// 				let normals = self.normal_buffer.store(bytemuck::cast_slice(&normals));
				// 				println!("indices len: {:?}", mesh.indices.len());
				// 				let indice_data = bytemuck::cast_slice(&mesh.indices);
				// 				println!("indice data len: {:?}", indice_data.len());
				// 				let incides = self.indices_buffer.store(indice_data);
				// 				let mesh_pointer = MeshPointer {
				// 					positions,
				// 					normals,
				// 					incides,
				// 					tex_coords: 0..0,
				// 					indices_count: mesh.indices.len() as u32
				// 				};
				// 				println!("mesh pointer: {:?}", mesh_pointer);
				// 				self.mesh_pointers.insert(mesh.id, mesh_pointer);
				// 			}
				// 		}
				// 	}

				// 	let transformation = Mat4::translation(-0.5, -5.0, 0.0);
				// 	println!("transformation {:?}", transformation);

				// 	let node_transfrom = NodeTransform {
				// 		model: transformation.data,
				// 		parent_index: -1,
				// 		_padding: [0; 3]
				// 	};
				// 	let data = &[node_transfrom];
				// 	let data = bytemuck::cast_slice(data);
				// 	self.queue.write_buffer(&self.node_buffer, 0, data);

				// 	let instance = RawInstance {
				// 		node_index: 0
				// 	};
				// 	let data = &[instance];
				// 	let data = bytemuck::cast_slice(data);
				// 	self.queue.write_buffer(&self.instance_buffer, 0, data);

				// 	let (_,first_mesh) = self.mesh_pointers.iter().next().unwrap();

				// 	self.draw_instructions.push(DrawInstruction {
				// 		position_range: first_mesh.positions.offset as u64..first_mesh.positions.offset as u64 + first_mesh.positions.size as u64,
				// 		index_range: first_mesh.incides.offset as u64..first_mesh.incides.offset as u64 + first_mesh.incides.size as u64,
				// 		indices_range: 0..first_mesh.indices_count,
				// 		instances_range: 0..1
				// 	});

				// 	let aspect = 16.0 / 9.0;
				// 	let fovy = std::f32::consts::PI / 3.0; // 60 degrees field of view
				// 	let znear = 0.1;
				// 	let zfar = 100.0;
				// 	let eye = glam::Vec3::new(3.0, 5.0, 6.0);
				// 	let center = glam::Vec3::new(0.0, 0.0, 0.0);
				// 	let up = glam::Vec3::new(0.0, 1.0, 0.0);

				// 	// Compute the view and projection matrices
				// 	let view_matrix = glam::Mat4::look_at_rh(eye, center, up);
				// 	let projection_matrix = glam::Mat4::perspective_rh(fovy, aspect, znear, zfar);
				// 	let camera = CameraUniform {
				// 		view: view_matrix.to_cols_array_2d(),
				// 		proj: projection_matrix.to_cols_array_2d()
				// 	};
				// 	self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[camera]));



				// 	self.render_every_window();
				// }
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
			}
		}
	}

	fn window_event(
		&mut self,
		event_loop: &winit::event_loop::ActiveEventLoop,
		window_id: winit::window::WindowId,
		event: winit::event::WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				event_loop.exit();
			}
			WindowEvent::RedrawRequested => {
				println!("redraw requested for window {:?}", window_id);
				match self.windows.get(&window_id) {
					Some(window) => {
						let renderer = &window.renderer;
						let args = RenderArgs {
							camera_bind_group: &self.camera_bind_group,
							node_bind_group: &self.node_bind_group,
							positions_buffer: &self.position_buffer.buffer(),
							indices_buffer: &self.indices_buffer.buffer(),
							instance_buffer: &self.instance_buffer,
							instructions: &self.draw_instructions
						};
						match renderer.render(args) {
							Ok(_) => {}
							Err(err) => {
								log::error!("Error rendering: {:?} window {:?}", err, window_id);
							}
						}
					}
					None => {
						log::error!("Window not found: {:?}", window_id);
					}
				}
			}
			_ => {}
		}
	}
}
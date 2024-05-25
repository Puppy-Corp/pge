use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::sync::Arc;
use wgpu::Backends;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::window::WindowId;
use crate::buffer::Buffer;
use crate::buffer::Slot;
use crate::gui;
use crate::types::*;
use crate::wgpu_renderer::RenderArgs;
use crate::wgpu_renderer::Renderer;
use crate::Window;

#[derive(Debug, Clone)]
pub enum Command {
	SaveWindow(Window),
	SaveScene(Scene),
	Exit
}

struct Inner {
	proxy: EventLoopProxy<Command>
}

impl Drop for Inner {
	fn drop(&mut self) {
		self.proxy.send_event(Command::Exit).unwrap();
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

	pub fn save_scene(&self, scene: &Scene) {
		self.proxy.send_event(Command::SaveScene(scene.clone())).unwrap();
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

struct MeshPointer {
	incides: Slot,
	positions: Slot,
	normals: Slot,
	tex_coords: Range<u64>,
	indices_count: u32,
}

struct WindowContext<'a> {
	guid_id: usize,
	renderer: Renderer<'a>,
	window: gui::Window
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
	mesh_pointers: HashMap<usize, MeshPointer>
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
			mesh_pointers: HashMap::new()
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
				println!("Saving window");
				let window_attributes = winit::window::Window::default_attributes()
					.with_title(gui_window.title.as_str());
				let window = event_loop.create_window(window_attributes).unwrap();
				let window = Arc::new(window);
				let renderer = Renderer::new(RenderArgs {
					window: window.clone(),
					instance: self.instance.clone(),
					adapter: self.adapter.clone(),
					device: self.device.clone(),
				});
				self.windows.insert(window.id(), WindowContext {
					guid_id: gui_window.id,
					renderer,
					window: gui_window
				});
			}
			Command::Exit => {
				println!("exit");
				event_loop.exit();
			}
			Command::SaveScene(scene) => {
				println!("Saving scene");
				for node in scene.nodes {
					if let Some(mesh) = node.mesh {
						let positions = mesh.positions;
						let normals = mesh.normals;

						match self.mesh_pointers.get(&mesh.id) {
							Some(pointer) => {
								self.position_buffer.write(pointer.positions.offset, bytemuck::cast_slice(&positions));
								self.normal_buffer.write(pointer.normals.offset, bytemuck::cast_slice(&normals));
							}
							None => {
								let positions = self.position_buffer.store(bytemuck::cast_slice(&positions));
								let normals = self.normal_buffer.store(bytemuck::cast_slice(&normals));
								let incides = self.indices_buffer.store(bytemuck::cast_slice(&mesh.indices));
								self.mesh_pointers.insert(mesh.id, MeshPointer {
									positions,
									normals,
									incides,
									tex_coords: 0..0,
									indices_count: 0
								});
							}
						}
					}
				}

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
				match self.windows.get(&window_id) {
					Some(window) => {
						let renderer = &window.renderer;
						match renderer.render() {
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
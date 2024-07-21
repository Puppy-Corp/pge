use std::collections::HashMap;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use thunderdome::Index;
use wgpu::Backends;
use wgpu::Features;
use wgpu::Origin3d;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;
use winit::window::WindowId;
use crate::buffer::*;
use crate::engine_state::EngineState;
use crate::internal_types::EngineEvent;
use crate::texture::create_texture_with_uniform_color;
use crate::texture::load_image;
use crate::types::*;
use crate::renderer::*;
use crate::wgpu_types::*;

pub async fn run<T>(app: T) -> anyhow::Result<()>
where
	T: App
{
	let event_loop: EventLoop<EngineEvent> = EventLoop::<EngineEvent>::with_user_event().build()?;
	let proxy = event_loop.create_proxy();
	let mut engine = Engine::new(app, proxy).await;
	Ok(event_loop.run_app(&mut engine)?)
}

struct GuiBuffers {
	positions_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	color_buffer: wgpu::Buffer,
	position_range: Range<u64>,
	index_range: Range<u64>,
	colors_range: Range<u64>,
	indices_range: Range<u32>,
}

impl GuiBuffers {
	pub fn new(device: Arc<wgpu::Device>) -> Self {
		let vertices = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Gui Vertex Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false
		});
		let indices = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Gui Index Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});
		let color_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Gui Color Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false
		});
		Self {
			positions_buffer: vertices,
			index_buffer: indices,
			color_buffer,
			position_range: 0..0,
			index_range: 0..0,
			colors_range: 0..0,
			indices_range: 0..0,
		}
	}
}

#[derive(Debug)]
struct WindowContext<'a> {
	window_id: Index,
	renderer: Renderer<'a>,
	wininit_window: Arc<winit::window::Window>,
}

struct Engine<'a, T> {
	i: usize,
	app: T,
	state: EngineState,
	adapter: Arc<wgpu::Adapter>,
	instance: Arc<wgpu::Instance>,
	queue: Arc<wgpu::Queue>,
	device: Arc<wgpu::Device>,
	position_buffer: wgpu::Buffer,
	tex_coords_buffer: wgpu::Buffer,
	normal_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
	// instaces: Arena<RawInstance>,
	windows: HashMap<WindowId, WindowContext<'a>>,
	instance_buffer: wgpu::Buffer,
	node_buffer: Buffer,
	point_light_buffer: Buffer,
	last_on_process_time: Instant,
	last_physics_update_time: Instant,
	gui_buffers: HashMap<Index, GuiBuffers>,
	texture_bind_groups: HashMap<Index, wgpu::BindGroup>,
	camera_buffers: HashMap<Index, Buffer>,
	default_texture: wgpu::BindGroup,
	proxy: EventLoopProxy<EngineEvent>,
}

impl<'a, T> Engine<'a, T>
where
	T: App
{
	pub async fn new(app: T, proxy: EventLoopProxy<EngineEvent>) -> Self {
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

		let default_texture = create_texture_with_uniform_color(&device, &queue);

		let position_buffer = RawPositions::create_buffer(&device, 10_000);
		let normal_buffer = RawNormal::create_buffer(&device, 10_000);
		let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Index Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});
		let tex_coords_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Tex Coords Buffer"),
			size: 10_000,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
			mapped_at_creation: false
		});
		let instance_buffer = RawInstance::create_buffer(&device, 10_000);
		let node_buffer = Buffer::new::<RawNode>(&device, 10_000);
		let point_light_buffer = Buffer::new::<RawPointLight>(&device, 10_000);

		Self {
			i: 0,
			app,
			state: EngineState::new(),
			adapter,
			instance,
			queue,
			device,
			position_buffer,
			tex_coords_buffer,
			normal_buffer,
			index_buffer,
			instance_buffer,
			windows: HashMap::new(),
			node_buffer,
			point_light_buffer,
			last_on_process_time: Instant::now(),
			last_physics_update_time: Instant::now(),
			gui_buffers: HashMap::new(),
			texture_bind_groups: HashMap::new(),
			camera_buffers: HashMap::new(),
			default_texture,
			proxy,
		}
	}

	pub fn update_buffers(&mut self) {
		let dt = self.last_on_process_time.elapsed().as_secs_f32();
		self.last_on_process_time = Instant::now();
		self.app.on_process(&mut self.state.state, dt);

		let mut new_textures = Vec::new();
		for (texture_id, texture) in &self.state.state.textures {
			if let None = self.state.textures.get(&texture_id) {
				new_textures.push((texture_id, texture.clone()));
				load_image(self.proxy.clone(), texture.source.clone(), texture_id)
			}
		}
		for t in new_textures {
			self.state.textures.insert(t.0, t.1);
		}

		self.state.update_assets();
		self.state.update_buffers();

		if self.state.all_instances_data.len() > 0 {
			self.queue.write_buffer(&self.instance_buffer, 0, &self.state.all_instances_data);
		}
		// if self.state.all_positions_data.len() > 0 {
		// 	self.queue.write_buffer(&self.position_buffer, 0, &self.state.all_positions_data);
		// }
		// if self.state.all_normals_data.len() > 0 {
		// 	self.queue.write_buffer(&self.normal_buffer, 0, &self.state.all_normals_data);
		// }
		// if self.state.all_tex_coords_data.len() > 0 {
		// 	self.queue.write_buffer(&self.tex_coords_buffer, 0, &self.state.all_tex_coords_data);
		// }
		// if self.state.all_indices_data.len() > 0 {
		// 	self.queue.write_buffer(&self.index_buffer, 0, &self.state.all_indices_data);
		// }
		// if self.state.all_nodes_data.len() > 0 {
		// 	self.queue.write_buffer(&self.node_buffer.buffer, 0, &self.state.all_nodes_data);
		// }
		// if self.state.all_cameras_data.len() > 0 {
		// 	self.queue.write_buffer(&self.camera_buffer.buffer, 0, &self.state.all_cameras_data);
		// }

		if self.state.triangles.vertices.len() > 0 {
			//log::info!("writing vertices {} {:?}", self.state.triangles.vertices.len(), self.state.triangles);
			self.queue.write_buffer(&self.position_buffer, 0, &self.state.triangles.vertices);
		}
		if self.state.triangles.normals.len() > 0 {
			//log::info!("writing normals {}", self.state.triangles.normals.len());
			self.queue.write_buffer(&self.normal_buffer, 0, &self.state.triangles.normals);
		}
		if self.state.triangles.indices.len() > 0 {
			//log::info!("writing indices {}", self.state.triangles.indices.len());
			self.queue.write_buffer(&self.index_buffer, 0, &self.state.triangles.indices);
		}
		if self.state.triangles.tex_coords.len() > 0 {
			//log::info!("writing tex_coords {}", self.state.triangles.tex_coords.len());
			self.queue.write_buffer(&self.tex_coords_buffer, 0, &self.state.triangles.tex_coords);
		}
		if self.state.all_nodes_data.len() > 0 {
			//log::info!("writing nodes {}", self.state.all_nodes_data.len());
			self.queue.write_buffer(&self.node_buffer.buffer, 0, &self.state.all_nodes_data);
		}

		if self.state.all_cameras_data.len() > 0 {
			for (camera_id, data) in &self.state.all_cameras_data {
				let camera_buffer = self.camera_buffers.entry(*camera_id)
					.or_insert(Buffer::new::<RawCamera>(&self.device, 10_000));
				self.queue.write_buffer(&camera_buffer.buffer, 0, data);
			}
		}
		if self.state.all_point_lights_data.len() > 0 {
			self.queue.write_buffer(&self.point_light_buffer.buffer, 0, &self.state.all_point_lights_data);
		}
	}

	fn update_physics(&mut self) {
		let dt = self.last_physics_update_time.elapsed().as_secs_f32();
		self.last_physics_update_time = Instant::now();
		self.state.physics_update(dt);
	}

	fn update_windows(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		for (window_id, window) in self.state.state.windows.iter_mut() {
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

		self.windows.retain(|_, w| self.state.state.windows.contains(w.window_id));

	}

	fn render_windows(&mut self) {
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		for (_, window_ctx) in self.windows.iter_mut() {
			let window = match self.state.state.windows.get(window_ctx.window_id) {
				Some(w) => w,
				None => continue,
			};

			let gui_id = match window.ui {
				Some(id) => id,
				None => continue,
			};

			let gui_buffers = match self.gui_buffers.get(&gui_id) {
				Some(b) => b,
				None => continue,
			};

			let compositor = match self.state.ui_compositors.get(&gui_id) {
				Some(c) => c,
				None => continue,
			};

			let mut views_3d = Vec::new();

			for v in &compositor.views_3d {
				let camera_buffer = match self.camera_buffers.get(&v.camera_id) {
					Some(b) => b,
					None => continue,
				};

				let calls: Vec<_> = self.state.draw_calls.iter().map(|d| {
					let texture_bind_group = match d.texture {
						Some(t) => self.texture_bind_groups.get(&t).unwrap_or(&self.default_texture),
						None => &self.default_texture,
					};
				
					DrawCall {
						index_range: d.index_range.clone(),
						indices_range: d.indices_range.clone(),
						normal_range: d.normal_range.clone(),
						instances_range: d.instances_range.clone(),
						position_range: d.position_range.clone(),
						tex_coords_range: d.tex_coords_range.clone(),
						texture_bind_group,
					}
				}).collect();

				let a = Render3DView {
					x: v.x,
					y: v.y,
					w: v.w,
					h: v.h,
					calls,
					camera_bind_group: &camera_buffer.bind_group,
					node_bind_group: &self.node_buffer.bind_group,
					point_light_bind_group: &self.point_light_buffer.bind_group,
					index_buffer: &self.index_buffer,
					instance_buffer: &self.instance_buffer,
					normal_buffer: &self.normal_buffer,
					tex_coords_buffer: &self.tex_coords_buffer,
					positions_buffer: &self.position_buffer,
				};
				views_3d.push(a);
			}

			let args = RenderArgs {
				encoder: &mut encoder,
				positions_buffer: &gui_buffers.positions_buffer,
				index_buffer: &gui_buffers.index_buffer,
				color_buffer: &gui_buffers.color_buffer,
				views_3d: &views_3d,
				index_range: gui_buffers.index_range.clone(),
				indices_range: gui_buffers.indices_range.clone(),
				position_range: gui_buffers.position_range.clone(),
				color_range: gui_buffers.colors_range.clone(),
			};

			window_ctx.renderer.render(args).unwrap();
		}
		self.queue.submit(std::iter::once(encoder.finish()));
	}

	fn update_ui_buffers(&mut self) {
		for (i, c) in &self.state.ui_compositors {
			let buffers = self.gui_buffers.entry(*i)
				.or_insert(GuiBuffers::new(self.device.clone()));

			if c.positions.len() > 0 {
				let positions_data = bytemuck::cast_slice(&c.positions);
				let positions_data_len = positions_data.len() as u64;
				self.queue.write_buffer(&buffers.positions_buffer, 0, positions_data);
				buffers.position_range = 0..positions_data_len;
			}

			if c.indices.len() > 0 {
				let indices_data = bytemuck::cast_slice(&c.indices);
				let indices_data_len = indices_data.len() as u64;
				self.queue.write_buffer(&buffers.index_buffer, 0, indices_data);
				buffers.index_range = 0..indices_data_len;
				buffers.indices_range = 0..c.indices.len() as u32;
			}

			if c.colors.len() > 0 {
				let colors_data = bytemuck::cast_slice(&c.colors);
				let colors_data_len = colors_data.len() as u64;
				self.queue.write_buffer(&buffers.color_buffer, 0, colors_data);
				buffers.colors_range = 0..colors_data_len;
			}
		}
	}
}

impl<'a, T> ApplicationHandler<EngineEvent> for Engine<'a, T>
where
	T: App
{
	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		self.app.on_create(&mut self.state.state);
		self.state.update_guis();
		self.update_ui_buffers();
		self.update_windows(event_loop);
	}

	fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: EngineEvent) {
		match event {
			EngineEvent::ImageLoaded { texture_id, width, height, data } => {
				log::info!("Image loading: texture_id={:?}, width={}, height={}", texture_id, width, height);
				assert_eq!(data.len(), (width * height * 4) as usize, "Texture data size mismatch");
	
				let texture = self.device.create_texture(
					&wgpu::TextureDescriptor {
						label: Some("Loaded Image"),
						size: wgpu::Extent3d {
							width,
							height,
							depth_or_array_layers: 1,
						},
						mip_level_count: 1,
						sample_count: 1,
						dimension: wgpu::TextureDimension::D2,
						format: wgpu::TextureFormat::Rgba8UnormSrgb,
						usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
						view_formats: &[]
					}
				);
	
				self.queue.write_texture(
					wgpu::ImageCopyTexture {
						texture: &texture,
						mip_level: 0,
						origin: wgpu::Origin3d::ZERO,
						aspect: wgpu::TextureAspect::All,
					},
					&data,
					wgpu::ImageDataLayout {
						offset: 0,
						bytes_per_row: Some(4 * width),
						rows_per_image: Some(height),
					},
					wgpu::Extent3d {
						width,
						height,
						depth_or_array_layers: 1,
					}
				);
	
				let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
				let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
					address_mode_u: wgpu::AddressMode::ClampToEdge,
					address_mode_v: wgpu::AddressMode::ClampToEdge,
					address_mode_w: wgpu::AddressMode::ClampToEdge,
					mag_filter: wgpu::FilterMode::Linear,
					min_filter: wgpu::FilterMode::Linear,
					mipmap_filter: wgpu::FilterMode::Nearest,
					..Default::default()
				});
	
				let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
					layout: &TextureBuffer::create_bind_group_layout(&self.device),
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&texture_view),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: wgpu::BindingResource::Sampler(&sampler),
						},
					],
					label: Some("texture_bind_group"),
				});
	
				self.texture_bind_groups.insert(texture_id, texture_bind_group);
			},
		}
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
					self.app.on_mouse_input(MouseEvent::Moved { dx, dy }, &mut self.state.state);

					if let Some(window) = self.state.state.windows.get(window_ctx.window_id) {
						if window.lock_cursor {
							window_ctx.wininit_window.set_cursor_position(PhysicalPosition::new(middle_x, middle_y)).unwrap();
						}
					}
				}
			}
			WindowEvent::MouseInput { device_id, state, button } => {
				match state {
					winit::event::ElementState::Pressed => self.app.on_mouse_input(MouseEvent::Pressed { button: MouseButton::from(button) }, &mut self.state.state),
					winit::event::ElementState::Released => self.app.on_mouse_input(MouseEvent::Released { button: MouseButton::from(button) }, &mut self.state.state),
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
						if !repeat {
							match physical_key {
								winit::keyboard::PhysicalKey::Code(code) => {
									if KeyCode::Escape == code {
										event_loop.exit();
									}

									match state {
										winit::event::ElementState::Pressed => self.app.on_keyboard_input(KeyboardKey::from(code), KeyAction::Pressed, &mut self.state.state),
										winit::event::ElementState::Released => self.app.on_keyboard_input(KeyboardKey::from(code), KeyAction::Released, &mut self.state.state),
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

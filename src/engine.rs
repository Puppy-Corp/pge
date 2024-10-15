use crate::buffer::*;
use crate::engine_state::EngineState;
use crate::hardware;
use crate::hardware::Hardware;
use crate::hardware::WgpuHardware;
use crate::internal_types::EngineEvent;
use crate::renderer::*;
use crate::texture::load_image;
use crate::types::*;
use crate::wgpu_types::*;
use crate::ArenaId;
use crate::GUIElement;
use crate::Window;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use glam::Mat4;
use wgpu::Backends;
use wgpu::Features;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalPosition;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::keyboard::KeyCode;
use winit::window::WindowId;

pub async fn run<T>(app: T) -> anyhow::Result<()>
where
    T: App,
{
    let event_loop: EventLoop<EngineEvent> = EventLoop::<EngineEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let mut engine = Engine::new(app, proxy).await;
    Ok(event_loop.run_app(&mut engine)?)
}

struct GuiBuffers {
    vertices_buffer: hardware::Buffer,
    index_buffer: hardware::Buffer,
    color_buffer: hardware::Buffer,
    position_range: Range<u64>,
    index_range: Range<u64>,
    colors_range: Range<u64>,
    indices_range: Range<u32>,
}

impl GuiBuffers {
    pub fn new(hardware: &mut WgpuHardware) -> Self {
        let vertices_buffer = hardware.create_buffer("vertices");
		let index_buffer = hardware.create_buffer("indices");
		let color_buffer = hardware.create_buffer("colors");
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

#[derive(Debug)]
struct WindowContext<'a> {
    window_id: ArenaId<Window>,
    renderer: Renderer<'a>,
    wininit_window: Arc<winit::window::Window>,
}

struct Engine<'a, T> {
    app: T,
    state: EngineState,
    hardware: WgpuHardware,
    adapter: Arc<wgpu::Adapter>,
    instance: Arc<wgpu::Instance>,
    queue: Arc<wgpu::Queue>,
    device: Arc<wgpu::Device>,
    vertices_buffer: hardware::Buffer,
    tex_coords_buffer: hardware::Buffer,
    normal_buffer: hardware::Buffer,
    index_buffer: hardware::Buffer,
    windows: HashMap<WindowId, WindowContext<'a>>,
    point_light_buffers: HashMap<ArenaId<Scene>, hardware::Buffer>,
    last_on_process_time: Instant,
    last_physics_update_time: Instant,
    gui_buffers: HashMap<ArenaId<GUIElement>, GuiBuffers>,
    texture_bind_groups: HashMap<ArenaId<Texture>, hardware::Texture>,
    camera_buffers: HashMap<ArenaId<Camera>, hardware::Buffer>,
    default_texture: hardware::Texture,
	default_point_lights: hardware::Buffer,
    proxy: EventLoopProxy<EngineEvent>,
    scene_instance_buffers: HashMap<ArenaId<Scene>, hardware::Buffer>,
	textures: HashSet<ArenaId<Texture>>,
}

impl<'a, T> Engine<'a, T>
where
    T: App,
{
    pub async fn new(app: T, proxy: EventLoopProxy<EngineEvent>) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapters = instance.enumerate_adapters(Backends::all());
        for adapter in adapters {
            println!("Adapter: {:?}", adapter.get_info());
        }
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .expect("Failed to find an appropriate adapter");
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: Features::VERTEX_WRITABLE_STORAGE,
                    required_limits: wgpu::Limits {
                        max_uniform_buffer_binding_size: 20_000_000,
                        max_buffer_size: 100_000_000,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let device = Arc::new(device);
        let queue = Arc::new(queue);
        let adapter = Arc::new(adapter);
        let instance = Arc::new(instance);
        let mut hardware = WgpuHardware::new(device.clone(), queue.clone());

        let data: [u8; 4] = [255, 100, 200, 255]; // pink
        let default_texture = hardware.create_texture("default_texture", &data);

        let vertices_buffer = hardware.create_buffer("vertices");
        let tex_coords_buffer = hardware.create_buffer("tex_coords");
        let normal_buffer = hardware.create_buffer("normals");
        let index_buffer = hardware.create_buffer("indices");

		let default_point_lights = hardware.create_buffer("default_point_lights");

        Self {
            app,
            state: EngineState::new(),
            hardware,
            adapter,
            instance,
            queue,
            device,
            vertices_buffer,
            tex_coords_buffer,
            normal_buffer,
            index_buffer,
            windows: HashMap::new(),
            point_light_buffers: HashMap::new(),
            last_on_process_time: Instant::now(),
            last_physics_update_time: Instant::now(),
            gui_buffers: HashMap::new(),
            texture_bind_groups: HashMap::new(),
            camera_buffers: HashMap::new(),
            default_texture,
            proxy,
            scene_instance_buffers: HashMap::new(),
			default_point_lights,
			textures: HashSet::new(),
        }
    }


    pub fn update_buffers(&mut self) {
        let mut new_textures = Vec::new();
        for (texture_id, texture) in &self.state.state.textures {
            if !self.textures.contains(&texture_id) {
                new_textures.push((texture_id, texture.clone()));
                load_image(self.proxy.clone(), texture.source.clone(), texture_id)
            }
        }
        for t in new_textures {
            self.textures.insert(t.0);
        }

		// let vertices: [[f32; 3]; 4] = [[0.0, 0.5, 0.0], [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.0, 0.0]];
		// let indices: [u16; 4] = [0, 1, 2, 0];


		// let mesh = cube(0.5);
		// println!("indices len: {}", mesh.primitives[0].indices.len());
		// let vertices_data = bytemuck::cast_slice(&mesh.primitives[0].vertices);
		// println!("vertices_data len: {}", vertices_data.len());
		// println!("vertices_data: {:?}", vertices_data);
		// let indices_data = bytemuck::cast_slice(&mesh.primitives[0].indices);
		// println!("indices_data len: {}", indices_data.len());
		// println!("indices_data: {:?}", indices_data);
		// // self.queue.write_buffer(&self.vertices_buffer2, 0, vertices_data);
		// // self.queue.write_buffer(&self.index_buffer2, 0, indices_data);
		// self.vertices_buffer.write(vertices_data);
		// self.index_buffer.write(indices_data);

        if self.state.triangles.vertices.len() > 0 && self.state.triangles.vertices.dirty {
            //log::info!("writing triangle vertices len: {}", self.state.triangles.vertices.len());
    		self.vertices_buffer.write(&self.state.triangles.vertices.data());
            self.state.triangles.vertices.dirty = false;
        }
        if self.state.triangles.indices.len() > 0 && self.state.triangles.indices.dirty {
            //log::info!("writing triangle indices len: {}", self.state.triangles.indices.len());
			self.index_buffer.write(&self.state.triangles.indices.data());
            self.state.triangles.indices.dirty = false;
        }
        if self.state.triangles.tex_coords.len() > 0 && self.state.triangles.tex_coords.dirty {
            log::info!("writing triangle tex coords len: {}", self.state.triangles.tex_coords.len());
            self.tex_coords_buffer
                .write(&self.state.triangles.tex_coords.data());
            self.state.triangles.tex_coords.dirty = false;
        }
        if self.state.triangles.normals.len() > 0 && self.state.triangles.normals.dirty {
            //log::info!("writing triangle normals len: {}", self.state.triangles.normals.len());
            self.normal_buffer.write(&self.state.triangles.normals.data());
            self.state.triangles.normals.dirty = false;
        }

        for (index, b) in &mut self.state.scene_instance_buffers {
            if !b.dirty {
                continue;
            }
			b.dirty = false;

			//log::info!("[{:?}] writing instance buffer len: {}", index, b.len());

            let buff = self.scene_instance_buffers.entry(*index).or_insert(
                self.hardware.create_buffer("scene_instance_buffer"),
            );

            buff.write(&b.data());
        }
        for (id, b) in &mut self.state.scene_point_lights {
            if !b.dirty {
                continue;
            }
			b.dirty = false;

			//log::info!("[{:?}] writing point light buffer len: {}", id.index(), b.len());

            let buff = self.point_light_buffers.entry(*id)
                .or_insert(self.hardware.create_buffer("point_light_buffer"));

            buff.write(&b.data());
        }

        for (id, b) in &mut self.state.camera_buffers {
            if !b.dirty {
                continue;
            }
			b.dirty = false;

            //log::info!("[{:?}] writing camera buffer len: {}", id.index(), b.len());

			let data = Mat4::IDENTITY.to_cols_array();

            let buff = self
                .camera_buffers
                .entry(*id)
                .or_insert(self.hardware.create_buffer("camera_buffer"));

            buff.write(&b.data());
			//buff.write(bytemuck::cast_slice(&data));
        }

		for (i, c) in &self.state.ui_compositors {
            let buffers = self
                .gui_buffers
                .entry(*i)
                .or_insert(GuiBuffers::new(&mut self.hardware));

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
        }
    }

    fn update_windows(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        for (window_id, window) in self.state.state.windows.iter_mut() {
            match self
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
                    let renderer = Renderer::new(NewRendererArgs {
                        window: wininit_window.clone(),
                        instance: self.instance.clone(),
                        adapter: self.adapter.clone(),
                        queue: self.queue.clone(),
                        device: self.device.clone(),
                    })
                    .unwrap();

                    let wininit_window_id = wininit_window.id();
                    let window_ctx = WindowContext {
                        window_id: window_id,
                        wininit_window,
                        renderer,
                    };
                    self.windows.insert(wininit_window_id, window_ctx);
                }
            }
        }

        self.windows
            .retain(|_, w| self.state.state.windows.contains(&w.window_id));
    }

    fn render(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        for (_, window_ctx) in self.windows.iter_mut() {
            let args = match self.state.get_window_render_args(window_ctx.window_id) {
                Some(a) => a,
                None => {
                    //log::error!("Window render args not found");
					continue;
                }
            };

            let gui_buffers = match self.gui_buffers.get(&args.ui) {
                Some(b) => b,
                None => {
                    //panic!("Gui buffers not found");
					continue;
                }
            };

            let mut views = Vec::new();
            for v in &args.views {
                let camera_buffer = match self.camera_buffers.get(&v.camview.camera_id) {
                    Some(b) => b,
                    None => {
                        panic!("Camera buffer not found");
                    }
                };

                let calls = match self.state.get_camera_draw_calls(v.camview.camera_id) {
                    Some(c) => c,
                    None => {
                        //panic!("Draw calls not found");
						continue;
                    }
                };

                let calls: Vec<_> = calls
                    .iter()
                    .map(|d| {
                        let texture_bind_group = match d.texture {
                            Some(t) => {
								match self
                                .texture_bind_groups
                                .get(&t) {
									Some(t) => {
										t
									},
									None => &self.default_texture,
								}
							}
                            None => &self.default_texture,
                        };

                        if d.indices.start == d.indices.end {
							log::info!("call {:?}", d);
                            panic!("Index range is empty");
                        }

                        if d.indices_range.start == d.indices_range.end {
							log::info!("call {:?}", d);
                            panic!("Indices range is empty");
                        }

                        if d.normals.start == d.normals.end {
							log::info!("call {:?}", d);
                            panic!("Normal range is empty");
                        }

                        if d.instances.start == d.instances.end {
							log::info!("call {:?}", d);
                            panic!("Instances range is empty");
                        }

                        if d.vertices.start == d.vertices.end {
							log::info!("call {:?}", d);
                            panic!("vertices range is empty");
                        }

                        if d.tex_coords.start == d.tex_coords.end {
							log::info!("call {:?}", d);
                            panic!("Tex coords range is empty");
                        }

						// log::info!("call {:?}", d);

                        DrawCall {
                            index_range: d.indices.clone(),
                            indices_range: d.indices_range.clone(),
                            normal_range: d.normals.clone(),
                            instances_range: d.instances.clone(),
                            vertices: d.vertices.clone(),
                            tex_coords_range: d.tex_coords.clone(),
                            texture_bind_group,
                        }
                    })
                    .collect();

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

                let a = Render3DView {
                    x: v.camview.x,
                    y: v.camview.y,
                    w: v.camview.w,
                    h: v.camview.h,
                    calls,
                    camera_bind_group: &camera_buffer.bind_group(),
                    point_light_bind_group: &point_light_buffer.bind_group(),
                    index_buffer: &self.index_buffer,
                    instance_buffer: &instance_buffer,
                    normal_buffer: &self.normal_buffer,
                    tex_coords_buffer: &self.tex_coords_buffer,
                    vertices_buffer: &self.vertices_buffer,
                };
                views.push(a);
            }

            let args = RenderArgs {
                encoder: &mut encoder,
                positions_buffer: &gui_buffers.vertices_buffer,
                index_buffer: &gui_buffers.index_buffer,
                color_buffer: &gui_buffers.color_buffer,
                views: &views,
                index_range: gui_buffers.index_range.clone(),
                indices_range: gui_buffers.indices_range.clone(),
                position_range: gui_buffers.position_range.clone(),
                color_range: gui_buffers.colors_range.clone(),
            };

            window_ctx.renderer.render(args).unwrap();
        }
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl<'a, T> ApplicationHandler<EngineEvent> for Engine<'a, T>
where
    T: App,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		log::info!("calling on_create");
        self.app.on_create(&mut self.state.state);
		log::info!("on_create done");
        self.state.process(0.0);
        self.update_windows(event_loop);
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: EngineEvent) {
        match event {
            EngineEvent::ImageLoaded {
                texture_id,
                width,
                height,
                data,
            } => {
                log::info!(
                    "Image loading: texture_id={:?}, width={}, height={}",
                    texture_id,
                    width,
                    height
                );
                assert_eq!(
                    data.len(),
                    (width * height * 4) as usize,
                    "Texture data size mismatch"
                );

                let texture = self.hardware.create_texture("Loaded Image", &data);

                self.texture_bind_groups
                    .insert(texture_id, texture);
            }
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let timer = Instant::now();
        let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));

		self.update_windows(event_loop);
        let dt = self.last_on_process_time.elapsed().as_secs_f32();
        self.last_on_process_time = Instant::now();
        self.app.on_process(&mut self.state.state, dt);
        self.state.process(dt);
        self.update_buffers();
        self.render();
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
            WindowEvent::CursorMoved {
                device_id,
                position,
            } => {
                if let Some(window_ctx) = self.windows.get(&window_id) {
                    let size = &window_ctx.wininit_window.inner_size();
                    let middle_x = size.width as f64 / 2.0;
                    let middle_y = size.height as f64 / 2.0;
                    let dx = position.x - middle_x;
                    let dy = position.y - middle_y;
                    let dx = dx as f32;
                    let dy = dy as f32;
                    self.app
                        .on_mouse_input(MouseEvent::Moved { dx, dy }, &mut self.state.state);

                    if let Some(window) = self.state.state.windows.get(&window_ctx.window_id) {
                        if window.lock_cursor {
                            window_ctx
                                .wininit_window
                                .set_cursor_position(PhysicalPosition::new(middle_x, middle_y))
                                .unwrap();
							window_ctx.wininit_window.set_cursor_visible(false);
                        }
                    }
                }
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => match state {
                winit::event::ElementState::Pressed => self.app.on_mouse_input(
                    MouseEvent::Pressed {
                        button: MouseButton::from(button),
                    },
                    &mut self.state.state,
                ),
                winit::event::ElementState::Released => self.app.on_mouse_input(
                    MouseEvent::Released {
                        button: MouseButton::from(button),
                    },
                    &mut self.state.state,
                ),
            },
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => match event {
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
                                    winit::event::ElementState::Pressed => {
                                        self.app.on_keyboard_input(
                                            KeyboardKey::from(code),
                                            KeyAction::Pressed,
                                            &mut self.state.state,
                                        )
                                    }
                                    winit::event::ElementState::Released => {
                                        self.app.on_keyboard_input(
                                            KeyboardKey::from(code),
                                            KeyAction::Released,
                                            &mut self.state.state,
                                        )
                                    }
                                }
                            }
                            winit::keyboard::PhysicalKey::Unidentified(_) => {}
                        }
                    }
                }
            },
            _ => {}
        }
    }
}

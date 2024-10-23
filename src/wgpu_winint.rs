use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::engine::Engine;
use crate::hardware::Hardware;
use crate::hardware::WgpuHardware;
use crate::App;
use crate::ArenaId;
use crate::KeyboardKey;
use crate::MouseButton;
use crate::MouseEvent;
use crate::Window;

struct WindowContext {
    wininit_window: Arc<winit::window::Window>,
    window_id: ArenaId<Window>,
}

struct PgeWininitHandler<A, H> {
    engine: Engine<A, H>,
    last_on_process_time: Instant,
    windows: HashMap<winit::window::WindowId, WindowContext>,
}

impl<A, H> PgeWininitHandler<A, H> {
    fn new(engine: Engine<A, H>) -> Self {
        Self {
            engine,
            last_on_process_time: Instant::now(),
            windows: HashMap::new(),
        }
    }
}

impl<A, H> ApplicationHandler<()> for PgeWininitHandler<A, H> 
where
    A: App,
    H: Hardware,
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
		/*log::info!("calling on_create");
        self.app.on_create(&mut self.state.state);
		log::info!("on_create done");
        self.state.process(0.0);
        self.update_windows(event_loop);*/

        for w in self.engine.state.state.windows.iter() {
            
        }
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: ()) {
        /*match event {
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
        }*/
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let timer = Instant::now();
        let timer = timer.checked_add(Duration::from_millis(500)).unwrap();
        event_loop.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));
        let dt = self.last_on_process_time.elapsed().as_secs_f32();
        self.last_on_process_time = Instant::now();
        self.engine.render(dt);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // println!("window event");

        let window_ctx = match self.windows.get_mut(&window_id) {
            Some(window) => window,
            None => {
                log::error!("Window not found: {:?}", window_id);
                return;
            }
        };

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
                let size = &window_ctx.wininit_window.inner_size();
                let middle_x = size.width as f64 / 2.0;
                let middle_y = size.height as f64 / 2.0;
                let dx = position.x - middle_x;
                let dy = position.y - middle_y;
                let dx = dx as f32;
                let dy = dy as f32;
                self.engine.on_cursor_moved(window_ctx.window_id, dx, dy);

                /*if let Some(window) = self.state.state.windows.get(&window_ctx.window_id) {
                    if window.lock_cursor {
                        window_ctx
                            .wininit_window
                            .set_cursor_position(PhysicalPosition::new(middle_x, middle_y))
                            .unwrap();
                        window_ctx.wininit_window.set_cursor_visible(false);
                    }
                }*/
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {
                self.engine.on_mouse_button_event(window_ctx.window_id, MouseButton::from(button), state.is_pressed());
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
                                self.engine.on_keyboard_input(window_ctx.window_id, KeyboardKey::from(code), state.is_pressed());
                                /*if KeyCode::Escape == code {
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
                                }*/
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

pub async fn run(app: impl App) -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
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
                    required_features: wgpu::Features::VERTEX_WRITABLE_STORAGE,
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
    let hardware = WgpuHardware::new(instance, device, queue, adapter);
    let engine = Engine::new(app, hardware);
    let mut handler = PgeWininitHandler::new(engine);
    let event_loop = EventLoop::<()>::with_user_event().build()?;
    Ok(event_loop.run_app(&mut handler)?)
}

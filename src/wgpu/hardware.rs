use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use futures::executor::block_on;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoop;

use crate::engine::Engine;
use crate::hardware::Hardware;
use crate::hardware::PipelineHandle;
use crate::hardware::RenderEncoder;
use crate::hardware::TextureHandle;
use super::wgpu_types::*;
use crate::App;
use crate::ArenaId;
use crate::KeyboardKey;
use crate::MouseButton;
use crate::Window;

enum UserEvent{
    CreateWindow {
        window_id: ArenaId<Window>,
        name: String,
    },
    DestroyWindow {
        window_id: ArenaId<Window>,
    },
    CreatePipeline {
        window_id: ArenaId<Window>,
        name: String,
        pipeline_id: u32
    },
    Render {
        window_id: ArenaId<Window>,
        encoder: RenderEncoder,
    }
}

struct WindowContext<'a> {
    wininit_window: Arc<winit::window::Window>,
    window_id: ArenaId<Window>,
    surface: Arc<wgpu::Surface<'a>>,
}

struct PipelineContext {
    pipeline: Arc<wgpu::RenderPipeline>,
    depth_texture_view: Arc<wgpu::TextureView>,
}

struct PgeWininitHandler<'a, A, H> {
    engine: Engine<A, H>,
    last_on_process_time: Instant,
    windows: Vec<WindowContext<'a>>,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance: Arc<wgpu::Instance>,
    pipelines: Vec<PipelineContext>,
}

impl<'a, A, H> PgeWininitHandler<'a, A, H> {
    fn new(engine: Engine<A, H>, adapter: Arc<wgpu::Adapter>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, instance: Arc<wgpu::Instance>) -> Self {
        Self {
            engine,
            last_on_process_time: Instant::now(),
            windows: Vec::new(),
            adapter,
            device,
            queue,
            instance,
            pipelines: Vec::new(),
        }
    }
}

impl<'a, A, H> ApplicationHandler<UserEvent> for PgeWininitHandler<'a, A, H> 
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
    }

    fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::CreateWindow {
                name,
                window_id,
            } => {
                let window_attributes = winit::window::Window::default_attributes().with_title(&name);
                let wininit_window = event_loop.create_window(window_attributes).unwrap();
                let wininit_window = Arc::new(wininit_window);
                let surface = Arc::new(self.instance.create_surface(wininit_window.clone()).unwrap());
                let wininit_window_id = wininit_window.id();
                let window_ctx = WindowContext {
                    surface,
                    window_id: window_id,
                    wininit_window,
                };
                self.windows.push(window_ctx);
            }
            UserEvent::DestroyWindow {
                window_id,
            } => {

            }
            UserEvent::CreatePipeline {
                window_id,
                name,
                pipeline_id,
            } => {
                let window_ctx = match self.windows.iter().find(|window| window.window_id == window_id) {
                    Some(window) => window,
                    None => {
                        log::error!("Window not found: {:?}", window_id);
                        return;
                    }
                };
                let surface_caps = window_ctx.surface.get_capabilities(&self.adapter);
                let surface_format = surface_caps
                    .formats
                    .iter()
                    .copied()
                    .find(|f| f.is_srgb())
                    .unwrap_or(surface_caps.formats[0]);

                let size = window_ctx.wininit_window.inner_size();
        
                let config = wgpu::SurfaceConfiguration {
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    format: surface_format,
                    width: size.width,
                    height: size.height,
                    present_mode: surface_caps.present_modes[0],
                    alpha_mode: surface_caps.alpha_modes[0],
                    view_formats: vec![],
                    desired_maximum_frame_latency: 1,
                };
        
                window_ctx.surface.configure(&self.device, &config);
                let depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                    label: None,
                    size: wgpu::Extent3d {
                        width: size.width,
                        height: size.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: Default::default(),
                });
        
                let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                let camera_bind_group_layout = RawCamera::create_bind_group_layout(&self.device);
                let point_light_bind_group_layout = RawPointLight::create_bind_group_layout(&self.device);
                let texture_layout = TextureBuffer::create_bind_group_layout(&self.device);
            
                let tex_coords_layout = wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<TexCoords>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 2,
                    }],
                };
            
                let layouts = &[&camera_bind_group_layout, &point_light_bind_group_layout, &texture_layout];
                let buffers = &[Vertices::desc(), RawInstance::desc(), Normals::desc(), tex_coords_layout];
                let shader_source = wgpu::ShaderSource::Wgsl(include_str!("../shaders/3d_shader.wgsl").into());
                log::info!("Creating pipeline for shader: {:?}", shader_source);
            
                let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Shader"),
                    source: shader_source
                });
            
                let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: layouts,
                    push_constant_ranges: &[],
                });
            
                let depth_stencil_state = wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24PlusStencil8,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                };
            
                let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers,
                        compilation_options: Default::default(),
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: wgpu::TextureFormat::Bgra8UnormSrgb,
                            blend: Some(wgpu::BlendState {
                                color: wgpu::BlendComponent::REPLACE,
                                alpha: wgpu::BlendComponent::REPLACE,
                            }),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                        compilation_options: Default::default(),
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None,
                        polygon_mode: wgpu::PolygonMode::Fill,
                        unclipped_depth: false,
                        conservative: false,
                    },
                    depth_stencil: Some(depth_stencil_state),
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    multiview: None,
                });

                let pipeline_ctx = PipelineContext {
                    pipeline: Arc::new(render_pipeline),
                    depth_texture_view: Arc::new(depth_texture_view),
                };
                self.pipelines.push(pipeline_ctx);
            }
            UserEvent::Render {
                window_id,
                encoder,
            } => {
                let window_ctx = match self.windows.iter().find(|window| window.window_id == window_id) {
                    Some(window) => window,
                    None => {
                        log::error!("Window not found: {:?}", window_id);
                        return;
                    }
                };
                let output = window_ctx.surface.get_current_texture().unwrap();
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut wgpu_encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
                for pass in encoder.passes {
                    let pipeline = pass.pipeline.unwrap();
                    let mut wgpu_pass = wgpu_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &pipeline_ctx.depth_texture_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        ..Default::default()
                    });
        
                    wgpu_pass.set_pipeline(&pipeline.pipeline);
        
        
                    for subpass in &pass.subpasses {
                        for (slot, texture) in &pass.textures {
                            wgpu_pass.set_bind_group(*slot, texture.bind_group(), &[]);
                        }
                        for (slot, buffer) in &subpass.buffers {
                            let bind_group = buffer.bind_group();
                            wgpu_pass.set_bind_group(*slot, &bind_group, &[]);
                        }
                        for (slot, buffer) in &subpass.vertex_buffers {
                            wgpu_pass.set_vertex_buffer(*slot, buffer.slice());
                        }
                        if let Some(index_buffer) = &subpass.index_buffer {
                            wgpu_pass.set_index_buffer(index_buffer.slice(), wgpu::IndexFormat::Uint16);
                        }
                        let indices = subpass.indices.clone().unwrap();
                        let instances = subpass.instances.clone().unwrap();
                        wgpu_pass.draw_indexed(indices.clone(), 0, instances.clone());
                    }
                }
                self.queue.submit(std::iter::once(wgpu_encoder.finish()));
                output.present();
            }
        }
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

pub fn run(app: impl App) -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let adapters = instance.enumerate_adapters(wgpu::Backends::all());
    for adapter in adapters {
        println!("Adapter: {:?}", adapter.get_info());
    }
    let adapter = block_on(instance
        .request_adapter(&wgpu::RequestAdapterOptions::default()))
        .expect("Failed to find an appropriate adapter");
    let (device, queue) = block_on(adapter
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
        ))
        .expect("Failed to create device");

    let device = Arc::new(device);
    let queue = Arc::new(queue);
    let adapter = Arc::new(adapter);
    let instance = Arc::new(instance);

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let proxy = event_loop.create_proxy();
    let hardware = WgpuHardware::new(proxy, instance.clone(), adapter.clone(), device.clone(), queue.clone());
    let engine = Engine::new(app, hardware);
    let mut handler = PgeWininitHandler::new(engine, adapter, device, queue, instance);
    Ok(event_loop.run_app(&mut handler)?)
}

struct BufferContext {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

struct TextureContext {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}

pub struct WgpuHardware {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
    buffers: Vec<BufferContext>,
    textures: Vec<TextureContext>,
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    pipeline_id: u32,
}

impl WgpuHardware {
    pub fn new(proxy: winit::event_loop::EventLoopProxy<UserEvent>, instance: Arc<wgpu::Instance>, adapter: Arc<wgpu::Adapter>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            instance,
            device,
            queue,
            adapter,
            buffers: Vec::new(),
            textures: Vec::new(),
            proxy,
            pipeline_id: 1
        }
    }
}

impl Hardware for WgpuHardware {
    fn create_buffer(&mut self, name: &str) -> crate::hardware::Buffer {
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(name),
            size: 10_000_000,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::INDEX | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let layout = self.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
            label: Some("Buffer Bind Group"),
        });

        let id = self.buffers.len();
        self.buffers.push(BufferContext {
            buffer,
            bind_group,
        });
        
        crate::hardware::Buffer::new(id as u32)
    }

    fn create_texture(&mut self, name: &str, data: &[u8]) -> TextureHandle {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: Default::default(),
        });
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
                bytes_per_row: Some(4),
                rows_per_image: None,
            },
            size
        );
    
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
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
        self.textures.push(TextureContext {
            texture,
            bind_group: texture_bind_group,
        });
        TextureHandle {
            id: self.textures.len() as u32 - 1,
        }
    }

    fn create_window(&mut self, window_id: ArenaId<Window>, window: &Window) -> ArenaId<Window> {
        self.proxy.send_event(UserEvent::CreateWindow {
            window_id,
            name: window.title.clone(),
        });
        window_id
    }
    
    fn destroy_window(&mut self, window_id: ArenaId<Window>) {
        self.proxy.send_event(UserEvent::DestroyWindow {
            window_id,
        });
    }

    fn create_pipeline(&mut self, name: &str, window_id: ArenaId<Window>) -> PipelineHandle {
        self.proxy.send_event(UserEvent::CreatePipeline {
            window_id,
            name: name.to_string(),
        });

        let pipeline_id = self.pipeline_id;
        self.pipeline_id += 1;
        PipelineHandle {
            id: pipeline_id,
        }
    }

    fn render(&mut self, encoder: RenderEncoder, window_id: ArenaId<Window>) {
        self.proxy.send_event(UserEvent::Render {
            window_id,
            encoder,
        });
    }
}

/*#[derive(Debug, Clone)]
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    queue: Arc<wgpu::Queue>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Texture {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}*/
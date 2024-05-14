use std::iter;

use winit::window::Window;

use super::buffers::create_camera_bind_group;
use super::buffers::create_camera_bind_group_layout;
use super::buffers::create_camera_empty_buffer;
use super::buffers::create_empty_normal_buffer;
use super::types::Normal;
use super::types::Position;
use super::types::TexCoords;

pub struct Renderer<'a> {
	surface: wgpu::Surface<'a>,
	device: wgpu::Device,
    queue: wgpu::Queue,
	camera_buffer: wgpu::Buffer,
	camera_bind_group: wgpu::BindGroup,
	position_buffer: wgpu::Buffer,
	normal_buffer: wgpu::Buffer,
	tex_coords_buffer: wgpu::Buffer,
	render_pipeline: wgpu::RenderPipeline,
}

impl<'a> Renderer<'a> {
	pub async fn new(window: &'a Window) -> Self {
		let size = window.inner_size();
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
			..Default::default()
        });
		let surface = unsafe { instance.create_surface(window).unwrap() };
		let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
		let (device, queue) = adapter
			.request_device(
				&wgpu::DeviceDescriptor {
					label: None,
					..Default::default()
				},
				None, // Trace path
			)
			.await
			.unwrap();
		let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
			desired_maximum_frame_latency: 1
        };
        surface.configure(&device, &config);

		let vertex_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("vertex.wgsl").into()),
        });
		let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("fragment.wgsl").into()),
		});

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "vs_main",
                buffers: &[Position::desc(), Normal::desc(), TexCoords::desc()],
				compilation_options: Default::default()
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
				compilation_options: Default::default()
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

		let camera_buffer = create_camera_empty_buffer(&device);
		let camera_bind_group_layout = create_camera_bind_group_layout(&device);
		let camera_bind_group = create_camera_bind_group(&device, &camera_bind_group_layout, &camera_buffer);
		let normal_buffer = create_empty_normal_buffer(&device, 500);
		let position_buffer = create_empty_normal_buffer(&device, 500);
		let tex_coords_buffer = create_empty_normal_buffer(&device, 500);
		
		Self {
			surface,
			device,
			queue,
			camera_buffer,
			camera_bind_group,
			normal_buffer,
			position_buffer,
			tex_coords_buffer,
			render_pipeline,
		}
	}

	pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

		self.queue.submit(iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}
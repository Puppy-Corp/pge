use std::ops::Range;
use std::sync::Arc;
use winit::window::Window;
use crate::wgpu_types::*;

pub struct RenderPipelineBuilder {
	pub instance: Arc<wgpu::Instance>,
	pub window: Arc<Window>,
	pub queue: Arc<wgpu::Queue>,
	pub device: Arc<wgpu::Device>,
	pub adapter: Arc<wgpu::Adapter>,
	pub node_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	pub camera_bind_group_layout: Arc<wgpu::BindGroupLayout>,
}

impl RenderPipelineBuilder {
	pub fn build(self) -> Renderer<'static> {
		log::info!("creating pipeline");
		let size = self.window.inner_size();
		let surface =  self.instance.create_surface(self.window.clone()).unwrap();
		let surface_caps = surface.get_capabilities(&self.adapter);
		log::info!("surface caps: {:?}", surface_caps);
		let surface_format = surface_caps
			.formats
			.iter()
			.copied()
			.find(|f| f.is_srgb())
			.unwrap_or_else(|| surface_caps.formats[0]);
		log::info!("surface format: {:?}", surface_format);
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

		log::info!("config {:?}", config);
		surface.configure(&self.device, &config);
		log::info!("configured surface");
		let shader = self.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
		});
		let render_pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[&self.camera_bind_group_layout],
			push_constant_ranges: &[],
		});
		log::info!("creating render pipeline");
		let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[Position::desc(), RawInstance::desc()],
				compilation_options: Default::default()
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
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
		log::info!("created render pipeline");
	
		Renderer {
			surface: surface,
			device: self.device.clone(),
			pipeline: render_pipeline,
			queue: self.queue.clone(),
		}
	}
}

#[derive(Debug)]
pub struct Renderer<'a> {
	pub surface: wgpu::Surface<'a>,
	pub device: Arc<wgpu::Device>,
	pub pipeline: wgpu::RenderPipeline,
	pub queue: Arc<wgpu::Queue>,
}

pub struct DrawInstruction {
	pub position_range: Range<u64>,
	pub index_range: Range<u64>,
	pub instances_range: Range<u32>,
	pub indices_range: Range<u32>,
}

pub struct RenderArgs<'a> {
	pub instructions: &'a [DrawInstruction],
	pub node_bind_group: &'a wgpu::BindGroup,
	pub camera_bind_group: &'a wgpu::BindGroup,
	pub positions_buffer: &'a wgpu::Buffer,
	pub indices_buffer: &'a wgpu::Buffer,
	pub instance_buffer: &'a wgpu::Buffer,
}

impl Renderer<'_> {
	pub fn render(&self, args: RenderArgs) -> anyhow::Result<()> {
		println!("rendering");
		let output = self.surface.get_current_texture()?;
		let view  = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder"),
		});
		
		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.5,
                            g: 1.0,
                            b: 1.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
				depth_stencil_attachment: None,
				..Default::default()
			});

			render_pass.set_pipeline(&self.pipeline);
			//render_pass.set_bind_group(0, &args.node_bind_group, &[]);
			render_pass.set_bind_group(0, &args.camera_bind_group, &[]);

			for draw in args.instructions {
				println!("draw");
				// println!("instances range {:?}", instruction.instances_range);
				// println!("position range {:?}", instruction.position_range);
				println!("indices range {:?}", draw.indices_range);
				println!("index range {:?}", draw.index_range);
				render_pass.set_vertex_buffer(0, args.positions_buffer.slice(draw.position_range.clone()));
				render_pass.set_vertex_buffer(1, args.positions_buffer.slice(..));
				render_pass.set_vertex_buffer(5, args.instance_buffer.slice(..));
				render_pass.set_index_buffer(args.indices_buffer.slice(draw.index_range.clone()), wgpu::IndexFormat::Uint16);
				render_pass.draw_indexed(draw.indices_range.clone(), 0, draw.instances_range.clone());
			}
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();
		Ok(())
	}
}

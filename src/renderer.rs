use std::ops::Range;
use std::sync::Arc;
use wgpu::TextureUsages;

use crate::buffer::*;
use crate::buffers::*;
use crate::wgpu_types::*;


#[derive(Debug, Default, Hash)]
pub struct DrawInstruction {
	pub position_range: Range<u64>,
	pub index_range: Range<u64>,
	pub normal_range: Range<u64>,
	pub instances_range: Range<u32>,
	pub indices_range: Range<u32>,
}

pub struct RenderArgs<'a> {
	pub instructions: &'a mut dyn Iterator<Item = &'a DrawInstruction>,
	pub node_buffer: &'a FixedBuffer<RawNode>,
	pub camera_buffer: &'a FixedBuffer<RawCamera>,
	pub point_light_buffer: &'a FixedBuffer<RawPointLight>,
	pub positions_buffer: &'a wgpu::Buffer,
	pub index_buffer: &'a wgpu::Buffer,
	pub normal_buffer: &'a wgpu::Buffer,
	pub instance_buffer: &'a wgpu::Buffer,
	pub encoder: &'a mut wgpu::CommandEncoder,
}

pub struct NewRendererArgs {
	pub window: Arc<winit::window::Window>,
	pub instance: Arc<wgpu::Instance>,
	pub adapter: Arc<wgpu::Adapter>,
	pub queue: Arc<wgpu::Queue>,
	pub device: Arc<wgpu::Device>,
}

#[derive(Debug)]
pub struct Renderer<'a> {
	pub surface: wgpu::Surface<'a>,
	pub device: Arc<wgpu::Device>,
	pub pipeline: wgpu::RenderPipeline,
	pub queue: Arc<wgpu::Queue>,
	pub depth_texture_view: wgpu::TextureView,
}

impl Renderer<'_> {
	pub fn new(args: NewRendererArgs) -> anyhow::Result<Self> {
		log::info!("creating pipeline");
		let size = args.window.inner_size();
		let surface =  args.instance.create_surface(args.window.clone())?;
		let surface_caps = surface.get_capabilities(&args.adapter);
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

		let depth_texture = args.device.create_texture(&wgpu::TextureDescriptor {
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
			usage: TextureUsages::RENDER_ATTACHMENT,
			view_formats: Default::default(),
		});
	
		let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

		log::info!("config {:?}", config);
		surface.configure(&args.device, &config);
		log::info!("configured surface");
		let shader = args.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/shader.wgsl").into()),
		});

		let camera_bind_group_layout = RawCamera::create_bind_group_layout(&args.device);
		let node_bind_group_layout = RawNode::create_bind_group_layout(&args.device);
		let point_light_bind_group_layout = RawPointLight::create_bind_group_layout(&args.device);

		let render_pipeline_layout = args.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[
				&camera_bind_group_layout, 
				&node_bind_group_layout, 
				&point_light_bind_group_layout
			],
			push_constant_ranges: &[],
		});

		let depth_stencil_state = wgpu::DepthStencilState {
			format: wgpu::TextureFormat::Depth24PlusStencil8,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		};

		log::info!("creating render pipeline");
		let render_pipeline = args.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[RawPositions::desc(), RawInstance::desc(), RawNormal::desc()],
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
				cull_mode: Some(wgpu::Face::Back),
				// Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
				// or Features::POLYGON_MODE_POINT
				polygon_mode: wgpu::PolygonMode::Fill,
				// Requires Features::DEPTH_CLIP_CONTROL
				unclipped_depth: false,
				// Requires Features::CONSERVATIVE_RASTERIZATION
				conservative: false,
			},
			depth_stencil: Some(depth_stencil_state),
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

		Ok(Self {
			surface: surface,
			device: args.device,
			pipeline: render_pipeline,
			queue: args.queue,
			depth_texture_view: depth_texture_view,
		})
	}

	pub fn render(&self, args: RenderArgs) -> anyhow::Result<()> {
		let output = self.surface.get_current_texture()?;
		let view  = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

		{
			let mut render_pass = args.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
				depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
					view: &self.depth_texture_view,
					depth_ops: Some(wgpu::Operations {
						load: wgpu::LoadOp::Clear(1.0),
						store: wgpu::StoreOp::Store,
					}),
					stencil_ops: None,
				}),
				..Default::default()
			});

			render_pass.set_pipeline(&self.pipeline);
			//render_pass.set_bind_group(0, &args.node_bind_group, &[]);
			render_pass.set_bind_group(0, &args.camera_buffer.bind_group(), &[]);
			render_pass.set_bind_group(1, &args.node_buffer.bind_group(), &[]);
			render_pass.set_bind_group(2, &args.point_light_buffer.bind_group(), &[]);

			for draw in args.instructions {
				render_pass.set_vertex_buffer(0, args.positions_buffer.slice(draw.position_range.clone()));
				//render_pass.set_vertex_buffer(1, args.positions_buffer.slice(..));
				render_pass.set_vertex_buffer(1, args.instance_buffer.slice(..));
				render_pass.set_vertex_buffer(2, args.normal_buffer.slice(draw.normal_range.clone()));
				// render_pass.set_vertex_buffer(3, args.tex_coords_buffer.slice(..));
				render_pass.set_index_buffer(args.index_buffer.slice(draw.index_range.clone()), wgpu::IndexFormat::Uint16);
				render_pass.draw_indexed(draw.indices_range.clone(), 0, draw.instances_range.clone());
			}
		}

		output.present();
		Ok(())
	}
}
use std::ops::Range;
use std::sync::Arc;
use wgpu::util::RenderEncoder;
use wgpu::TextureFormat;
use wgpu::TextureUsages;
use winit::dpi::PhysicalSize;

use crate::buffer::*;
use crate::wgpu_types::*;

struct CreatePipelineArgs<'a> {
    device: &'a wgpu::Device,
    surface: &'a wgpu::Surface<'a>,
    format: TextureFormat,
    size: winit::dpi::PhysicalSize<u32>,
}

fn create_pipeline(args: CreatePipelineArgs, shader_source: wgpu::ShaderSource, layouts: &[&wgpu::BindGroupLayout], buffers: &[wgpu::VertexBufferLayout]) -> wgpu::RenderPipeline {
    log::info!("Creating pipeline for shader: {:?}", shader_source);
    
    let shader = args.device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: shader_source
    });

    let render_pipeline_layout = args.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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

    let render_pipeline = args.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                format: args.format,
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

    log::info!("Created pipeline");
    render_pipeline
}

fn create_3d_pipeline(args: CreatePipelineArgs) -> wgpu::RenderPipeline {
    let camera_bind_group_layout = RawCamera::create_bind_group_layout(&args.device);
    let point_light_bind_group_layout = RawPointLight::create_bind_group_layout(&args.device);
    let texture_layout = TextureBuffer::create_bind_group_layout(&args.device);

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
	let shader_source = wgpu::ShaderSource::Wgsl(include_str!("./shaders/3d_shader.wgsl").into());
    create_pipeline(args, shader_source, layouts, buffers)
}

fn create_gui_pipeline(args: CreatePipelineArgs) -> wgpu::RenderPipeline {
    let color_layout = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertices>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[wgpu::VertexAttribute {
            offset: 0,
            format: wgpu::VertexFormat::Float32x4,
            shader_location: 1,
        }],
    };

    let layouts = &[];
    let buffers = &[Vertices::desc(), color_layout];

	let shader_source = wgpu::ShaderSource::Wgsl(include_str!("./shaders/gui_shader.wgsl").into());

    create_pipeline(args, shader_source, layouts, buffers)
}

#[derive(Debug)]
pub struct DrawCall<'a> {
    pub texture_bind_group: &'a wgpu::BindGroup,
    pub vertices: Range<u64>,
    pub index_range: Range<u64>,
    pub normal_range: Range<u64>,
    pub instances_range: Range<u32>,
    pub indices_range: Range<u32>,
    pub tex_coords_range: Range<u64>,
}

#[derive(Debug)]
pub struct Render3DView<'a> {
    pub vertices_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
    pub normal_buffer: &'a wgpu::Buffer,
    pub tex_coords_buffer: &'a wgpu::Buffer,
    pub instance_buffer: &'a wgpu::Buffer,
    pub camera_bind_group: &'a wgpu::BindGroup,
    pub point_light_bind_group: &'a wgpu::BindGroup,
    pub calls: Vec<DrawCall<'a>>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Debug)]
pub struct RenderArgs<'a> {
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub positions_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
    pub color_buffer: &'a wgpu::Buffer,
    pub views: &'a [Render3DView<'a>],
    pub position_range: Range<u64>,
    pub index_range: Range<u64>,
    pub indices_range: Range<u32>,
    pub color_range: Range<u64>,
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
    pub pipeline_gui: wgpu::RenderPipeline,
    pub pipeline_3d: wgpu::RenderPipeline,
    pub queue: Arc<wgpu::Queue>,
    pub depth_texture_view: wgpu::TextureView,
    size: PhysicalSize<u32>,
}

impl Renderer<'_> {
    pub fn new(args: NewRendererArgs) -> anyhow::Result<Self> {
        let size = args.window.inner_size();
        let surface = args.instance.create_surface(args.window)?;
        let surface_caps = surface.get_capabilities(&args.adapter);
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
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&args.device, &config);

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

        let pipeline_3d = create_3d_pipeline(CreatePipelineArgs {
            device: &args.device,
            surface: &surface,
            format: config.format,
            size,
        });

        let pipeline_gui = create_gui_pipeline(CreatePipelineArgs {
            device: &args.device,
            surface: &surface,
            format: config.format,
            size,
        });

        Ok(Self {
            surface,
            device: args.device,
            pipeline_3d,
            pipeline_gui,
            queue: args.queue,
            depth_texture_view,
            size,
        })
    }

    pub fn render(&self, args: RenderArgs) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = args.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            for view in args.views {
                let vx = view.x * self.size.width as f32;
                let vy = view.y * self.size.height as f32;
                let vw = view.w * self.size.width as f32;
                let vh = view.h * self.size.height as f32;

                render_pass.set_viewport(vx, vy, vw, vh, 0.0, 1.0);
                render_pass.set_pipeline(&self.pipeline_3d);
                render_pass.set_bind_group(0, &view.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &view.point_light_bind_group, &[]);

                for call in &view.calls {
					// log::info!("call: {:?}", call);
                    render_pass.set_bind_group(2, &call.texture_bind_group, &[]);
                    render_pass.set_vertex_buffer(0, view.vertices_buffer.slice(call.vertices.clone()));
                    render_pass.set_vertex_buffer(1, view.instance_buffer.slice(..));
                    render_pass.set_vertex_buffer(2, view.normal_buffer.slice(call.normal_range.clone()));
                    render_pass.set_vertex_buffer(3, view.tex_coords_buffer.slice(call.tex_coords_range.clone()));
                    render_pass.set_index_buffer(view.index_buffer.slice(call.index_range.clone()), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(call.indices_range.clone(), 0, call.instances_range.clone());
                }
            }

            let position_count = args.position_range.clone().count();
            if position_count > 0 {
                render_pass.set_pipeline(&self.pipeline_gui);
                render_pass.set_vertex_buffer(0, args.positions_buffer.slice(args.position_range));
                render_pass.set_vertex_buffer(1, args.color_buffer.slice(args.color_range));
                render_pass.set_index_buffer(args.index_buffer.slice(args.index_range), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(args.indices_range.clone(), 0, 0..1);
            }
        }

        output.present();
        Ok(())
    }
}

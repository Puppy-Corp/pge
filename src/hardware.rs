use std::ops::Range;
use std::sync::Arc;

use winit::dpi::PhysicalSize;

use crate::wgpu_types::BindableBufferRecipe;
use crate::wgpu_types::Normals;
use crate::wgpu_types::RawCamera;
use crate::wgpu_types::RawInstance;
use crate::wgpu_types::RawPointLight;
use crate::wgpu_types::TexCoords;
use crate::wgpu_types::TextureBuffer;
use crate::wgpu_types::Vertices;

pub trait Hardware {
    fn create_buffer(&mut self, name: &str) -> Buffer;
    fn create_texture(&mut self, name: &str, data: &[u8]) -> Texture;
    fn create_pipeline(&mut self, name: &str, surface: Arc<wgpu::Surface>, size: PhysicalSize<u32>) -> Arc<Pipeline>;
    fn submit(&mut self, encoder: RenderEncoder, surface: Arc<wgpu::Surface>);
}

#[derive(Debug, Clone)]
pub struct Buffer {
    buffer: Arc<wgpu::Buffer>,
    queue: Arc<wgpu::Queue>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Buffer {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn slice(&self, range: Range<u64>) -> BufferSlice {
        BufferSlice {
            buffer: self.buffer.clone(),
            range,
        }
    }

    pub fn full(&self) -> BufferSlice {
        BufferSlice {
            buffer: self.buffer.clone(),
            range: 0..self.buffer.size(),
        }
    }

    pub fn write(&self, data: &[u8]) {
        self.queue.write_buffer(&self.buffer, 0, data);
    }
}

#[derive(Debug, Clone)]
pub struct BufferSlice {
    buffer: Arc<wgpu::Buffer>,
    range: Range<u64>,
}

impl BufferSlice {
    pub fn slice(&self) -> wgpu::BufferSlice {
        self.buffer.slice(self.range.clone())
    }
}

#[derive(Debug, Clone)]
pub struct Texture {
    texture: Arc<wgpu::Texture>,
    queue: Arc<wgpu::Queue>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Texture {
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    depth_texture_view: wgpu::TextureView,
}

pub struct RenderEncoder {
    passes: Vec<RenderPass>
}

impl RenderEncoder {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    pub fn begin_render_pass(&mut self) -> &mut RenderPass {
        let render_pass = RenderPass::default();
        self.passes.push(render_pass);
        self.passes.last_mut().unwrap()
    }
}   

#[derive(Default)]
pub struct RenderPass {
    subpasses: Vec<Subpass>,
    vertex_buffers: Vec<(u32, BufferSlice)>,
    index_buffer: Option<BufferSlice>,
    pipeline: Option<Arc<Pipeline>>,
    buffers: Vec<(u32, Buffer)>,
    textures: Vec<(u32, Texture)>,
    indices: Option<Range<u32>>,
    instances: Option<Range<u32>>,
}

impl RenderPass {
    pub fn bind_buffer(&mut self, slot: u32, buffer: Buffer) {
        self.buffers.push((slot, buffer));
    }

    pub fn bind_texture(&mut self, slot: u32, texture: Texture) {
        self.textures.push((slot, texture));
    }

    pub fn set_vertex_buffer(&mut self, slot: u32, buffer: BufferSlice) {
        self.vertex_buffers.push((slot, buffer));
    }

    pub fn set_index_buffer(&mut self, buffer: BufferSlice) {
        self.index_buffer = Some(buffer);
    }

    pub fn draw_indexed(&mut self, indices: Range<u32>, instances: Range<u32>) {
        self.indices = Some(indices);
        self.instances = Some(instances);
        let subpass = Subpass {
            vertex_buffers: self.vertex_buffers.clone(),
            index_buffer: self.index_buffer.clone(),
            pipeline: self.pipeline.clone(),
            buffers: self.buffers.clone(),
            indices: self.indices.clone(),
            instances: self.instances.clone(),
        };
        self.subpasses.push(subpass);
    }

    pub fn set_pipeline(&mut self, pipeline: Arc<Pipeline>) {
        self.pipeline = Some(pipeline);
    }
}

pub struct Subpass {
    vertex_buffers: Vec<(u32, BufferSlice)>,
    index_buffer: Option<BufferSlice>,
    pipeline: Option<Arc<Pipeline>>,
    buffers: Vec<(u32, Buffer)>,
    indices: Option<Range<u32>>,
    instances: Option<Range<u32>>,
}

pub struct WgpuHardware {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    instance: Arc<wgpu::Instance>,
    adapter: Arc<wgpu::Adapter>,
}

impl WgpuHardware {
    pub fn new(instance: Arc<wgpu::Instance>, device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, adapter: Arc<wgpu::Adapter>) -> Self {
        Self {
            instance,
            device,
            queue,
            adapter,
        }
    }
}

impl Hardware for WgpuHardware {
    fn create_buffer(&mut self, name: &str) -> Buffer {
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
        Buffer {
            buffer: Arc::new(buffer),
            queue: self.queue.clone(),
            bind_group: Arc::new(bind_group),
        }
    }

    fn create_texture(&mut self, name: &str, data: &[u8]) -> Texture {
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
        Texture {
            texture: Arc::new(texture),
            queue: self.queue.clone(),
            bind_group: Arc::new(texture_bind_group),
        }
    }

    fn create_pipeline(&mut self, name: &str, surface: Arc<wgpu::Surface>, size: PhysicalSize<u32>) -> Arc<Pipeline> {
        let surface_caps = surface.get_capabilities(&self.adapter);
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

        surface.configure(&self.device, &config);
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
        let shader_source = wgpu::ShaderSource::Wgsl(include_str!("./shaders/3d_shader.wgsl").into());
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
    
        log::info!("Created pipeline");
        Arc::new(Pipeline {
            pipeline: render_pipeline,
            depth_texture_view,
        })
    }

    fn submit(&mut self, encoder: RenderEncoder, surface: Arc<wgpu::Surface>) {
        let output = surface.get_current_texture().unwrap();
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
                    view: &pipeline.depth_texture_view,
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



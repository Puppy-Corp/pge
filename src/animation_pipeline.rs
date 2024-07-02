use std::sync::Arc;


pub struct AnimationPipeline {
	pipeline: wgpu::ComputePipeline,
}

pub struct AnimationPipelineArgs {
	pub instance: Arc<wgpu::Instance>,
	pub queue: Arc<wgpu::Queue>,
	pub device: Arc<wgpu::Device>,
	pub adapter: Arc<wgpu::Adapter>,
	pub animation_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	pub node_bind_group_layout: Arc<wgpu::BindGroupLayout>,
	pub change_node_bind_group_layout: Arc<wgpu::BindGroupLayout>,
}

#[derive(Debug)]
pub struct AnimateArgs<'a> {
	pub encoder: &'a mut wgpu::CommandEncoder,
	pub animation_bind_group: &'a wgpu::BindGroup,
	pub node_bind_group: &'a wgpu::BindGroup,
	pub change_node_bind_group: &'a wgpu::BindGroup,
}

impl AnimationPipeline {
	pub fn new(args: AnimationPipelineArgs) -> Self {
		println!("creating animation pipeline");
		let shader = args.device.create_shader_module(wgpu::ShaderModuleDescriptor {
			label: Some("Animation Shader"),
			source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/animation.wgsl").into()),
		});

		let pipeline_layout = args.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Animation Pipeline Layout"),
			bind_group_layouts: &[&args.animation_bind_group_layout, &args.node_bind_group_layout, &args.change_node_bind_group_layout],
			push_constant_ranges: &[],
		});

		let pipeline = args.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
			label: Some("Animation Pipeline"),
			layout: Some(&pipeline_layout),
			module: &shader,
			entry_point: "main",
			compilation_options: Default::default(),
		});

		println!("created animation pipeline");

		Self {
			pipeline,
		}
	}

	pub fn animate(&self, args: AnimateArgs) {
		// println!("animating {:?}", args);
		let mut cpass = args.encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { 
			label: None,
			timestamp_writes: None 
		});
		cpass.set_pipeline(&self.pipeline);
		cpass.set_bind_group(0, &args.animation_bind_group, &[]);
		cpass.set_bind_group(1, &args.node_bind_group, &[]);
		cpass.set_bind_group(2, &args.change_node_bind_group, &[]);
		cpass.dispatch_workgroups(64, 1, 1);
	}
}
pub trait BufferRecipe {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer;
	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout;
	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup;
}

pub trait VertexBufferRecipe {
	fn desc() -> wgpu::VertexBufferLayout<'static>;
}


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Positions {
	pub position: [f32; 3],
}

impl Positions {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Positions>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					format: wgpu::VertexFormat::Float32x3,
					shader_location: 0,
				}
			]
		}
	}

	pub fn create_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Position Buffer"),
			size: (std::mem::size_of::<Positions>() * size) as u64,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Normal {
    normal: [f32; 3],
}

impl Normal {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Normal>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					format: wgpu::VertexFormat::Float32x3,
					shader_location: 1,
				}
			]
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexCoords {
    tex_coords: [f32; 2],
}

impl TexCoords {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<TexCoords>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					format: wgpu::VertexFormat::Float32x2,
					shader_location: 2,
				}
			]
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    padding: [f32; 2],  // Align to 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawCamera {
    pub proj: [[f32; 4]; 4],
	pub node_inx: i32,
	pub _padding: [u32; 3],
}

impl BufferRecipe for RawCamera {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Camera Buffer"),
			size: std::mem::size_of::<RawCamera>() as u64,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Camera Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("Camera Bind Group"),
		})
	}

}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RawInstance {
    pub node_index: i32
}

impl RawInstance {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<RawInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Sint32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLight {
	position: [f32; 3],
	intensity: f32,
	color: [f32; 3],
	range: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DirectionalLight {
    direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    padding: f32,  // Align to 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpotLight {
    position: [f32; 3],
    direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
    padding: [f32; 2],  // Align to 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct AmbientLight {
    color: [f32; 3],
    intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct RectangularAreaLight {
    position: [f32; 3],
    direction: [f32; 3],
    width: f32,
    height: f32,
    color: [f32; 3],
    intensity: f32,
}

const MAX_POINT_LIGHTS: usize = 10;
const MAX_SPOT_LIGHTS: usize = 5;
const MAX_DIRECTIONAL_LIGHTS: usize = 2;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    ambient_light: AmbientLight,
    num_point_lights: u32,
    num_spot_lights: u32,
    num_directional_lights: u32,
    padding: u32, // Alignment
    point_lights: [PointLight; MAX_POINT_LIGHTS],
    spot_lights: [SpotLight; MAX_SPOT_LIGHTS],
    directional_lights: [DirectionalLight; MAX_DIRECTIONAL_LIGHTS],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RawNode {
	pub model: [[f32; 4]; 4],
	pub parent_index: i32,
	pub _padding: [u32; 3],
}

impl BufferRecipe for RawNode {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Node Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Node Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE | wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: false },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("Node Bind Group"),
		})
	}
}

// impl RawNode {
// 	pub fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
// 		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
// 			label: Some("Node Bind Group Layout"),
// 			entries: &[
// 				wgpu::BindGroupLayoutEntry {
// 					binding: 0,
// 					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
// 					ty: wgpu::BindingType::Buffer {
// 						ty: wgpu::BufferBindingType::Storage { read_only: false },
// 						has_dynamic_offset: false,
// 						min_binding_size: None,
// 					},
// 					count: None,
// 				},
// 			],
// 		})
// 	}

// 	pub fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
// 		device.create_buffer(&wgpu::BufferDescriptor {
// 			label: Some("Node Buffer"),
// 			size: 1024,
// 			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
// 			mapped_at_creation: false,
// 		})
// 	}

// 	pub fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
// 		device.create_bind_group(&wgpu::BindGroupDescriptor {
// 			layout,
// 			entries: &[
// 				wgpu::BindGroupEntry {
// 					binding: 0,
// 					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
// 						buffer,
// 						offset: 0,
// 						size: None,
// 					}),
// 				},
// 			],
// 			label: Some("Node Bind Group"),
// 		})
// 	}
// }

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NodeTransformation {
	pub model: [[f32; 4]; 4],
	pub waiting: u32,
	pub _padding: [u32; 3],
}

impl BufferRecipe for NodeTransformation {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Node Transformation Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Node Transformation Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: false },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("Node Transformation Bind Group"),
		})
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Keyframe {
	pub value: [[f32; 4]; 4],
	pub is_running: u32,
	pub node_inx: u32
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawPointLight {
	pub color: [f32; 3],
	pub intensity: f32,
	// pub _padding: f32,
	pub node_inx: i32,
	// pub _padding2: [f32; 3],
}

impl BufferRecipe for RawPointLight {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Point Light Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Point Light Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("Point Light Bind Group"),
		})
	}
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawAnimation {
	pub first_keyframe: u32,
	pub last_keyframe: u32,
}

impl BufferRecipe for RawAnimation {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Animation Buffer"),
			size: std::mem::size_of::<RawAnimation>() as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Animation Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::COMPUTE,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: true },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("Animation Bind Group"),
		})
	}

}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawKeyFrame {
	pub value: [[f32; 4]; 4],
}

impl BufferRecipe for RawKeyFrame {
	fn create_buffer(device: &wgpu::Device) -> wgpu::Buffer {
		device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("KeyFrame Buffer"),
			size: std::mem::size_of::<RawKeyFrame>() as u64,
			usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		})
	}

	fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
		device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("KeyFrame Bind Group Layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Storage { read_only: false },
						has_dynamic_offset: false,
						min_binding_size: None,
					},
					count: None,
				},
			],
		})
	}

	fn create_bind_group(device: &wgpu::Device, buffer: &wgpu::Buffer, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
						buffer,
						offset: 0,
						size: None,
					}),
				},
			],
			label: Some("KeyFrame Bind Group"),
		})
	}
}
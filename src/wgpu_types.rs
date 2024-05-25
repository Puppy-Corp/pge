use crate::buffer::Buffer;
use crate::math::Mat4;


#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Position {
	position: [f32; 3],
}

impl Position {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Position>() as wgpu::BufferAddress,
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
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    node_index: i32
}

impl Instance {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint32,
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

pub struct NodeTransform {
	pub model: Mat4,
	parent_index: i32
}

impl NodeTransform {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
		use std::mem;
		wgpu::VertexBufferLayout {
			array_stride: mem::size_of::<NodeTransform>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Instance,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 3,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: 16,
					shader_location: 4,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: 32,
					shader_location: 5,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: 48,
					shader_location: 6,
					format: wgpu::VertexFormat::Float32x4,
				},
				wgpu::VertexAttribute {
					offset: 64,
					shader_location: 7,
					format: wgpu::VertexFormat::Sint32,
				},
			],
		}
	}
}

pub struct WgpuState {
	node_bind_group_layout: wgpu::BindGroupLayout,
	camera_bind_group_layout: wgpu::BindGroupLayout,
	position_buffer: Buffer,
}
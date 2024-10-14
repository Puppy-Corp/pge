
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MaterialUniform {
    base_color: [f32; 4],
    metallic: f32,
    roughness: f32,
    padding: [f32; 2], // Align to 16 bytes
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct DirectionalLight {
    direction: [f32; 3],
    intensity: f32,
    color: [f32; 3],
    padding: f32, // Align to 16 bytes
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
    padding: [f32; 2], // Align to 16 bytes
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
    point_lights: [RawPointLight; MAX_POINT_LIGHTS],
    spot_lights: [SpotLight; MAX_SPOT_LIGHTS],
    directional_lights: [DirectionalLight; MAX_DIRECTIONAL_LIGHTS],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Keyframe {
    pub value: [[f32; 4]; 4],
    pub is_running: u32,
    pub node_inx: u32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawPointLight {
    pub color: [f32; 3], // 12 bytes
    _padding1: f32,      // 4 bytes to align `intensity` to 16 bytes
    pub intensity: f32,  // 4 bytes
    _padding2: [f32; 3], // 12 bytes to align `position` to 16 bytes
    pub position: [f32; 3], // 12 bytes
    _padding3: f32,      // 4 bytes to align the total size to 16 bytes
}

impl RawPointLight {
	pub fn new(color: [f32; 3], intensity: f32, position: [f32; 3]) -> Self {
		Self {
			color,
			_padding1: 0.0,
			intensity,
			_padding2: [0.0; 3],
			position,
			_padding3: 0.0,
		}
	}
}
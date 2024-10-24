use image::codecs::qoi;

use crate::ArenaId;
use crate::Camera;
use crate::Texture;

pub struct WriteCommand {
	pub start: usize,
	pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
	ImageLoaded {
		texture_id: ArenaId<Texture>,
		width: u32,
		height: u32,
		data: Vec<u8>,
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct CamView {
	pub camera_id: ArenaId<Camera>,
	pub x: f32,
	pub y: f32,
	pub w: f32,
	pub h: f32
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RawCamera {
    pub model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Debug)]
pub struct RawInstance {
    pub model: [[f32; 4]; 4],
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
		Self { color, intensity, position, _padding1: 0.0, _padding2: [0.0; 3], _padding3: 0.0 }
	}
}
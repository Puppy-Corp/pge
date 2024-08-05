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
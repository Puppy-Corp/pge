use thunderdome::Index;

pub struct WriteCommand {
	pub start: usize,
	pub data: Vec<u8>
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
	ImageLoaded {
		texture_id: Index,
		width: u32,
		height: u32,
		data: Vec<u8>,
	}
}
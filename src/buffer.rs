use std::sync::Arc;

pub struct Pointer {
	
}

pub struct Slot {
	pub offset: usize,
	pub size: usize
}

pub struct Buffer {
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	slots: Vec<Slot>
}

impl Buffer {
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		Self {
			device,
			queue,
			slots: Vec::new()
		}
	}

	pub fn store(&self, data: &[u8]) -> Slot {
		Slot {
			offset: 0,
			size: 0
		}
	}

	pub fn write(&self, slot: usize, data: &[u8]) {
		
	}
}
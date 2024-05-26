use std::sync::Arc;

pub struct Pointer {
	
}

#[derive(Debug, Clone, Copy)]
pub struct Slot {
	pub offset: usize,
	pub size: usize
}

pub struct Buffer {
	device: Arc<wgpu::Device>,
	queue: Arc<wgpu::Queue>,
	slots: Vec<Slot>,
	buffer: wgpu::Buffer
}

impl Buffer {
	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
		let buff = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Buffer"),
			size: 1024,
			usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::INDEX,
			mapped_at_creation: false
		});

		Self {
			device,
			queue,
			slots: Vec::new(),
			buffer: buff
		}
	}

	pub fn store(&self, data: &[u8]) -> Slot {
		let slot = Slot {
			offset: 0,
			size: data.len()
		};

		self.queue.write_buffer(&self.buffer, 0, data);

		slot
	}

	pub fn write(&self, slot: Slot, data: &[u8]) {
		self.queue.write_buffer(&self.buffer, slot.offset as u64, data);
	}

	pub fn bind_group(&self) {

	}

	pub fn buffer(&self) -> &wgpu::Buffer {
		&self.buffer
	}
}
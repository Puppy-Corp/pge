
pub struct DrawCmd {
	pub mesh_id: usize,
	pub node_id: usize
}

pub struct DrawQueue {
	pub queue: Vec<DrawCmd>
}

impl DrawQueue {
	pub fn new() -> Self {
		Self {
			queue: Vec::new()
		}
	}

	pub fn draw(&mut self, mesh_id: usize, node_id: usize) {
		println!("Drawing mesh: {}", mesh_id);
		self.queue.push(DrawCmd {
			mesh_id,
			node_id
		});
	}
}
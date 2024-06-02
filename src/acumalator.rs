use std::collections::HashMap;

pub struct TransformationAcumalator {
	items: HashMap<usize, glam::Mat4>
}

impl TransformationAcumalator {
	pub fn new() -> Self {
		Self {
			items: HashMap::new()
		}
	}

	pub fn accumulate(&mut self, id: usize, mat: glam::Mat4) {
		match self.items.get_mut(&id) {
			Some(item) => {
				*item = mat * *item;
			},
			None => {
				self.items.insert(id, mat);
			}
		}
	}

	pub fn get_items(&self) -> &HashMap<usize, glam::Mat4> {
		&self.items
	}

	pub fn clear(&mut self) {
		self.items.clear();
	}
}
use crate::{window::Window, world_3d::World3D};

pub struct Engine {
	
}

impl Engine {
	pub fn new() -> Self {
		Self {
			
		}
	}

	pub fn create_window(&self) -> Window {
		Window::new()
	}

	pub fn create_world(&self) -> World3D {
		World3D::new()
	}
}
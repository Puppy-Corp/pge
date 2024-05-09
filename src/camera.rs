pub struct Camera {

}

impl Clone for Camera {
	fn clone(&self) -> Camera {
		Camera {}
	}
}

impl Camera {
	pub fn new() -> Camera {
		Camera {}
	}

	pub fn set_loc(&self, x: f32, y: f32, z: f32) -> Camera {
		log::info!("Camera location set to x: {}, y: {}, z: {}", x, y, z);
		Camera { }
	}

	pub fn set_looking_at(&self, x: f32, y: f32, z: f32) -> Camera {
		log::info!("Camera looking at x: {}, y: {}, z: {}", x, y, z);
		Camera { }
	}
}
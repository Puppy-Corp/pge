pub struct Camera {

}

pub fn camera() -> Camera {
	Camera::new()
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

	pub fn rotate(&self, x: f32, y: f32, z: f32) -> Camera {
		log::info!("Camera rotated by x: {}, y: {}, z: {}", x, y, z);
		Camera { }
	}
}

pub struct CameraView {
	pub camera: Camera,
}

impl CameraView {
	pub fn new(camera: Camera) -> CameraView {
		CameraView {
			camera
		}
	}
	
	pub fn set_size(&self, width: usize, height: usize) {
		log::info!("Camera view size set to width: {}, height: {}", width, height);
	}

	pub fn set_loc(&self, x: usize, y: usize) {
		log::info!("Camera view location set to x: {}, y: {}", x, y);
	}	
}

pub fn camera_view(camera: Camera) -> CameraView {
	CameraView::new(camera)
}
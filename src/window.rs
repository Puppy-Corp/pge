use crate::camera::CameraView;
use crate::gui::GUI;
use crate::input::InputHandler;

pub struct Window {

}

impl Window {
	pub fn new() -> Window {
		Window {}
	}

	pub fn input_handler(&self) -> InputHandler {
		InputHandler::new()
	}

	pub fn set_gui(&self, gui: GUI) {
		
	}

	pub fn add_view(&self, view: GUI) {
		
	}

	pub fn add_camera_view(&self, view: CameraView) {
		
	}

	pub fn view(&self, view: CameraView) -> Window {
		
	}
}

pub fn window() -> Window {
	Window::new()
}
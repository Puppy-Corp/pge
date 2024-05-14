use tokio::sync::mpsc;
use winit::event_loop::EventLoopProxy;

use crate::camera::CameraView;
use crate::gui::GUI;
use crate::input::InputHandler;
use crate::types::*;

pub struct Window {

}

impl Window {
	pub fn new(tx: mpsc::UnboundedSender<UserEvent>) -> Window {
		Window {}
	}

	pub fn input_handler(&self) -> InputHandler {
		InputHandler::new()
	}

	pub fn set_gui(&self, gui: GUI) {
		
	}

	pub fn add_view(&self, view: GUI) {
		
	}

	pub fn camera_view(&self, view: CameraView) {
		
	}

	pub fn view(&self, view: CameraView) -> Window {
		Window {}	
	}
}

// pub fn window() -> Window {
// 	Window::new()
// }
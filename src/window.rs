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
}
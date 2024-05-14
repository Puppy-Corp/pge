use crate::types::InputEvent;

pub struct InputHandler {

}

impl InputHandler {
	pub fn new() -> InputHandler {
		InputHandler {}
	}

	pub async fn poll(&self) -> InputEvent {
		InputEvent::KeyboardEvent
	}
}
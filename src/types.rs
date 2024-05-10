
#[derive(Debug)]
pub enum InputEvent {
	MouseEvent,
	KeyboardEvent,
}

#[derive(Debug)]
pub enum Event {
	InputEvent(InputEvent),
}

pub enum PhysicsType {
	Static,
	Dynamic,
}
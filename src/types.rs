use crate::wgpu::renderer::Renderer;


#[derive(Debug)]
pub enum MouseEvent {
	Moved { dx: f32, dy: f32 }
}

#[derive(Debug)]
pub enum InputEvent {
	MouseEvent(MouseEvent),
	KeyboardEvent,
}

pub enum PhycicsEvent {
	Collision { id: usize }
}

#[derive(Debug)]
pub enum Event {
	InputEvent(InputEvent),
	Redraw,
}

pub enum PhysicsType {
	Static,
	Dynamic,
}

pub struct Mesh {

}

impl Clone for Mesh {
	fn clone(&self) -> Mesh {
		Mesh {}
	}
}

impl Mesh {
	pub fn new() -> Self {
		Self {}
	}
}

pub struct PhycicsProps {

}

pub struct Rotation {
	
}

impl Rotation {
	pub fn new() -> Self {
		Self {}
	}

	pub fn rotate(&mut self, x: f32, y: f32) {
		println!("Rotating: x: {}, y: {}", x, y);
	}
}

#[derive(Debug)]
pub enum UserEvent {
	CreateWindow
}

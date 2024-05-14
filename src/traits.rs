use crate::types::InputEvent;
use crate::Renderer;

pub trait Entity<T> {
	fn render(&mut self, renderer: &Renderer) { }
	fn handle_input(&mut self, event: InputEvent, time: u64) {}
	fn publish_event(&mut self, event: T) {}
}

pub trait PuppyApplication {
	fn on_start(&mut self) {}
	fn on_stop(&mut self) {}
	fn on_input(&mut self, event: InputEvent, time: u64) {}
}
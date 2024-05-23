use crate::system::Context;
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

#[async_trait::async_trait]
pub trait Actor: Send {
	type Message: Send;

	async fn init(&mut self, system: &mut Context) {}
	fn handle(&mut self, message: Self::Message, system: &mut Context) {}
}

pub trait IntoBoxedActor<T> {
    fn into_boxed_actor(self) -> Box<dyn Actor<Message = T>>;
}

impl<T, A> IntoBoxedActor<T> for A
where
    A: Actor<Message = T> + 'static,
{
    fn into_boxed_actor(self) -> Box<dyn Actor<Message = T>> {
        Box::new(self)
    }
}
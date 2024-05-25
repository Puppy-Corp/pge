// use std::collections::HashMap;
// use std::time::Duration;

// use tokio::sync::mpsc;
// use tokio::task::LocalSet;
// use tokio::time::sleep;
// use winit::application::ApplicationHandler;
// use winit::event;
// use winit::event::WindowEvent;
// use winit::event_loop::EventLoop;
// use winit::event_loop::EventLoopProxy;
// use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
// use winit::window::Window;
// use winit::window::WindowId;

// use crate::app::App;
// use crate::traits::PuppyApplication;
// use crate::types::*;

// pub struct Engine {
// 	windows: HashMap<WindowId, Window>,
// 	tx: mpsc::UnboundedSender<UserEvent>,
// }

// impl Engine {
// 	pub fn new() -> anyhow::Result<Self> {
// 		let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

// 		tokio::task::spawn_blocking(move || {
// 			let mut event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
// 			let proxy = event_loop.create_proxy();
// 			tokio::spawn(async move {
// 				while let Some(event) = rx.recv().await {
// 					proxy.send_event(event).unwrap();
// 				}
// 			});
// 			let mut app = App::new();
// 			event_loop.run_app_on_demand(&mut app);
// 			event_loop.run_app(&mut app);
// 		});

// 		Ok(Self {
// 			windows: HashMap::new(),
// 			tx,
// 		})
// 	}

// 	// pub fn add_entity<T>(&mut self, entity: impl Entity<T>) {
		
// 	// }

// 	pub fn create_window(&self) -> crate::window::Window {
// 		self.tx.send(UserEvent::CreateWindow).unwrap();
// 		crate::window::Window::new(self.tx.clone())
// 	}

// 	pub fn on_init(&self, f: impl FnOnce()) {
// 		f();
// 	}

// 	// pub fn create_world(&self) -> World3D {
// 	// 	World3D::new()
// 	// }

// 	// pub fn render(&self, root: Window) {
		
// 	// }

// 	pub async fn next_event(&mut self) -> Event {
// 		sleep(Duration::from_secs(10)).await;
// 		Event::Redraw
// 	}

// 	pub fn run(mut self, app: impl PuppyApplication) {
// 		app.on_start();
// 		let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
// 		let mut app = App::new();
// 		event_loop.run_app(&mut app);



// 		// loop {
// 		// 	let event = self.next_event().await;
// 		// 	match event {
// 		// 		Event::InputEvent(ev) => {
// 		// 			app.on_input(ev, 10);
// 		// 		}
// 		// 		Event::Redraw => {
					
// 		// 		}
// 		// 	}
// 		// }

// 	}
// }

use std::future::Future;

use tokio::sync::mpsc::UnboundedReceiver;

use crate::system::Context;
use crate::traits::Actor;
use crate::traits::IntoBoxedActor;
use crate::types::*;
use crate::Window;

// pub struct Engine<F> {
// 	f: F
// }

// impl<F, Fut> Engine<F>
// 	where
// 		F: FnOnce(Recevier, EngineContext) -> Fut,
// 		Fut: Future<Output = ()> + Send + 'static,
// {
// 	pub fn new(init: F) -> Self {
// 		Self {
// 			f: init
// 		}
// 	}

// 	pub async fn run(mut self) {

// 	}
// }

pub enum Command {

}

pub struct Commader {

}

impl Commader {
	pub fn new() -> Self {
		Self {}
	}

	pub async fn on_keydown(&self, cmd: Command) -> UnboundedReceiver<()> {
		let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
		tx.send(()).unwrap();
		rx
	}
}

pub struct EngineHandle {

}

impl EngineHandle {
	pub fn new() -> Self {
		Self {}
	}

	pub fn save_scene(&self, scene: &Scene) {

	}

	pub fn save_window(&self, window: &Window) {
		
	}
}

pub struct Engine<F> {
	f: F
}

impl<F, Fut> Engine<F>
where
	F: FnOnce(EngineHandle) -> Fut,
	Fut: Future<Output = ()> + Send + 'static,
{
	pub fn new(f: F) -> Self {
		Self {
			f
		}
	}

	// pub fn add_actor<A>(&mut self, actor: A)
	// where
	// 	A: IntoBoxedActor<T> 
	// {
	// 	let mut actor = actor.into_boxed_actor();
	// 	let mut ctx = Context::new();
	// 	let init_future = actor.init(&mut ctx);
	// 	self.actors.push(actor);
	// }

	pub async fn run(mut self) {

	}
}
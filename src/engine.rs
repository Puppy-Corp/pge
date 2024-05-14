use std::collections::HashMap;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::task::LocalSet;
use tokio::time::sleep;
use winit::application::ApplicationHandler;
use winit::event;
use winit::event::WindowEvent;
use winit::event_loop::EventLoop;
use winit::event_loop::EventLoopProxy;
use winit::platform::run_on_demand::EventLoopExtRunOnDemand;
use winit::window::Window;
use winit::window::WindowId;

use crate::app::App;
use crate::traits::PuppyApplication;
use crate::types::*;

pub struct Engine {
	windows: HashMap<WindowId, Window>,
	tx: mpsc::UnboundedSender<UserEvent>,
}

impl Engine {
	pub fn new() -> anyhow::Result<Self> {
		let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

		tokio::task::spawn_blocking(move || {
			let mut event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
			let proxy = event_loop.create_proxy();
			tokio::spawn(async move {
				while let Some(event) = rx.recv().await {
					proxy.send_event(event).unwrap();
				}
			});
			let mut app = App::new();
			event_loop.run_app_on_demand(&mut app);
			event_loop.run_app(&mut app);
		});

		Ok(Self {
			windows: HashMap::new(),
			tx,
		})
	}

	pub fn add_entity<T>(&mut self, entity: impl Entity<T>) {
		
	}

	pub fn create_window(&self) -> crate::window::Window {
		self.tx.send(UserEvent::CreateWindow).unwrap();
		crate::window::Window::new(self.tx.clone())
	}

	pub fn on_init(&self, f: impl FnOnce()) {
		f();
	}

	// pub fn create_world(&self) -> World3D {
	// 	World3D::new()
	// }

	// pub fn render(&self, root: Window) {
		
	// }

	pub async fn next_event(&mut self) -> Event {
		sleep(Duration::from_secs(10)).await;
		Event::Redraw
	}

	pub fn run(mut self, app: impl PuppyApplication) {
		app.on_start();
		let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
		let mut app = App::new();
		event_loop.run_app(&mut app);



		loop {
			let event = self.next_event().await;
			match event {
				Event::InputEvent(ev) => {
					app.on_input(ev, 10);
				}
				Event::Redraw => {
					
				}
			}
		}

	}
}

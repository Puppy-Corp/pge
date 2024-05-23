// use std::collections::HashMap;

// use winit::application::ApplicationHandler;
// use winit::event::WindowEvent;
// use winit::window::Window;
// use winit::window::WindowId;

// use crate::traits::PuppyApplication;
// use crate::types::UserEvent;


// pub struct App {
// 	handler: Box<dyn PuppyApplication>
// }

// impl App {
// 	pub fn new(handler: PuppyApplication) -> Self {
// 		Self {
// 			handler: Box::new(handler)
// 		}
// 	}
// }

// impl ApplicationHandler<UserEvent> for App {
// 	fn user_event(&mut self, event_loop: &winit::event_loop::ActiveEventLoop, event: UserEvent) {
// 		match event {
// 			UserEvent::CreateWindow => {
// 				let window = winit::window::Window::default_attributes().with_title("puppy");
// 				let window = event_loop.create_window(window).unwrap();
// 				self.windows.insert(window.id(), window);
// 			}
// 		}
		
// 	}

// 	fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {

// 	}

// 	fn window_event(
// 		&mut self,
// 		event_loop: &winit::event_loop::ActiveEventLoop,
// 		window_id: winit::window::WindowId,
// 		event: winit::event::WindowEvent,
// 	) {
// 		match event {
// 			WindowEvent::CloseRequested => {
// 				// let win = self.windows.get(&window_id).unwrap();
// 				self.windows.remove(&window_id);
// 			},
// 			WindowEvent::MouseInput { device_id, state, button } => {},
// 			WindowEvent::CursorMoved { device_id, position } => {}
// 			_ => {}
// 		}
// 	}
// }
// use puppy_game_engine::camera::camera;
// use puppy_game_engine::camera::camera_view;
// use puppy_game_engine::camera::Camera;
// use puppy_game_engine::camera::CameraView;
// use puppy_game_engine::engine::Engine;
// use puppy_game_engine::gui::GUI;
// use puppy_game_engine::location::Location;
// use puppy_game_engine::shapes::cube;
// use puppy_game_engine::traits::PuppyApplication;
// use puppy_game_engine::types::Entity;
// use puppy_game_engine::types::*;
// use puppy_game_engine::types::Rotation;
// use puppy_game_engine::Renderer;

// static GROUND_ID: usize = 1;

// enum GameEvent {

// }

// struct FpsPlayer {
// 	last_time: u64,
// 	cube: Mesh,
// 	location: Location,
// 	rotation: Rotation
// }

// impl Entity<GameEvent> for FpsPlayer {
// 	fn render(&mut self, renderer: &Renderer) {

// 	}
// }

// impl FpsPlayer {
// 	pub fn new() -> FpsPlayer {
// 		FpsPlayer {
// 			last_time: 0,
// 			cube: cube(1.0),
// 			location: Location::new(),
// 			rotation: Rotation::new()
// 		}
// 	}

// 	pub fn handle_input(&mut self, event: InputEvent, time: u64) {
// 		let delta = time - self.last_time;
// 		self.last_time = time;

// 		match event {
// 			InputEvent::MouseEvent(e) => {
// 				match e {
// 					MouseEvent::Moved { dx, dy } => {
// 						println!("Mouse moved: dx: {}, dy: {}", dx, dy);
// 						self.rotation.rotate(dx, dy);
// 					}
// 				}
// 				println!("Mouse event");
// 			},
// 			InputEvent::KeyboardEvent => {
// 				println!("Keyboard event");
// 			},
// 			_ => {
// 				println!("Event: {:?}", event);
// 			}
// 		}
// 	}
	
// 	pub fn render(&mut self, renderer: &Renderer) {
// 		// Render code here
		
// 	}
// }

// // impl FpsPlayer {
// // 	pub fn new() -> FpsPlayer {
// // 		FpsPlayer {
// // 			last_time: 0,
// // 			cube: cube(1.0),
// // 			location: Location::new(),
// // 			rotation: Rotation::new()
// // 		}
// // 	}

// // 	pub fn handle_input(&mut self, event: InputEvent, time: u64) {
// // 		let delta = time - self.last_time;
// // 		self.last_time = time;

// // 		match event {
// // 			InputEvent::MouseEvent(e) => {
// // 				match e {
// // 					MouseEvent::Moved { dx, dy } => {
// // 						println!("Mouse moved: dx: {}, dy: {}", dx, dy);
// // 						self.rotation.rotate(dx, dy);
// // 					}
// // 				}
// // 				println!("Mouse event");
// // 			},
// // 			InputEvent::KeyboardEvent => {
// // 				println!("Keyboard event");
// // 			},
// // 			_ => {
// // 				println!("Event: {:?}", event);
// // 			}
// // 		}
// // 	}

// // 	pub fn handle_phycics_event(&mut self, event: PhysicsEvent) {
// // 		// Handle physics event here
// // 		match event {
// // 			PhycicsEvent::Collision { id } => {
// // 				println!("Collision event");

// // 			}
// // 		}
// // 	}

// // 	pub fn render(&self) -> Mesh {
// // 		// Render code here

// // 	}

// // 	pub fn phycics() -> PhycicsProps {
// // 		PhycicsProps {}
// // 	}
// // }

// struct FpsExample {
// 	fps: FpsPlayer
// }

// impl FpsExample {
// 	pub fn new() -> FpsExample {
// 		FpsExample {
// 			fps: FpsPlayer::new()
// 		}
// 	}
// }

// impl PuppyApplication for FpsExample {

// }

use std::clone;
use std::future::Future;
use std::time::Duration;

use pge::*;
use tokio::time::sleep;

// struct Matrix4x4 {
// 	data: [f32; 16]
// }

// fn rotateXY(x: f32, y: f32) -> Matrix4x4 {
// 	println!("Rotating: x: {}, y: {}", x, y);
// }

#[derive(Debug, Clone)]
pub enum Message {

}

pub struct FpsPlayer {}

impl FpsPlayer {
	pub fn new() -> FpsPlayer {
		FpsPlayer {}
	}
}

#[async_trait::async_trait]
impl Actor for FpsPlayer {
	type Message = Message;

	async fn init(&mut self, system: &mut Context) {
		// Initialize actor here
	}

	fn handle(&mut self, message: Self::Message, system: &mut Context) {
		// Handle message here
	}
}

pub struct Floor {}

impl Floor {
	pub fn new() -> Floor {
		Floor {}
	}
}

impl Actor for Floor {
	type Message = Message;

	// fn init(&mut self, system: &mut System) {
	// 	// Initialize actor here
	// }

	fn handle(&mut self, message: Self::Message, system: &mut Context) {
		// Handle message here
	}
}

#[tokio::main]
async fn main() {
	let mut engine = Engine::new();
	// engine.add_actor(FpsPlayer::new());
	// engine.add_actor(Floor::new());
	engine.run().await;
}

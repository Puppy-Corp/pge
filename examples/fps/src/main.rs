use puppy_game_engine::camera::camera;
use puppy_game_engine::camera::camera_view;
use puppy_game_engine::camera::Camera;
use puppy_game_engine::camera::CameraView;
use puppy_game_engine::engine::Engine;
use puppy_game_engine::gui::GUI;
use puppy_game_engine::location::Location;
use puppy_game_engine::shapes::cube;
use puppy_game_engine::traits::PuppyApplication;
use puppy_game_engine::types::Entity;
use puppy_game_engine::types::*;
use puppy_game_engine::types::Rotation;
use puppy_game_engine::Renderer;

static GROUND_ID: usize = 1;

enum GameEvent {

}

struct FpsPlayer {
	last_time: u64,
	cube: Mesh,
	location: Location,
	rotation: Rotation
}

impl Entity<GameEvent> for FpsPlayer {
	fn render(&mut self, renderer: &Renderer) {

	}
}

impl FpsPlayer {
	pub fn new() -> FpsPlayer {
		FpsPlayer {
			last_time: 0,
			cube: cube(1.0),
			location: Location::new(),
			rotation: Rotation::new()
		}
	}

	pub fn handle_input(&mut self, event: InputEvent, time: u64) {
		let delta = time - self.last_time;
		self.last_time = time;

		match event {
			InputEvent::MouseEvent(e) => {
				match e {
					MouseEvent::Moved { dx, dy } => {
						println!("Mouse moved: dx: {}, dy: {}", dx, dy);
						self.rotation.rotate(dx, dy);
					}
				}
				println!("Mouse event");
			},
			InputEvent::KeyboardEvent => {
				println!("Keyboard event");
			},
			_ => {
				println!("Event: {:?}", event);
			}
		}
	}
	
	pub fn render(&mut self, renderer: &Renderer) {
		// Render code here
		
	}
}

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

// 	pub fn handle_phycics_event(&mut self, event: PhysicsEvent) {
// 		// Handle physics event here
// 		match event {
// 			PhycicsEvent::Collision { id } => {
// 				println!("Collision event");

// 			}
// 		}
// 	}

// 	pub fn render(&self) -> Mesh {
// 		// Render code here

// 	}

// 	pub fn phycics() -> PhycicsProps {
// 		PhycicsProps {}
// 	}
// }

struct FpsExample {
	fps: FpsPlayer
}

impl FpsExample {
	pub fn new() -> FpsExample {
		FpsExample {
			fps: FpsPlayer::new()
		}
	}
}

impl PuppyApplication for FpsExample {

}

#[tokio::main]
async fn main() {
	let app = Engine::new().run(FpsExample::new());
	
	// let mut engine = .unwrap();
	// // let fps = FpsPlayer::new();
	// // engine.add_entity(fps);

	// engine.on_init(|| {
	// 	let fps = FpsPlayer::new();
	// });

	// engine.run();

	// Engine::new(

	// let window = engine.create_window();

	// loop {
	// 	let event = engine.next_event().await;

	// 	match event {
	// 		Event::InputEvent(ev) => {
	// 			// fps.handle_input(ev, 10);
	// 		}
	// 		Event::Redraw => {
				
	// 		}
	// 	}
	
	// }

	// let world = engine.create_world();
	// let window = engine.create_window();

	// loop {
	// 	let event = engine.next_event();

	// 	match event {
	// 		Event::InputEvent(ev) => {
	// 			fps.handle_input(ev, 10);
	// 		}
	// 		Event::Redraw => {
				
	// 		}
	// 	}



	// 	let camera = camera();
	// 	let camera_view = camera_view(camera);
	// 	let window = window().camera_view(camera_view);

	// 	let root = root().winow(window);

	// 	root3d().

	// 	engine.render();
	// }
}

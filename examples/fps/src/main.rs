use puppy_game_engine::camera::camera_view;
use puppy_game_engine::camera::Camera;
use puppy_game_engine::camera::CameraView;
use puppy_game_engine::engine::Engine;
use puppy_game_engine::entity::Entity;
use puppy_game_engine::gui::GUI;
use puppy_game_engine::shapes::plane;
use puppy_game_engine::shapes::rect;
use puppy_game_engine::types::InputEvent;
use puppy_game_engine::types::PhysicsType;
use puppy_game_engine::window::window;


#[tokio::main]
async fn main() {
    let mut engine = Engine::new();

	loop {
		let event = engine.next_event();

		match event {
			InputEvent::MouseEvent => {
				println!("Mouse event");
			},
			InputEvent::KeyboardEvent => {
				println!("Keyboard event");
			},
			_ => {
				println!("Event: {:?}", event);
			}
		}

		engine.render(window().view(camera_view()));
	}

	// let window = engine.create_window();
	// let world = engine.create_world();

	// let input_handler = window.input_handler();

	// let camera = Camera::new()
	// 	.set_loc(0.0, 10.0, 15.0)
	// 	.set_looking_at(0.0, 0.0, 0.0);

	// let floor = plane(50.0, 50.0)
	// 	.set_translation(0.09, 0.0, 0.0)
	// 	.set_phycics_type(PhysicsType::Dynamic);
	
	// let gui = GUI::new();
	// window.set_gui(gui);

	// world.add_entity(floor.clone());
	// world.add_camera(camera.clone());

	// let camera_view = CameraView::new()
	// 	.set_camera(camera);

	// window.add_camera_view(camera_view);

	// loop {
	// 	let input = input_handler.poll().await;




		
	// 	let collisions =  floor.collides();

	// 	for collision in collisions {
	// 		println!("Collision with {:?}", collision);
	// 	}
	// }
}

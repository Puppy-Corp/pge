use puppy_game_engine::camera::Camera;
use puppy_game_engine::engine::Engine;
use puppy_game_engine::entity::Entity;
use puppy_game_engine::shapes::plane;
use puppy_game_engine::shapes::rect;
use puppy_game_engine::types::PhysicsType;


#[tokio::main]
async fn main() {
    let engine = Engine::new();
	let window = engine.create_window();
	let world = engine.create_world();

	let input_handler = window.input_handler();

	let camera = Camera::new()
		.set_loc(0.0, 10.0, 15.0)
		.set_looking_at(0.0, 0.0, 0.0);

	let floor = plane(50.0, 50.0)
		.set_translation(0.09, 0.0, 0.0)
		.set_phycics_type(PhysicsType::Dynamic);
	

	world.add_entity(floor.clone());
	world.add_camera(camera);

	loop {
		let input = input_handler.poll().await;


		
		let collisions =  floor.collides();

		for collision in collisions {
			println!("Collision with {:?}", collision);
		}
	}
}

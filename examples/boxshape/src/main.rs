use puppy_game_engine::*;

#[tokio::main]
async fn main() {
    Engine::new(|handle| async move {
		let scene = Scene::new();

		let mut root = Node::new();
		let material = Material::new();

		let mut cube_node = Node::new();
		let mut cube_mesh = cube(1.0);
		cube_mesh.set_material(material);
		cube_node.set_mesh(cube_mesh);
		root.add_node(cube_node);

		let mut camera_node = Node::new();
		let camera = Camera::new();
		camera_node.set_camera(camera);
		camera_node.set_translation(0.0, 2.0, 3.0);
		camera_node.looking_at(0.0, 0.0, 0.0);
		root.add_node(camera_node);

		handle.save(&scene);

		let mut window = Window::new();
		window.title = "BIG box".to_string();
		handle.save_window(&window);
	}).run().await;
}
 
use std::time::Duration;

use pge::*;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Engine::new(|handle| async move {
		let mut scene = Scene::new();


		let mut root = Node::new();
		// let material = Material::new();

		// let mut cube_node = Node::new();
		// let mut cube_mesh = cube(1.0);
		// cube_mesh.set_material(material);
		// cube_node.set_mesh(cube_mesh);
		// root.add_node(cube_node);

		let mut camera_node = Node::new();
		let camera = Camera::new();
		let scene_cam = SceneCam::new(&camera);
		camera_node.set_camera(camera);
		camera_node.set_translation(0.0, 2.0, 3.0);
		camera_node.looking_at(0.0, 0.0, 0.0);
		root.add_node(camera_node);

		

		let mut window = Window::new();
		window.title = "BIG box".to_string();
		window.body = view().add(scene_cam).into();
		handle.save_window(&window);

		let cube_mesh = cube(1.0);
		let mut node = Node::new();
		node.set_mesh(cube_mesh);
		root.add_node(node);

		scene.add_node(root);
		handle.save_scene(scene);


		sleep(Duration::from_secs(120)).await;
	}).run().await?;
	Ok(())
}
 
use std::time::Duration;

use pge::*;
use text::FontMesh;
use tokio::time::sleep;
use ttf_parser::GlyphId;
use ttf_parser::OutlineBuilder;

struct TodoItem {
	text: String,
	completed: bool,
}

struct Builder {}

impl OutlineBuilder for Builder {
	fn move_to(&mut self, x: f32, y: f32) {
		println!("Move to: {}, {}", x, y);
	}

	fn line_to(&mut self, x: f32, y: f32) {
		println!("Line to: {}, {}", x, y);
	}

	fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
		println!("Quad to: {}, {}, {}, {}", x1, y1, x, y);
	}

	fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
		println!("Curve to: {}, {}, {}, {}, {}, {}", x1, y1, x2, y2, x, y);
	}

	fn close(&mut self) {
		println!("Close");
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Engine::new(|mut engine| async move {
		println!("begin ??");

		let font = FontMesh::load("./fonts/Roboto-Regular.ttf")?;
		
		let mut window = Window::new();
		window.title = "Todo app".to_string();
		engine.save_window(&window);

		let mut scene = Scene::new();
		let mut root = Node::new();

		let mut anode = Node::new();
		let amesh = font.get_mesh('A').unwrap();
		println!("amsh: {:?}", amesh);
		anode.set_mesh(amesh);
		root.add_node(anode);

		let mut anode = Node::new();
		anode.set_translation(1.0, 0.0, 0.0);
		let amesh = font.get_mesh('B').unwrap();
		println!("amsh: {:?}", amesh);
		anode.set_mesh(amesh);
		root.add_node(anode);

		let mut anode = Node::new();
		anode.set_translation(2.0, 0.0, 0.0);
		let amesh = font.get_mesh('C').unwrap();
		println!("amsh: {:?}", amesh);
		anode.set_mesh(amesh);
		root.add_node(anode);

		let mut camera_node = Node::new();
		let camera_node_id = camera_node.id;
		let camera = Camera::new();
		camera_node.set_camera(camera);
		camera_node.set_translation(0.0, 0.0, -6.0);
		camera_node.looking_at(0.0, 0.0, 0.0);
		root.add_node(camera_node.clone());

		scene.add_node(root);
		engine.save_scene(scene);

		println!("will no sleeppp");
		sleep(Duration::from_secs(120)).await;

		let mut todo_items: Vec<TodoItem> = Vec::new();
		
		let root = vstack()
			.add(text("Todo app").font(font))
			.add(list().add_many(
				todo_items.iter().map(|item| text(&item.text)).collect()
			));

		window.render(root);
		Ok(())
	}).run().await?;

	Ok(())
}

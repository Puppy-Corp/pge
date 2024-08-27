use std::collections::HashSet;
use std::time::Duration;

use args::Command;
use clap::Parser;
use pge::*;
use tokio::time::sleep;
mod args;

struct PgeEditor {
	asset_path: Option<String>,
	windows: Vec<ArenaId<Window>>,
	scenes: HashSet<ArenaId<Scene>>,
	camera_nodes: HashSet<ArenaId<Node>>,
	node_editor: Option<NodeEditor>,
}

impl PgeEditor {
	fn new() -> Self {
		Self {
			asset_path: None,
			windows: Vec::new(),
			scenes: HashSet::new(),
			camera_nodes: HashSet::new(),
			node_editor: None,
		}
	}

	pub fn set_inspect_path(&mut self, path: String) {
		self.asset_path = Some(path);
	}
}

impl pge::App for PgeEditor {
	fn on_create(&mut self, state: &mut State) {
		if let Some(path) = &self.asset_path {
			state.load_3d_model(path);
		}

		self.node_editor = Some(NodeEditor::new(state));
	}

	fn on_process(&mut self, state: &mut State, delta: f32) {
		// for (scene_id,scene) in state.scenes.iter_mut() {
		// 	match self.scenes.contains(&scene_id) {
		// 		true => {

		// 		},
		// 		false => {
		// 			let name = scene.name.clone().unwrap_or_default();
		// 			self.scenes.insert(scene_id);
		// 			log::info!("Scene added: {:?}", scene_id);

		// 			let mut light_node = Node::new();
		// 			light_node.parent = NodeParent::Scene(scene_id);
		// 			light_node.translation = Vec3::new(0.0, 5.0,-5.0);
		// 			let light_node_id = state.nodes.insert(light_node);
		// 			let mut light = PointLight::new();
		// 			light.node_id = Some(light_node_id);
		// 			state.point_lights.insert(light);

		// 			let mut camera_node = Node::new();
		// 			camera_node.translation = Vec3::new(0.0, 2.5, 3.3);
		// 			camera_node.looking_at(0.0, 1.0, 0.0);
		// 			camera_node.parent = NodeParent::Scene(scene_id);
		// 			let camera_node_id = state.nodes.insert(camera_node);
		// 			self.camera_nodes.insert(camera_node_id);

		// 			let mut camera = Camera::new();
		// 			camera.node_id = Some(camera_node_id);
		// 			let camera_id = state.cameras.insert(camera);

		// 			let ui = camera_view(camera_id);
		// 			let ui_id = state.guis.insert(ui);

		// 			let window = Window::new().title(&name).ui(ui_id).lock_cursor(true);
		// 			state.windows.insert(window);
		// 		},
		// 	}
		// }

		if let Some(node_editor) = &mut self.node_editor {
			node_editor.on_process(state);
		}
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Moved { dx, dy } => {
				// let sensitivity = 0.005;
				// let rotation_x = Quat::from_axis_angle(Vec3::Y, -dx * sensitivity);
				// let rotation_y = Quat::from_axis_angle(Vec3::X, -dy * sensitivity);
				// let rotation = rotation_y * rotation_x;

				// for node_id in &self.camera_nodes {
				// 	if let Some(node) = state.nodes.get_mut(node_id) {
				// 		node.rotation = rotation * node.rotation;
				// 	}
				// }
			},
			_ => {}
		}

		if let Some(node_editor) = &mut self.node_editor {
			node_editor.on_mouse_input(event, state);
		}
	}

	fn on_mouse_wheel(&mut self, event: MouseWheelEvent, state: &mut State) {
		log::info!("Mouse wheel: {:?}", event);
		if let Some(node_editor) = &mut self.node_editor {
			node_editor.on_mouse_wheel(event, state);
		}
	}

	fn on_cursor_moved(&mut self, event: CursorMovedEvent, state: &mut State) {
		if let Some(node_editor) = &mut self.node_editor {
			node_editor.on_cursor_moved(event, state);
		}
	}

	fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {
		if let Some(node_editor) = &mut self.node_editor {
			node_editor.on_keyboard_input(key, action, state);
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    pge::init_logging();

	let mut editor = PgeEditor::new();

	let args = args::Args::parse();

	if let Some(command) = args.command {
		match command {
			Command::Inspect { path } => {
				editor.set_inspect_path(path);
			}
		}
	}

	Ok(pge::run(editor).await?)
}

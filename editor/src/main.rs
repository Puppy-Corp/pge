use std::collections::HashSet;
use std::time::Duration;

use args::Command;
use clap::Parser;
use pge::*;
mod args;

struct PgeEditor {
	asset_path: Option<String>,
	windows: Vec<ArenaId<Window>>,
	scenes: HashSet<ArenaId<Scene>>,
	camera_nodes: HashSet<ArenaId<Node>>,
}

impl PgeEditor {
	fn new() -> Self {
		Self {
			asset_path: None,
			windows: Vec::new(),
			scenes: HashSet::new(),
			camera_nodes: HashSet::new(),
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
	}

	fn on_process(&mut self, state: &mut State, delta: f32) {
		for (scene_id,scene) in state.scenes.iter_mut() {
			match self.scenes.contains(&scene_id) {
				true => {

				},
				false => {
					let name = scene.name.clone().unwrap_or_default();
					self.scenes.insert(scene_id);
					log::info!("Scene added: {:?}", scene_id);

					let mut light_node = Node::new();
					light_node.parent = NodeParent::Scene(scene_id);
					light_node.translation = Vec3::new(0.0, 5.0,-5.0);
					let light_node_id = state.nodes.insert(light_node);
					let mut light = PointLight::new();
					light.node_id = Some(light_node_id);
					state.point_lights.insert(light);

					let mut camera_node = Node::new();
					camera_node.translation = Vec3::new(0.0, 2.5, 3.3);
					camera_node.looking_at(0.0, 1.0, 0.0);
					camera_node.parent = NodeParent::Scene(scene_id);
					let camera_node_id = state.nodes.insert(camera_node);
					self.camera_nodes.insert(camera_node_id);

					let mut camera = Camera::new();
					camera.node_id = Some(camera_node_id);
					let camera_id = state.cameras.insert(camera);

					let ui = camera_view(camera_id);
					let ui_id = state.guis.insert(ui);

					let window = Window::new().title(&name).ui(ui_id);
					state.windows.insert(window);
				},
			}
		}
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Moved { dx, dy } => {
				let sensitivity = 0.005;
				let rotation_x = Quat::from_axis_angle(Vec3::Y, -dx * sensitivity);
				let rotation_y = Quat::from_axis_angle(Vec3::X, -dy * sensitivity);
				let rotation = rotation_y * rotation_x;

				for node_id in &self.camera_nodes {
					if let Some(node) = state.nodes.get_mut(node_id) {
						node.rotation = rotation * node.rotation;
					}
				}
			},
			_ => {}
		}
	}
}

fn main() {
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

	pge::run(editor).unwrap();
}

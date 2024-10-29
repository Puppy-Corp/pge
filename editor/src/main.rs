use std::collections::HashSet;
use std::time::Duration;

use args::Command;
use clap::Parser;
use pge::*;
mod args;

struct SceneViewer {
	window_id: ArenaId<Window>,
	scene_id: ArenaId<Scene>,
	camera_node_id: ArenaId<Node>,
}

impl SceneViewer {
	fn new(state: &mut State, scene_id: ArenaId<Scene>) -> Self {
		let scene = state.scenes.get(&scene_id).unwrap();
		let name = scene.name.clone().unwrap_or_default();
		log::info!("Scene added: {:?}", scene_id);
		for (_, node) in state.nodes.iter_mut().filter(|(_, node)| node.parent == NodeParent::Scene(scene_id)) {
			node.scale = Vec3::new(10.0, 10.0, 10.0);
		}

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

		let mut camera = Camera::new();
		camera.node_id = Some(camera_node_id);
		let camera_id = state.cameras.insert(camera);

		let ui = camera_view(camera_id);
		let ui_id = state.guis.insert(ui);

		let window = Window::new().title(&name).ui(ui_id);
		let window_id = state.windows.insert(window);
		Self { window_id, scene_id, camera_node_id }
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Moved { dx, dy } => {
				let sensitivity = 0.005;
				let rotation_x = Quat::from_axis_angle(Vec3::Y, -dx * sensitivity);
				let rotation_y = Quat::from_axis_angle(Vec3::X, -dy * sensitivity);
				let rotation = rotation_y * rotation_x;

				/*for node_id in &self.camera_nodes {
					if let Some(node) = state.nodes.get_mut(node_id) {
						node.rotation = rotation * node.rotation;
					}
				}*/
			},
			_ => {}
		}
	}
}

struct PgeEditor {
	asset_path: Option<String>,
	windows: Vec<ArenaId<Window>>,
	scenes: HashSet<ArenaId<Scene>>,
	scene_viewers: Vec<SceneViewer>,
}

impl PgeEditor {
	fn new() -> Self {
		Self {
			asset_path: None,
			windows: Vec::new(),
			scenes: HashSet::new(),
			scene_viewers: Vec::new(),
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
		let mut new_scene_ids = Vec::new();
		for (scene_id,_) in state.scenes.iter_mut() {
			if self.scenes.contains(&scene_id) {
				continue;
			}
			new_scene_ids.push(scene_id);
			self.scenes.insert(scene_id);
		}
		for scene_id in new_scene_ids {
			let scene_viewer = SceneViewer::new(state, scene_id);
			self.scene_viewers.push(scene_viewer);
		}
	}

	fn on_mouse_input(&mut self, window_id: ArenaId<Window>, event: MouseEvent, state: &mut State) {
		let scene_viewer = match self.scene_viewers.iter_mut().find(|v| v.window_id == window_id) {
			Some(v) => v,
			None => return,
		};
		scene_viewer.on_mouse_input(event, state);
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

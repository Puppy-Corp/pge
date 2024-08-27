use std::collections::HashMap;

use ::gltf::camera;
use winit::event::MouseScrollDelta;

use crate::*;

pub struct NodeEditor {
	// main_window_id: ArenaId<Window>,
	camera_id: ArenaId<Camera>,
	camera_node_id: ArenaId<Node>,
	moving: bool,
	start_x: f32,
	start_y: f32,
	original_x: f32,
	original_y: f32,
	nodes: Vec<ArenaId<Node>>,
	windows: HashMap<ArenaId<Scene>, ArenaId<Window>>
}

impl NodeEditor {
	pub fn new(state: &mut State) -> Self {
		let mut scene = Scene::new();
		scene.background_color = [1.0, 1.0, 1.0, 1.0];
		let scene_id = state.scenes.insert(scene);

		let mut camera_node = Node::new();
		camera_node.translation = Vec3::new(0.0, 0.0, -1.0);
		camera_node.parent = NodeParent::Scene(scene_id);
		let camera_node_id = state.nodes.insert(camera_node);
		
		let mut camera = Camera::new();
		camera.projection = Projection::Orthographic { 
			left: -1.0, 
			right: 1.0, 
			bottom: -1.0, 
			top: 1.0, 
		};
		camera.node_id = Some(camera_node_id);
		let camera_id: ArenaId<Camera> = state.cameras.insert(camera);

		let ui = camera_view(camera_id);

		// let cube1 = cube(0.1);
		// let mesh_id = state.meshes.insert(cube1);

		// let mut node = Node::new();
		// node.mesh = Some(mesh_id);
		// node.parent = NodeParent::Scene(scene_id);
		// let node_id = state.nodes.insert(node);

		// let cube2 = cube(0.1);
		// let mesh_id = state.meshes.insert(cube2);

		// let mut node = Node::new();
		// node.translation = Vec3::new(0.5, 0.0, 0.0);
		// node.mesh = Some(mesh_id);
		// node.parent = NodeParent::Scene(scene_id);
		// let node_id = state.nodes.insert(node);

		// let cube3 = cube(0.1);
		// let mesh_id = state.meshes.insert(cube3);

		// let mut node = Node::new();
		// node.translation = Vec3::new(0.0, 0.3, 0.0);
		// node.mesh = Some(mesh_id);
		// node.parent = NodeParent::Scene(scene_id);
		// let node_id = state.nodes.insert(node);

		// let cube3 = cube(0.1);
		// let mesh_id = state.meshes.insert(cube3);

		// let mut node = Node::new();
		// node.translation = Vec3::new(0.0, -0.3, 0.0);
		// node.mesh = Some(mesh_id);
		// node.parent = NodeParent::Scene(scene_id);
		// let node_id = state.nodes.insert(node);

		// let cube3 = cube(0.1);
		// let mesh_id = state.meshes.insert(cube3);

		// let mut node = Node::new();
		// node.translation = Vec3::new(-0.5, 0.0, 0.0);
		// node.mesh = Some(mesh_id);
		// node.parent = NodeParent::Scene(scene_id);
		// let node_id = state.nodes.insert(node);

		// let mut window = Window::new();
		// window.title = "Node Editor".to_string();
		// window.height = 500;
		// window.width = 500;
		// window.ui = Some(state.guis.insert(ui));
		// let window_id = state.windows.insert(window);
		Self {
			// main_window_id: window_id,
			camera_id,
			camera_node_id,
			moving: false,
			start_x: 0.0,
			start_y: 0.0,
			original_x: 0.0,
			original_y: 0.0,
			nodes: vec![],
			windows: HashMap::new(),
		}
	}

	pub fn on_process(&mut self, state: &mut State) {
		for (scene_id, scene) in state.scenes.iter_mut() {
			if scene.name == "scene_editor" {
				continue;
			}	

			if self.windows.contains_key(&scene_id) {
				continue;
			}

			let window = Window {
				title: scene.name.clone(),
				width: 400,
				height: 400,
				..Default::default()
			};
			let window_id = state.windows.insert(window);
			self.windows.insert(scene_id, window_id);
		}
	}

	pub fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {
		match action {
			KeyAction::Pressed => {
				println!("Key pressed: {:?}", key);
				match key {
					KeyboardKey::H => {
						let camera_node = state.nodes.get_mut(&self.camera_node_id).unwrap();
						camera_node.translation = Vec3::new(0.0, 0.0, -1.0);
					}
					_ => {}
				}
			},
			KeyAction::Released => {},
		}
	}

	pub fn on_cursor_moved(&mut self, event: CursorMovedEvent, state: &mut State) {
		if self.moving {
			if self.start_x == 0.0 {
				self.start_x = event.dx;
			}
			if self.start_y == 0.0 {
				self.start_y = event.dy;
			}

			let dx = event.dx - self.start_x;
			let dy = event.dy - self.start_y;

			let camera_node = state.nodes.get_mut(&self.camera_node_id).unwrap();
			camera_node.translation.x = self.original_x - dx * 0.001;
			camera_node.translation.y = self.original_y + dy * 0.001;
		}
	}

	pub fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Pressed { button } => {
				match button {
					MouseButton::Left => {},
					MouseButton::Right => {},
					MouseButton::Middle => {
						self.moving = true;
						self.start_x = 0.0;
						self.start_y = 0.0;
						let camera_node = state.nodes.get(&self.camera_node_id).unwrap();
						self.original_x = camera_node.translation.x;
						self.original_y = camera_node.translation.y;
					},
				}
			},
			MouseEvent::Released { button } => {
				match button {
					MouseButton::Left => {},
					MouseButton::Right => {},
					MouseButton::Middle => {
						self.moving = false;
					},
				}
			},
			_ => {}
		}
	}

	pub fn on_mouse_wheel(&mut self, event: MouseWheelEvent, state: &mut State) {
		match event.delta {
			MouseScrollDelta::LineDelta(x, y) => {
				let camera = state.cameras.get_mut(&self.camera_id).unwrap();
				camera.zoom(-y * 0.01);
			},
			MouseScrollDelta::PixelDelta(_) => todo!(),
		}
	}
}
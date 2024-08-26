use crate::*;

pub struct NodeEditor {
	main_window_id: ArenaId<Window>,
}

impl NodeEditor {
	pub fn new(state: &mut State) -> Self {
		let scene = Scene::new();
		let scene_id = state.scenes.insert(scene);

		let mut camera_node = Node::new();
		camera_node.translation = Vec3::new(0.0, 0.0, -1.0);
		camera_node.parent = NodeParent::Scene(scene_id);
		let camera_node_id = state.nodes.insert(camera_node);
		
		let mut camera = Camera::new();
		camera.node_id = Some(camera_node_id);
		let camera_id: ArenaId<Camera> = state.cameras.insert(camera);

		let ui = camera_view(camera_id);

		let cube1 = cube(0.1);
		let mesh_id = state.meshes.insert(cube1);

		let mut node = Node::new();
		node.mesh = Some(mesh_id);
		node.parent = NodeParent::Scene(scene_id);
		let node_id = state.nodes.insert(node);

		let cube2 = cube(0.1);
		let mesh_id = state.meshes.insert(cube2);

		let mut node = Node::new();
		node.translation = Vec3::new(0.5, 0.0, 0.0);
		node.mesh = Some(mesh_id);
		node.parent = NodeParent::Scene(scene_id);
		let node_id = state.nodes.insert(node);

		let cube3 = cube(0.1);
		let mesh_id = state.meshes.insert(cube3);

		let mut node = Node::new();
		node.translation = Vec3::new(0.0, 0.3, 0.0);
		node.mesh = Some(mesh_id);
		node.parent = NodeParent::Scene(scene_id);
		let node_id = state.nodes.insert(node);

		let cube3 = cube(0.1);
		let mesh_id = state.meshes.insert(cube3);

		let mut node = Node::new();
		node.translation = Vec3::new(0.0, -0.3, 0.0);
		node.mesh = Some(mesh_id);
		node.parent = NodeParent::Scene(scene_id);
		let node_id = state.nodes.insert(node);

		let cube3 = cube(0.1);
		let mesh_id = state.meshes.insert(cube3);

		let mut node = Node::new();
		node.translation = Vec3::new(-0.5, 0.0, 0.0);
		node.mesh = Some(mesh_id);
		node.parent = NodeParent::Scene(scene_id);
		let node_id = state.nodes.insert(node);

		let mut window = Window::new();
		window.title = "Node Editor".to_string();
		window.height = 500;
		window.width = 500;
		window.ui = Some(state.guis.insert(ui));
		let window_id = state.windows.insert(window);
		Self {
			main_window_id: window_id,
		}
	}

	pub fn on_process(&mut self, state: &mut State) {
		
	}

	pub fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {

	}

	pub fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {

	}
}
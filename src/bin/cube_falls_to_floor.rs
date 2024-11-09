use pge::*;


struct TestApp {

}

impl App for TestApp {
	fn on_create(&mut self, state: &mut State) {
		let scene = Scene::new();
		let scene_id = state.scenes.insert(scene);
		let cube_mesh = state.meshes.insert(cube(1.0));
		let floor_mesh = state.meshes.insert(plane(100.0, 100.0));

		let mut light_node = Node::new();
		light_node.name = Some("Light".to_string());
		light_node.set_translation(10.0, 10.0, 0.0);
		light_node.parent = NodeParent::Scene(scene_id);
		let light_node_id = state.nodes.insert(light_node);
		let mut light = PointLight::new();
		light.node_id = Some(light_node_id);
		state.point_lights.insert(light);

		let mut player = Node::new();
		player.name = Some("Player".to_string());
		player.set_translation(0.0, 3.0, -10.0);
		player.parent = NodeParent::Scene(scene_id);
		let player_id = state.nodes.insert(player);

		let mut cube_node = Node::new();
		cube_node.name = Some("Cube".to_string());
		cube_node.set_translation(0.0, 10.0, 0.0);
		cube_node.mesh = Some(cube_mesh);
		cube_node.physics.typ = PhycisObjectType::Dynamic;
		cube_node.physics.mass = 10.0;
		cube_node.collision_shape = Some(CollisionShape::Box { size: Vec3::new(1.0, 1.0, 1.0) });
		cube_node.parent = NodeParent::Scene(scene_id);
		state.nodes.insert(cube_node);

		let mut cube2_node = Node::new();
		cube2_node.name = Some("Cube2".to_string());
		cube2_node.set_translation(3.0, 10.0, 0.0);
		cube2_node.mesh = Some(cube_mesh);
		cube2_node.physics.typ = PhycisObjectType::Dynamic;
		cube2_node.physics.mass = 10.0;
		cube2_node.collision_shape = Some(CollisionShape::Box { size: Vec3::new(1.0, 1.0, 1.0) });
		cube2_node.rotation = Quat::from_euler(EulerRot::XYZ, 0.1, 0.0, 0.0);
		cube2_node.parent = NodeParent::Scene(scene_id);
		state.nodes.insert(cube2_node);

		let mut floor_node = Node::new();
		floor_node.name = Some("Floor".to_string());
		floor_node.set_translation(0.0, 1.0, 0.0);
		floor_node.mesh = Some(floor_mesh);
		floor_node.parent = NodeParent::Scene(scene_id);
		floor_node.physics.typ = PhycisObjectType::Static;
		floor_node.collision_shape = Some(CollisionShape::Box { size: Vec3::new(100.0, 0.1, 100.0) });
		state.nodes.insert(floor_node);

		let mut camera = Camera::new();
		camera.zfar = 1000.0;
		camera.node_id = Some(player_id);
		let camera_id = state.cameras.insert(camera);

		let gui_id = state.guis.insert(camera_view(camera_id));
		state.windows.insert(window().title("JUST A CUBE!!").ui(gui_id));
	}
}

fn main() {
	init_logging();
	let app = TestApp {};
	pge::run(app).unwrap();
}
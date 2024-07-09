use std::f32::consts::PI;

use pge::*;

#[derive(Debug, Clone)]
struct PressedKeys {
	forward: bool,
	backward: bool,
	left: bool,
	right: bool,
}

impl PressedKeys {
	pub fn new() -> Self {
		Self {
			forward: false,
			backward: false,
			left: false,
			right: false,
		}
	}

	pub fn to_vec3(&self) -> glam::Vec3 {
        let mut direction = glam::Vec3::ZERO;

        if self.forward {
            direction += glam::Vec3::Z;
        }
        if self.backward {
            direction -= glam::Vec3::Z;
        }
        if self.left {
            direction -= glam::Vec3::X;
        }
        if self.right {
            direction += glam::Vec3::X;
        }

        if direction.length_squared() > 0.0 {
            direction = direction.normalize();
        }

        direction
    }
}

pub fn compute_new_angle(
	last_yaw: f32,
	x_delta: f32,
	sensitivity: f32,
) -> f32 {
	let delta = x_delta * sensitivity;
	let new_yaw = last_yaw + delta;
	let new_yaw = new_yaw % (2.0*PI);

	if new_yaw < 0.0 {
		2.0*PI + new_yaw
	} else {
		new_yaw
	}
}

pub struct FpsShooter {
	sensitivity: f32,
	player_move_force: PhysicsForce,
	player_inx: Option<Index>,
	light_inx: Option<Index>,
	light_circle_i: f32,
	pressed_keys: PressedKeys,
	yaw: f32,
	pitch: f32,
	speed: f32,
}

impl FpsShooter {
	pub fn new() -> Self {
		Self {
			player_inx: None,
			light_inx: None,
			sensitivity: 0.001,
			player_move_force: PhysicsForce::new(),
			pressed_keys: PressedKeys::new(),
			yaw: 0.0,
			pitch: 0.0,
			speed: 10.0,
			light_circle_i: 0.0,
		}
	}
}

impl FpsShooter {
	pub fn rotate_player(&mut self, dx: f32, dy: f32) {
		self.yaw = compute_new_angle(self.yaw, dx, self.sensitivity);
		self.pitch = compute_new_angle(self.pitch, dy, self.sensitivity);
		// self.pitch = self.pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);
	}
}

impl pge::App for FpsShooter {
	fn on_create(&mut self, state: &mut State) {
		// let mut scene = Scene::new();
		// let mut floor = Node::new();
		// floor.physics.typ = PhycisObjectType::Static;
		// floor.mesh = Some(state.meshes.insert(plane(10.0, 10.0)));
		// let floor_id = state.nodes.insert(floor);
		// scene.nodes.push(floor_id);

		// let mut cube_node = Node::new();
		// cube_node.set_translation(-2.0, 0.0, 0.0);
		// cube_node.mesh = Some(state.meshes.insert(cube(1.0)));
		// state.nodes.insert(cube_node);

		// let mut cube_node = Node::new();
		// cube_node.set_translation(2.0, 0.0, 0.0);
		// cube_node.mesh = Some(state.meshes.insert(cube(1.0)));
		// state.nodes.insert(cube_node);

		let mut light_node = Node::new();
		light_node.set_translation(10.0, 10.0, 0.0);
		let light_inx = state.nodes.insert(light_node);
		self.light_inx = Some(light_inx);
		let mut light = PointLight::new();
		light.node_id = Some(light_inx);
		state.point_lights.insert(light);

		let cube_mesh = state.meshes.insert(cube(1.0));
		let cube_mesh2 = state.meshes.insert(cube(4.0).set_name("Big CUBE"));
		let plane_mesh = state.meshes.insert(plane(10.0, 10.0));



		// let mut cube_node = Node::new();
		// cube_node.name = Some("Cube1".to_string());
		// cube_node.set_translation(-50.0, 5.0, 0.0);
		// cube_node.mesh = Some(cube_mesh2);
		// cube_node.physics.typ = PhycisObjectType::None;
		// cube_node.physics.mass = 1.0;
		// state.nodes.insert(cube_node);

		// for i in 0..10 {
		// 	for j in 0..10 {
		// 		let mut cube_node = Node::new();
		// 		cube_node.set_translation(i as f32 * 3.0, 10.0, j as f32 * 3.0);
		// 		cube_node.mesh = Some(cube_mesh);
		// 		cube_node.physics.typ = PhycisObjectType::None;
		// 		cube_node.physics.mass = 1.0;
		// 		state.nodes.insert(cube_node);
		// 	}
		// }

		// let mut cube_node = Node::new();
		// cube_node.name = Some("Cube2".to_string());
		// cube_node.set_translation(30.0, 5.0, 0.0);
		// cube_node.mesh = Some(cube_mesh);
		// cube_node.physics.typ = PhycisObjectType::None;
		// cube_node.physics.mass = 1.0;
		// state.nodes.insert(cube_node);

		let mut player = Node::new();
		player.set_translation(0.0, 5.0, -20.0);
		// player.mesh = Some(state.meshes.insert(cube(1.0)));
		player.physics.typ = PhycisObjectType::Dynamic;
		player.physics.mass = 1.0;
		player.looking_at(0.0, 0.0, 0.0);
		player.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(1.0, 2.0, 1.0) });
		let player_id = state.nodes.insert(player);

		let mut cube_node = Node::new();
		cube_node.name = Some("Cube1".to_string());
		cube_node.set_translation(0.0, 20.0, 0.0);
		cube_node.mesh = Some(cube_mesh2);
		cube_node.physics.typ = PhycisObjectType::Dynamic;
		cube_node.physics.mass = 1.0;
		cube_node.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(4.0, 4.0, 4.0) });
		state.nodes.insert(cube_node);

		let mut plane_node = Node::new();
		plane_node.name = Some("Floor".to_string());
		plane_node.set_translation(0.0, -1.0, 0.0);
		plane_node.mesh = Some(plane_mesh);
		plane_node.physics.typ = PhycisObjectType::Static;
		plane_node.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(10.0, 0.1, 10.0) });
		state.nodes.insert(plane_node);

		let mut camera = Camera::new();
		camera.node_id = Some(player_id);
		let camera_id = state.cameras.insert(camera);
		// scene.nodes.push(player_id);
		println!("player id: {:?}", player_id);
		self.player_inx = Some(player_id);

		let gui = camera_view(camera_id);
		let gui_id = state.guis.insert(gui);

		// state.scenes.insert(scene);
		state.windows.insert(window().title("FPS Shooter1").cam(gui_id).lock_cursor(true));
		//.cam(gui_id));
		// state.windows.insert(window().title("FPS Shooter2").cam(gui_id));
		// state.windows.insert(window().title("FPS Shooter3").cam(gui_id));
		// state.windows.insert(window().title("FPS Shooter4").cam(gui_id));
	}

	fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {
		match action {
			KeyAction::Pressed => {
				match key {
					KeyboardKey::W => self.pressed_keys.forward = true,
					KeyboardKey::S => self.pressed_keys.backward = true,
					KeyboardKey::A => self.pressed_keys.left = true,
					KeyboardKey::D => self.pressed_keys.right = true,
					_ => {}
				}
			},
			KeyAction::Released => {
				match key {
					KeyboardKey::W => self.pressed_keys.forward = false,
					KeyboardKey::S => self.pressed_keys.backward = false,
					KeyboardKey::A => self.pressed_keys.left = false,
					KeyboardKey::D => self.pressed_keys.right = false,
					_ => {}
				}
			},
		};

		let player = match self.player_inx {
			Some(index) => match state.nodes.get_mut(index) {
				Some(node) => node,
				None => return,
			},
			None => return,
		};

		let dir = self.pressed_keys.to_vec3();

		let force = PhysicsForce {
			force: dir,
			id: 1,
			max_velocity: 200.0,
		};

		log::info!("move dir: {:?}", dir);

		// player.forces = vec![force];

		// let dir = self.pressed_keys.to_mat4();
		// println!("dir: {:?}", dir);
		// let player_inx = match self.player_inx {
		// 	Some(index) => index,
		// 	None => return,
		// };
		// let player = state.nodes.get_mut(player_inx).unwrap();
		// player.translation += player.rotation.inverse() * dir;
		// player.mov()
		// player.forces[0].direction = mat;	
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		//println!("mouse event: {:?} state: {:?}", event, state);
		match event {
			MouseEvent::Moved { dx, dy } => {
				let player_inx = match self.player_inx {
					Some(index) => index,
					None => return,
				};
				self.rotate_player(dx, dy);
				let player = state.nodes.get_mut(player_inx).unwrap();
				player.rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);
				// player.rotate(dx * self.sensitivity,  dy* self.sensitivity);
				// let rot = glam::Mat4::from_quat(glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0));
				// player.model = rot * player.model;
			},
		}
	}

	fn on_process(&mut self, state: &mut State, delta: f32) {
		if let Some(index) = self.light_inx {
			let light = state.nodes.get_mut(index).unwrap();
			self.light_circle_i += delta;
			let x = 10.0 * self.light_circle_i.cos();
			let z = 10.0 * self.light_circle_i.sin();
			light.set_translation(x, 10.0, z);
		}

		let player = match self.player_inx {
			Some(index) => match state.nodes.get_mut(index) {
				Some(node) => node,
				None => return,
			},
			None => return,
		};

		let amount = self.pressed_keys.to_vec3() * delta * self.speed;
		player.translation += player.rotation * amount;
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	simple_logger::init_with_level(log::Level::Info)?;
	Ok(pge::run(FpsShooter::new()).await?)
}

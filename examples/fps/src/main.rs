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

	pub fn to_mat4(&self) -> glam::Vec3 {
		let mut mat = glam::Vec3::ZERO;
		if self.forward {
			mat += glam::Vec3::Z;
		}
		if self.backward {
			mat -= glam::Vec3::Z;
		}
		if self.left {
			mat += glam::Vec3::X;
		}
		if self.right {
			mat -= glam::Vec3::X;
		}

		if mat.length() > 0.0 {
			mat = mat.normalize();
		}

		mat
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

#[test]
fn test_pressed_keys() {
	let keys = PressedKeys::new();
	assert_eq!(keys.to_mat4(), glam::Vec3::ZERO);
}

pub struct FpsShooter {
	sensitivity: f32,
	player_move_force: PhycicsForce,
	player_inx: Option<Index>,
	pressed_keys: PressedKeys,
	yaw: f32,
	pitch: f32,
}

impl FpsShooter {
	pub fn new() -> Self {
		Self {
			player_inx: None,
			sensitivity: 0.001,
			player_move_force: PhycicsForce::new(),
			pressed_keys: PressedKeys::new(),
			yaw: 0.0,
			pitch: 0.0,
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

		let mut cube_node = Node::new();
		cube_node.set_translation(-2.0, 0.0, 0.0);
		cube_node.mesh = Some(state.meshes.insert(cube(1.0)));
		state.nodes.insert(cube_node);

		let mut cube_node = Node::new();
		cube_node.set_translation(2.0, 0.0, 0.0);
		cube_node.mesh = Some(state.meshes.insert(cube(1.0)));
		state.nodes.insert(cube_node);
	
	
		let mut player = Node::new();
		player.set_translation(0.0, 0.0, -5.0);
		// player.mesh = Some(state.meshes.insert(cube(1.0)));
		player.physics.typ = PhycisObjectType::Dynamic;
		player.forces.push(PhycicsForce::new());
		// player.looking_at(0.0, 0.0, 0.0);
		let player_id = state.nodes.insert(player);

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

		let dir = self.pressed_keys.to_mat4();
		println!("dir: {:?}", dir);
		let player_inx = match self.player_inx {
			Some(index) => index,
			None => return,
		};
		let player = state.nodes.get_mut(player_inx).unwrap();
		player.translation += player.rotation.inverse() * dir;
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
				println!("yaw: {}, pitch: {}", self.yaw, self.pitch);
				player.rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, self.pitch, self.yaw, 0.0);
				// player.rotate(dx * self.sensitivity,  dy* self.sensitivity);
				// let rot = glam::Mat4::from_quat(glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0));
				// player.model = rot * player.model;
			},
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	pge::run(FpsShooter::new()).await?;

	// Engine::new(|engine| async move {
	// 	let world = engine.create_world().await;
	// 	let node = world.create_node();
	// 	node.set_mesh(plane(10.0, 10.0));
	// 	node.set_phycis_props(PhycicsProps {
	// 		typ: PhycisObjectType::Static,
	// 	});
	// 	let player = world.create_node();
	// 	player.set_translation(0.0, 1.0, 0.0);
	// 	player.set_phycis_props(PhycicsProps {
	// 		typ: PhycisObjectType::Dynamic,
	// 	});
	// 	let camera = world.create_camera();
	// 	player.set_camera(&camera);

	// 	let window = engine.create_window();
	// 	window.set_gui(camera_view(camera.id));

	// 	let mut pressed_keys = PressedKeys {
	// 		forward: false,
	// 		backward: false,
	// 		left: false,
	// 		right: false,
	// 	};

	// 	let move_force = PhycicsForce::new();
	// 	move_force.max_velocity = 3.0;

	// 	loop {
	// 		match engine.next_event().await {
	// 			Some(e) => {
	// 				match e {
	// 					Event::InputEvent(e) => {
	// 						match e {
	// 							InputEvent::KeyboardEvent(k) => {
	// 								match k.action {
	// 									KeyAction::Pressed => {
	// 										match k.key {
	// 											KeyboardKey::W => pressed_keys.forward = true,
	// 											KeyboardKey::S => pressed_keys.backward = true,
	// 											KeyboardKey::A => pressed_keys.left = true,
	// 											KeyboardKey::D => pressed_keys.right = true,
	// 											_ => {}
	// 										}
	// 									},
	// 									KeyAction::Released => {
	// 										match k.key {
	// 											KeyboardKey::W => pressed_keys.forward = false,
	// 											KeyboardKey::S => pressed_keys.backward = false,
	// 											KeyboardKey::A => pressed_keys.left = false,
	// 											KeyboardKey::D => pressed_keys.right = false,
	// 											_ => {}
	// 										}
	// 									},
	// 								}

	// 								println!("presed keys: {:?}", pressed_keys);

	// 								let mat = pressed_keys.to_vec3();
	// 								move_force.set_direction(mat);
	// 							},
	// 							InputEvent::MouseEvent(m) => {
	// 								match m {
	// 									MouseEvent::Moved { dx, dy } => {
	// 										println!("mouse moved: dx: {}, dy: {}", dx, dy);
	// 										let sensitivity = 0.001;
	// 										let dx = dx * sensitivity;
	// 										let dy = dy * sensitivity;
	// 										player.rotate(dx, dy, 0.0);
	// 									},
	// 								}
									
	// 							},
	// 						}

	// 					},
	// 					_ => {}
	// 				}
	// 			},
	// 			None => return Ok(()),
	// 		}
	// 	}
	// }).run().await?;
	Ok(())
}

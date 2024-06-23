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
		let mut mat = glam::Vec3::ZERO;
		if self.forward {
			mat = mat * glam::Vec3::new(0.0, 0.0, 1.0);
		}
		if self.backward {
			mat = mat * glam::Vec3::new(0.0, 0.0, -1.0);
		}
		if self.left {
			mat = mat * glam::Vec3::new(1.0, 0.0, 0.0);
		}
		if self.right {
			mat = mat * glam::Vec3::new(-1.0, 0.0, 0.0);
		}
		mat
	}
}

pub struct FpsShooter {
	sensitivity: f32,
	player_move_force: PhycicsForce,
	player_inx: Option<Index>,
	pressed_keys: PressedKeys,
}

impl FpsShooter {
	pub fn new() -> Self {
		Self {
			player_inx: None,
			sensitivity: 0.001,
			player_move_force: PhycicsForce::new(),
			pressed_keys: PressedKeys::new(),
		}
	}
}

impl pge::App for FpsShooter {
	fn on_create(&mut self, state: &mut State) {
		let mut scene = Scene::new();
		let mut floor = Node::new();
		floor.physics.typ = PhycisObjectType::Static;
		floor.set_mesh(state.meshes.insert(plane(10.0, 10.0)));
		let floor_id = state.nodes.insert(floor);
		scene.nodes.push(floor_id);
	
		let camera = Camera::new();
		let camera_id = state.cameras.insert(camera);
		let mut player = Node::new();
		player.translation = glam::Vec3::new(0.0, 1.0, 0.0);
		player.mesh = Some(state.meshes.insert(cube(1.0)));
		player.physics.typ = PhycisObjectType::Dynamic;
		player.camera = Some(camera_id);
		player.forces.push(PhycicsForce::new());
		let player_id = state.nodes.insert(player);
		scene.nodes.push(player_id);
		self.player_inx = Some(player_id);

		let gui = camera_view(camera_id);
		let gui_id = state.guis.insert(gui);

		state.scenes.insert(scene);
		state.windows.insert(window().title("FPS Shooter1").gui(gui_id));
		state.windows.insert(window().title("FPS Shooter2").gui(gui_id));
		state.windows.insert(window().title("FPS Shooter3").gui(gui_id));
		state.windows.insert(window().title("FPS Shooter4").gui(gui_id));
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

		let mat = self.pressed_keys.to_vec3();
		let player_inx = match self.player_inx {
			Some(index) => index,
			None => return,
		};
		let player = state.nodes.get_mut(player_inx).unwrap();
		player.forces[0].direction = mat;	
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Moved { dx, dy } => {
				let player_inx = match self.player_inx {
					Some(index) => index,
					None => return,
				};
				let player = state.nodes.get_mut(player_inx).unwrap();
				player.rotate(dx * self.sensitivity, dy * self.sensitivity, 0.0);
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

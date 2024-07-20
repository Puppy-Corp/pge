use std::f32::consts::PI;

use pge::*;
use rand::Rng;

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

	pub fn any_pressed(&self) -> bool {
		self.forward || self.backward || self.left || self.right
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
	player_inx: Option<Index>,
	light_inx: Option<Index>,
	light_circle_i: f32,
	pressed_keys: PressedKeys,
	yaw: f32,
	pitch: f32,
	speed: f32,
	dashing: bool,
	movement_force: f32,
}

impl FpsShooter {
	pub fn new() -> Self {
		Self {
			player_inx: None,
			light_inx: None,
			sensitivity: 0.001,
			pressed_keys: PressedKeys::new(),
			yaw: 0.0,
			pitch: 0.0,
			speed: 10.0,
			light_circle_i: 0.0,
			movement_force: 4000.0,
			dashing: false,
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
		let texture = Texture::new("./assets/gandalf.jpg");
	 	let texture_id = state.textures.insert(texture);

		let mut light_node = Node::new();
		light_node.set_translation(10.0, 10.0, 0.0);
		let light_inx = state.nodes.insert(light_node);
		self.light_inx = Some(light_inx);
		let mut light = PointLight::new();
		light.node_id = Some(light_inx);
		state.point_lights.insert(light);

		let cube_mesh = state.meshes.insert(cube(1.0).set_texture(texture_id));
		let plane_mesh = state.meshes.insert(plane(1.0, 1.0).set_texture(texture_id));

		let mut player = Node::new();
		player.name = Some("Player".to_string());
		player.set_translation(0.0, 30.0, -20.0);
		// player.mesh = Some(state.meshes.insert(cube(1.0)));
		player.physics.typ = PhycisObjectType::Dynamic;
		player.physics.mass = 70.0;
		player.looking_at(0.0, 0.0, 0.0);
		player.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(1.0, 2.0, 1.0) });
		let player_id = state.nodes.insert(player);


		//Spawn random cubes
		let mut rng = rand::thread_rng();
		for i in 0..10 {
			let x = rng.gen_range(-20.0..20.0);
			let z = rng.gen_range(-20.0..20.0);
			let mut cube_node = Node::new();
			cube_node.name = Some(format!("Cube{}", i));
			cube_node.set_translation(x, 10.0, z);
			cube_node.mesh = Some(cube_mesh);
			cube_node.physics.typ = PhycisObjectType::Dynamic;
			cube_node.physics.mass = 10.0;
			cube_node.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(1.0, 1.0, 1.0) });
			state.nodes.insert(cube_node);
		}

		// // spawn static cube
		// let mut cube_node = Node::new();
		// cube_node.name = Some("Cube".to_string());
		// cube_node.set_translation(0.0, 0.0, 0.0);
		// cube_node.mesh = Some(cube_mesh);
		// cube_node.physics.typ = PhycisObjectType::Static;
		// cube_node.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(1.0, 1.0, 1.0) });
		// state.nodes.insert(cube_node);

		let plane_size = 1000.0;

		let mut plane_node = Node::new();
		plane_node.name = Some("Floor".to_string());
		plane_node.set_translation(0.0, -1.0, 0.0);
		plane_node.mesh = Some(plane_mesh);
		plane_node.physics.typ = PhycisObjectType::Static;
		plane_node.scale = glam::Vec3::new(plane_size, 1.0, plane_size);
		plane_node.collision_shape = Some(CollisionShape::Box { size: glam::Vec3::new(plane_size, 0.1, plane_size) });
		state.nodes.insert(plane_node);

		let mut camera = Camera::new();
		camera.zfar = 1000.0;
		camera.node_id = Some(player_id);
		let camera_id = state.cameras.insert(camera);
		self.player_inx = Some(player_id);

		let gui = stack(&[
			camera_view(camera_id),
			rect().background_color(Color::GREEN).height(0.1).anchor_bottom()
		]);
		let gui_id = state.guis.insert(gui);

		state.windows.insert(window().title("FPS Shooter1").ui(gui_id).lock_cursor(true));
	}

	fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {
		match action {
			KeyAction::Pressed => {
				match key {
					KeyboardKey::W => self.pressed_keys.forward = true,
					KeyboardKey::S => self.pressed_keys.backward = true,
					KeyboardKey::A => self.pressed_keys.left = true,
					KeyboardKey::D => self.pressed_keys.right = true,
					KeyboardKey::Space => {
						let player_inx = match self.player_inx {
							Some(index) => index,
							None => return,
						};
						let player = state.nodes.get_mut(player_inx).unwrap();
						player.physics.velocity.y = 10.0;
					},
					KeyboardKey::ShiftLeft => {
						self.dashing = true;
					},
					_ => {}
				}
			},
			KeyAction::Released => {
				match key {
					KeyboardKey::W => self.pressed_keys.forward = false,
					KeyboardKey::S => self.pressed_keys.backward = false,
					KeyboardKey::A => self.pressed_keys.left = false,
					KeyboardKey::D => self.pressed_keys.right = false,
					KeyboardKey::ShiftLeft => {
						self.dashing = false;
					},
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

		// let dir = self.pressed_keys.to_vec3();
		// player.physics.force = player.rotation * dir * 300.0;
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		match event {
			MouseEvent::Moved { dx, dy } => {
				let player_inx = match self.player_inx {
					Some(index) => index,
					None => return,
				};
				self.rotate_player(dx, dy);
				let player = state.nodes.get_mut(player_inx).unwrap();
				player.rotation = glam::Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, 0.0);
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

		if let Some(player_inx) = self.player_inx {
			if let Some(player) = state.nodes.get_mut(player_inx) {
				let current_speed = player.physics.velocity.length();
				if self.pressed_keys.any_pressed() {
					let dir = self.pressed_keys.to_vec3();
					let mut force = player.rotation * dir;

					if force.x > 0.0 && player.physics.velocity.x < 0.0 {
						force.x += -player.physics.velocity.x * self.movement_force;
					} else if force.x < 0.0 && player.physics.velocity.x > 0.0 {
						force.x += -player.physics.velocity.x * self.movement_force;
					} else if current_speed < 25.0 {
						force.x *= self.movement_force;
					}

					if force.z > 0.0 && player.physics.velocity.z < 0.0 {
						force.z += -player.physics.velocity.z * self.movement_force;
					} else if force.z < 0.0 && player.physics.velocity.z > 0.0 {
						force.z += -player.physics.velocity.z * self.movement_force;
					} else if current_speed < 25.0 {
						force.z *= self.movement_force;
					}

					force.y = 0.0;

					player.physics.force = force;
					log::info!("force: {:?}", player.physics.force);
				} else {
					// We calculate force opposite of momevement to slow down the player
					let force = -player.physics.velocity.xz() * self.movement_force;
					player.physics.force = glam::Vec3::new(force.x, 0.0, force.y);
					//player.physics.force = glam::Vec3::ZERO;
				}
			}
		}

		if self.dashing {
			let player_inx = match self.player_inx {
				Some(index) => index,
				None => return,
			};
			let player = match state.nodes.get_mut(player_inx) {
				Some(node) => node,
				None => return,
			};
			let dir = player.rotation * Vec3::new(0.0, 0.0, 1.0);
			player.physics.velocity = dir * 100.0;
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	simple_logger::init_with_level(log::Level::Info)?;
	Ok(pge::run(FpsShooter::new()).await?)
}

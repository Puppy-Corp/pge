use std::time::Duration;

use glam::Quat;
use pge::*;
use text::FontMesh;
use tokio::time::sleep;

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

	pub fn to_mat4(&self) -> glam::Mat4 {
		let mut mat = glam::Mat4::IDENTITY;
		if self.forward {
			mat = mat * glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, 1.0));
		}
		if self.backward {
			mat = mat * glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, -1.0));
		}
		if self.left {
			mat = mat * glam::Mat4::from_translation(glam::Vec3::new(1.0, 0.0, 0.0));
		}
		if self.right {
			mat = mat * glam::Mat4::from_translation(glam::Vec3::new(-1.0, 0.0, 0.0));
		}
		mat
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Engine::new(|mut handle| async move {
		let mut scene = Scene::new();


		let mut root = Node::new();
		// let material = Material::new();

		// let mut cube_node = Node::new();
		// let mut cube_mesh = cube(1.0);
		// cube_mesh.set_material(material);
		// cube_node.set_mesh(cube_mesh);
		// root.add_node(cube_node);

		let mut light_node = Node::new();
		light_node.set_translation(0.0, 0.0, 0.0);
		let light = PointLight::new();
		light_node.set_point_light(light);
		let light_cube = cube(0.1);
		light_node.set_mesh(light_cube);
		let light_id = light_node.id;
		scene.add_node(light_node);

		
		let handle2 = handle.clone();
		tokio::spawn(async move {
			let mut z = 0.0;
			let mut dir = 1.0;
			let speed = 0.08;

			loop {
				sleep(Duration::from_millis(20)).await;
				let amount = speed * dir;
				handle2.apply_transformation(light_id, glam::Mat4::from_translation(glam::Vec3::new(0.0, 0.0, amount)));
				z += amount;
				if z > 1.5 {
					dir = -1.0;
				}
				if z < -1.5 {
					dir = 1.0;
				}
			}
		});

		
		let mut light_node = Node::new();
		light_node.set_translation(4.0, 0.0, 0.0);
		let light = PointLight::new();
		light_node.set_point_light(light);
		let light_cube = cube(0.1);
		light_node.set_mesh(light_cube);
		scene.add_node(light_node);


		let mut camera_node = Node::new();
		let camera_node_id = camera_node.id;
		let camera = Camera::new();
		// let scene_cam = SceneCam::new(&camera);
		camera_node.set_camera(camera);
		camera_node.set_translation(0.0, 0.0, -6.0);
		camera_node.looking_at(0.0, 0.0, 0.0);
		root.add_node(camera_node.clone());

		

		let mut window = Window::new();
		window.title = "BIG box".to_string();
		window.lock_cursor = true;
		// window.body = view().add(scene_cam).into();
		handle.save_window(&window);

		let cube_mesh = cube(1.0);
		let mut node = Node::new();
		node.set_translation(-2.0, 0.0, 0.0);
		node.set_mesh(cube_mesh);
		root.add_node(node);
		
		let cube_mesh = cube(1.0);
		let mut node = Node::new();
		let cube_node_id = node.id;
		node.set_translation(2.0, 0.0, 0.0);
		node.set_mesh(cube_mesh);		
		root.add_node(node);

		let font = FontMesh::load("./fonts/Roboto-Regular.ttf")?;
		let mut anode = Node::new();
		let amesh = font.get_mesh('A').unwrap();
		println!("amsh: {:?}", amesh);
		//let amesh = cube(1.0);
		anode.set_mesh(amesh);
		anode.set_translation(0.0, 3.0, 0.0);
		anode.scale(2.0, 2.0, 2.0);
		root.add_node(anode);

		scene.add_node(root);
		handle.save_scene(scene);
		
		let mut pressed_keys = PressedKeys {
			forward: false,
			backward: false,
			left: false,
			right: false,
		};

		loop {
			match handle.next_event().await {
				Some(e) => {
					match e {
						Event::InputEvent(e) => {
							match e {
								InputEvent::KeyboardEvent(k) => {
									match k.action {
										KeyAction::Pressed => {
											match k.key {
												KeyboardKey::W => pressed_keys.forward = true,
												KeyboardKey::S => pressed_keys.backward = true,
												KeyboardKey::A => pressed_keys.left = true,
												KeyboardKey::D => pressed_keys.right = true,
												_ => {}
											}
										},
										KeyAction::Released => {
											match k.key {
												KeyboardKey::W => pressed_keys.forward = false,
												KeyboardKey::S => pressed_keys.backward = false,
												KeyboardKey::A => pressed_keys.left = false,
												KeyboardKey::D => pressed_keys.right = false,
												_ => {}
											}
										},
									}

									println!("presed keys: {:?}", pressed_keys);

									// if pressed_keys.forward {
									// 	light_node.set_translation(light_node.translation.x, light_node.translation.y, light_node.translation.z + 0.1);
									// }

									// if pressed_keys.backward {
									// 	light_node.set_translation(light_node.translation.x, light_node.translation.y, light_node.translation.z - 0.1);
									// }

									// handle

									// let animation = Animation::new()
									// 	.every(Duration::from_secs(1))
									// 	.transform(pressed_keys.to_mat4());

									let mat = pressed_keys.to_mat4();
		
									handle.apply_transformation(camera_node_id, mat);
								},
								InputEvent::MouseEvent(m) => {
									match m {
										MouseEvent::Moved { dx, dy } => {
											println!("mouse moved: dx: {}, dy: {}", dx, dy);
											let sensitivity = 0.001;
											let dx = dx * sensitivity;
											let dy = dy * sensitivity;
											// // let rot = Quat::from_euler(glam::EulerRot::XYZ, dx, dy, 0.0);
											// let mat = glam::Mat4::from_euler(glam::EulerRot::XYZ, dy, dx, 0.0);
											// handle.apply_transformation(camera_node_id, mat);
											handle.rotate_node(camera_node_id, dx, dy);
										},
									}
									
								},
							}

						},
						_ => {}
					}
				},
				None => todo!(),
			}
		}

		sleep(Duration::from_secs(120)).await;
	}).run().await?;
	Ok(())
}
 
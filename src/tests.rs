
#[cfg(test)]
mod tests {
	use std::time::Duration;
use std::time::Instant;

use engine::Engine;
	use mock_hardware::MockHardware;
	use crate::*;

	#[test]
	fn it_works() {
		let light_position = glam::Vec3::new(-2.0, 1.5, 0.0);
		let world_position = glam::Vec3::new(-2.0, 0.0, 0.0);
		let light_direction = (light_position - world_position).normalize();
		let normal = glam::Vec3::new(0.0, 1.0, 0.0);
		println!("light direction: {:?}", light_direction);

		let diffuce = normal.dot(light_direction).max(0.0);
		println!("diffuse: {:?}", diffuce);

		let light_color = glam::Vec3::new(1.0, 1.0, 1.0);

		let diffuse_color = light_color * diffuce;
		println!("diffuse color: {:?}", diffuse_color);

		let incolor = glam::Vec3::new(1.0, 0.0, 0.0);
		let result = diffuse_color * incolor;
		println!("result: {:?}", result);
	}

	#[test]
	fn object_does_not_fall_through_floor() {
		#[derive(Default)]
		struct TestApp {
			pub dynamic_node_id: Option<ArenaId<Node>>,
		}

		impl App for TestApp {
			fn on_create(&mut self, state: &mut crate::State) {
				let scene = Scene::new();
				let scene_id = state.scenes.insert(scene);

				// Create a static floor node
				let floor_node = Node {
					physics: PhysicsProps {
						typ: PhycisObjectType::Static,
						stationary: true,
						..Default::default()
					},
					translation: Vec3::new(0.0, 1.0, 0.0),
					collision_shape: Some(CollisionShape::Box { size: Vec3::new(10.0, 1.0, 10.0) }),
					parent: NodeParent::Scene(scene_id),
					..Default::default()
				};
				let floor_id = state.nodes.insert(floor_node);
				
				// Create a dynamic object above the floor
				let dynamic_node = Node {
					physics: PhysicsProps {
						typ: PhycisObjectType::Dynamic,
						mass: 1.0,
						stationary: false,
						..Default::default()
					},
					translation: Vec3::new(0.0, 10.0, 0.0),
					collision_shape: Some(CollisionShape::Box { size: Vec3::new(1.0, 1.0, 1.0) }),
					parent: NodeParent::Scene(scene_id),
					..Default::default()
				};
				self.dynamic_node_id = Some(state.nodes.insert(dynamic_node));
			}
		}

		let hardware = MockHardware::new();

		let mut engine = Engine::new(TestApp::default(), hardware);

		let timer = Instant::now();
		let dt = 0.016;
		for _ in 0..600 {
			engine.render(dt);
		}
		let duration = timer.elapsed();
		println!("duration: {:?}", duration);
		println!("per frame: {:?} micros", duration.as_micros() / 600);

		let dynamic_node = engine.state.nodes.get(&engine.app.dynamic_node_id.unwrap()).unwrap();
		println!("dynamic_node.translation: {:?}", dynamic_node.translation);

		assert!(dynamic_node.translation.y >= 0.0, "Dynamic object fell through the floor");
	}

	#[test]
	fn fast_object_does_not_fall_through_floor() {
		#[derive(Default)]
		struct TestApp {
			pub dynamic_node_id: Option<ArenaId<Node>>,
		}

		impl App for TestApp {
			fn on_create(&mut self, state: &mut crate::State) {
				let scene = Scene::new();
				let scene_id = state.scenes.insert(scene);

				// Create a static floor node
				let floor_node = Node {
					physics: PhysicsProps {
						typ: PhycisObjectType::Static,
						stationary: true,
						..Default::default()
					},
					translation: Vec3::new(0.0, 1.0, 0.0),
					collision_shape: Some(CollisionShape::Box { size: Vec3::new(10.0, 1.0, 10.0) }),
					parent: NodeParent::Scene(scene_id),
					..Default::default()
				};
				let floor_id = state.nodes.insert(floor_node);
				
				// Create a dynamic object above the floor
				let dynamic_node = Node {
					physics: PhysicsProps {
						typ: PhycisObjectType::Dynamic,
						mass: 1.0,
						stationary: false,
						velocity: Vec3::new(0.0, -500.0, 0.0),
						..Default::default()
					},
					translation: Vec3::new(0.0, 10.0, 0.0),
					collision_shape: Some(CollisionShape::Box { size: Vec3::new(1.0, 1.0, 1.0) }),
					parent: NodeParent::Scene(scene_id),
					..Default::default()
				};
				self.dynamic_node_id = Some(state.nodes.insert(dynamic_node));
			}
		}

		let hardware = MockHardware::new();

		let mut engine = Engine::new(TestApp::default(), hardware);

		let timer = Instant::now();
		let dt = 0.016;
		for _ in 0..600 {
			engine.render(dt);
		}
		let duration = timer.elapsed();
		println!("duration: {:?}", duration);
		println!("per frame: {:?} micros", duration.as_micros() / 600);

		let dynamic_node = engine.state.nodes.get(&engine.app.dynamic_node_id.unwrap()).unwrap();
		println!("dynamic_node.translation: {:?}", dynamic_node.translation);

		assert!(dynamic_node.translation.y >= 0.0, "Fast object fell through the floor");
	}
}

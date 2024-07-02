use crate::Node;

fn sum_forces(node: &Node) -> glam::Vec3 {
	let mut total_force = glam::Vec3::ZERO;
	for force in &node.forces {
		total_force += force.force;
	}
	total_force
}

pub fn node_physics_update(node: &mut Node, dt: f32) {
	let mass = node.physics.mass;
	let gravity_force = if mass > 0.0 { glam::Vec3::new(0.0, -9.81, 0.0) * mass } else { glam::Vec3::ZERO };
	let total_force = sum_forces(node) + gravity_force;
	let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };

	node.physics.velocity += acceleration * dt;
	node.translation += node.physics.velocity * dt;
	node.physics.acceleration = acceleration;
}
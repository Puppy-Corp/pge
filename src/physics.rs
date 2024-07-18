use std::time::Instant;

use glam::Vec3;
use thunderdome::Index;

use crate::debug::ChangePrinter;
use crate::spatial_grid::SpatialGrid;
use crate::Node;
use crate::PhycisObjectType;
use crate::State;
use crate::AABB;

pub struct PhycisTiming {
	pub node_update_time: u32,
	pub broad_phase_time: u32,
	pub narrow_phase_time: u32,
	pub resolve_collision_time: u32,
	pub total_time: u32,
}

pub struct Collision {
	pub node1: Index,
	pub node2: Index,
	pub normal: glam::Vec3,
	pub point: glam::Vec3,
	pub penetration_depth: f32,
}

// fn calculate_impulse(node1: &Node, node2: &Node, collision: &Collision, coeff_of_friction: f32) -> (glam::Vec3, glam::Vec3) {
//     let relative_velocity = node2.physics.velocity - node1.physics.velocity;
//     let velocity_along_normal = relative_velocity.dot(collision.normal);

//     // If velocities are separating, no impulse is needed
//     if velocity_along_normal > 0.0 {
//         return (Vec3::ZERO, Vec3::ZERO);
//     }

//     // Calculate the sum of inverse masses
//     let inv_mass1 = if node1.physics.mass != 0.0 { 1.0 / node1.physics.mass } else { 0.0 };
//     let inv_mass2 = if node2.physics.mass != 0.0 { 1.0 / node2.physics.mass } else { 0.0 };
//     let inv_mass_sum = inv_mass1 + inv_mass2;

//     // Calculate the impulse scalar
//     let impulse_magnitude = -(1.0 + 0.0) * velocity_along_normal / inv_mass_sum;

//     // Impulse in normal direction
//     let normal_impulse = collision.normal * impulse_magnitude;

//     // Assuming friction computations if needed (simplified version)
//     let relative_velocity_tangent = relative_velocity - velocity_along_normal * collision.normal;
//     let friction_impulse = -relative_velocity_tangent; // This can be refined based on specific friction calculations

//     (normal_impulse, friction_impulse)
// }

fn calculate_impulse(node1: &Node, node2: &Node, collision: &Collision, coeff_of_friction: f32) -> (glam::Vec3, glam::Vec3) {
    let rel_v = node2.physics.velocity - node1.physics.velocity;

    // Calculate normal impulse
    let normal_impulse = 1.0 * rel_v.dot(collision.normal);
    let normal_impulse_vec = normal_impulse * collision.normal;

    // Calculate friction direction
    let tangent_dir = (rel_v - rel_v.dot(collision.normal) * collision.normal).normalize_or_zero();
    
    // Calculate friction impulse
    let friction_impulse_mag = coeff_of_friction * normal_impulse.abs();
    let friction_impulse_vec = friction_impulse_mag * tangent_dir;

    (normal_impulse_vec, friction_impulse_vec)
}

fn calculate_collision_point(a: &AABB, b: &AABB) -> [f32; 3] {
	let center_a = [(a.min[0] + a.max[0]) / 2.0, (a.min[1] + a.max[1]) / 2.0, (a.min[2] + a.max[2]) / 2.0];
	let center_b = [(b.min[0] + b.max[0]) / 2.0, (b.min[1] + b.max[1]) / 2.0, (b.min[2] + b.max[2]) / 2.0];

	let mut collision_point = [0.0; 3];

	for i in 0..3 {
		if center_a[i] < b.min[i] {
			collision_point[i] = b.min[i];
		} else if center_a[i] > b.max[i] {
			collision_point[i] = b.max[i];
		} else {
			collision_point[i] = center_a[i];
		}
	}

	collision_point
}

fn calculate_collision_normal(a: &AABB, b: &AABB) -> [f32; 3] {
	let center_a = [(a.min[0] + a.max[0]) / 2.0, (a.min[1] + a.max[1]) / 2.0, (a.min[2] + a.max[2]) / 2.0];
	let center_b = [(b.min[0] + b.max[0]) / 2.0, (b.min[1] + b.max[1]) / 2.0, (b.min[2] + b.max[2]) / 2.0];

	let overlap_x = (a.max[0].min(b.max[0]) - a.min[0].max(b.min[0])).max(0.0);
	let overlap_y = (a.max[1].min(b.max[1]) - a.min[1].max(b.min[1])).max(0.0);
	let overlap_z = (a.max[2].min(b.max[2]) - a.min[2].max(b.min[2])).max(0.0);

	let min_overlap = overlap_x.min(overlap_y).min(overlap_z);

	let normal = if min_overlap == overlap_x {
		if center_a[0] < center_b[0] {
			[-1.0, 0.0, 0.0]
		} else {
			[1.0, 0.0, 0.0]
		}
	} else if min_overlap == overlap_y {
		if center_a[1] < center_b[1] {
			[0.0, -1.0, 0.0]
		} else {
			[0.0, 1.0, 0.0]
		}
	} else {
		if center_a[2] < center_b[2] {
			[0.0, 0.0, -1.0]
		} else {
			[0.0, 0.0, 1.0]
		}
	};

	normal
}

fn calculate_penetration_depth(a: &AABB, b: &AABB) -> f32 {
    let overlap_x = (a.max[0].min(b.max[0]) - a.min[0].max(b.min[0])).max(0.0);
    let overlap_y = (a.max[1].min(b.max[1]) - a.min[1].max(b.min[1])).max(0.0);
    let overlap_z = (a.max[2].min(b.max[2]) - a.min[2].max(b.min[2])).max(0.0);

    overlap_x.min(overlap_y).min(overlap_z)
}

pub struct PhycicsSystem {
	printer: ChangePrinter,
}

impl PhycicsSystem {
	pub fn new() -> Self {
		Self {
			printer: ChangePrinter::new()
		}
	}

	fn sum_forces(&mut self, node: &Node) -> glam::Vec3 {
		let mut total_force = glam::Vec3::ZERO;
		for force in &node.forces {
			total_force += force.force;
		}
	
		if node.forces.len() > 0 {
			self.printer.print(node.id as u32, format!("forces: {:?}", node.forces));
		}
	
		total_force
	}
	
	pub fn node_physics_update(&mut self, node: &mut Node, dt: f32) {
		let mass = node.physics.mass;
		let gravity_force = if mass > 0.0 { glam::Vec3::new(0.0, -9.81, 0.0) * mass } else { glam::Vec3::ZERO };
		let total_force = self.sum_forces(node) + gravity_force;
		let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };
		node.physics.velocity += acceleration * dt;
		node.translation += node.physics.velocity * dt;
		log::debug!("[{}] total force: {:?} acceleration: {:?} velocity: {:?} translation: {:?}", node.id, total_force, acceleration, node.physics.velocity, node.translation);	
		node.physics.acceleration = acceleration;
	}
	
	fn update_nodes(&mut self, state: &mut State, dt: f32) {
		for (_, node) in &mut state.nodes {
			if node.physics.typ == crate::PhycisObjectType::Dynamic && !node.physics.stationary {
				self.node_physics_update(node, dt);
			}
		}
	}	
	
	fn broad_phase_collisions(&mut self, state: &mut State, grid: &SpatialGrid) -> Vec<Collision> {
		let mut collisions = Vec::new();
		for cell in grid.get_cells() {
			for i in 0..cell.len() {
				let node1_id = cell[i];
				let node1_aabb = match grid.get_node_rect(node1_id) {
					Some(a) => a,
					None => continue
				};
				for j in i+1..cell.len() {
					let node2_id = cell[j];
					let node2_aabb = match grid.get_node_rect(node2_id) {
						Some(a) => a,
						None => continue
					};
					if node1_aabb.intersects(&node2_aabb) {
						if collisions.iter().any(|c: &Collision| 
							(c.node1 == node1_id && c.node2 == node2_id) || 
							(c.node1 == node2_id && c.node2 == node1_id)) {
							continue;
						}
	
						let penetration_depth = calculate_penetration_depth(&node1_aabb, &node2_aabb);

						log::info!("penetration_depth: {:?}", penetration_depth);

						collisions.push(Collision {
							node1: node1_id,
							node2: node2_id,
							normal: calculate_collision_normal(&node1_aabb, &node2_aabb).into(),
							point: calculate_collision_point(&node1_aabb, &node2_aabb).into(),
							penetration_depth,
						});
					}
				}
			}
		}
		collisions
	}
	
	pub fn physics_update(&mut self, state: &mut State, grid: &mut SpatialGrid, dt: f32) -> PhycisTiming {
		let timer = Instant::now();
		self.update_nodes(state, dt);
		let node_update_time = timer.elapsed().as_millis() as u32;
		let collisions = self.broad_phase_collisions(state, grid);
		let broad_phase_time = timer.elapsed().as_millis() as u32 - node_update_time;
	
		if collisions.len() > 0 {
			self.printer.print(9999, format!("collisions: {:?}", collisions.len()));
			for collision in collisions {
				let node1 = state.nodes.get(collision.node1).unwrap();
				let node2 = state.nodes.get(collision.node2).unwrap();
				let (normal_impulse, friction_impulse) = calculate_impulse(node1, node2, &collision, /* coefficient of friction */ 0.5);
				//self.printer.print(100_000, format!("friction_impulse: {:?}", friction_impulse));
				let total_mass = node1.physics.mass + node2.physics.mass;
				let correction_ratio = 1.0; // tweakable parameter for how strong the correction should be

				let corr_vec1 = if total_mass > 0.0 { collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node2.physics.mass } else { glam::Vec3::ZERO };
				let corr_vec2 = if total_mass > 0.0 { collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node1.physics.mass } else { glam::Vec3::ZERO };

				// if total_mass > 0.0 {
				// 	if let Some(node1) = state.nodes.get_mut(collision.node1) {
				// 		let correction_vector = collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node2.physics.mass;
				// 		node1.translation -= correction_vector;
				// 	}
				// 	if let Some(node2) = state.nodes.get_mut(collision.node2) {
				// 		let correction_vector = collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node1.physics.mass;
				// 		node2.translation += correction_vector;
				// 	}
				// }

				if let Some(node1) = state.nodes.get_mut(collision.node1) {
					if node1.physics.typ == PhycisObjectType::Dynamic {
						node1.physics.velocity += normal_impulse + friction_impulse;
						node1.translation += corr_vec1;
					}
				}
				if let Some(node2) = state.nodes.get_mut(collision.node2) {
					if node2.physics.typ == PhycisObjectType::Dynamic {
						node2.physics.velocity -= normal_impulse + friction_impulse;
						node2.translation -= corr_vec2;
					}
				}

				// // Resolve interpenetration
				// let total_mass = node1.physics.mass + node2.physics.mass;
				// let correction_ratio = 0.5; // tweakable parameter for how strong the correction should be
				// if total_mass > 0.0 {
				// 	if let Some(node1) = state.nodes.get_mut(collision.node1) {
				// 		let correction_vector = collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node2.physics.mass;
				// 		node1.translation -= correction_vector;
				// 	}
				// 	if let Some(node2) = state.nodes.get_mut(collision.node2) {
				// 		let correction_vector = collision.normal * correction_ratio * (collision.penetration_depth / total_mass) * node1.physics.mass;
				// 		node2.translation += correction_vector;
				// 	}
				// }
			}
		}
		let resolve_collision_time = timer.elapsed().as_millis() as u32 - broad_phase_time;
		let total_time = timer.elapsed().as_millis() as u32;
		
		PhycisTiming {
			node_update_time,
			broad_phase_time,
			narrow_phase_time: 0,
			resolve_collision_time,
			total_time,
		}
	}
}
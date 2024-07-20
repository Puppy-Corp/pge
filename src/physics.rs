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
	pub correction: glam::Vec3,
}

fn calculate_impulse(node1: &Node, node2: &Node, collision: &Collision, 
	coeff_of_restitution: f32, coeff_of_friction: f32) -> (glam::Vec3, glam::Vec3) {
    let rel_v = node2.physics.velocity - node1.physics.velocity;
    let velocity_along_normal = rel_v.dot(collision.normal);

    if velocity_along_normal < 0.0 {
        return (Vec3::ZERO, Vec3::ZERO);
    }

    let inv_mass1 = if node1.physics.mass != 0.0 { 1.0 / node1.physics.mass } else { 0.0 };
    let inv_mass2 = if node2.physics.mass != 0.0 { 1.0 / node2.physics.mass } else { 0.0 };
    let inv_mass_sum = inv_mass1 + inv_mass2;
    let impulse_magnitude = -(1.0 + coeff_of_restitution) * (velocity_along_normal / inv_mass_sum);
    let normal_impulse = collision.normal * impulse_magnitude;
    let relative_velocity_tangent = rel_v - velocity_along_normal * collision.normal;
    let tangent_velocity_magnitude = relative_velocity_tangent.length();

    let mut tangent_impulse = glam::Vec3::ZERO;
    if tangent_velocity_magnitude > 0.0 {
        let tangent_direction = relative_velocity_tangent / tangent_velocity_magnitude;
        let friction_magnitude = coeff_of_friction * -impulse_magnitude;
        tangent_impulse = tangent_direction * friction_magnitude;
    }

    (normal_impulse, -tangent_impulse)
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

#[derive(Debug, Default, Clone)]
pub struct PhysicsSystem {
	printer: ChangePrinter,
	gravity: glam::Vec3,
}

impl PhysicsSystem {
	pub fn new() -> Self {
		Self {
			printer: ChangePrinter::new(),
			gravity: glam::Vec3::new(0.0, -10.0, 0.0),
		}
	}
	
	pub fn node_physics_update(&mut self, node: &mut Node, dt: f32) {
		let mass = node.physics.mass;
		let gravity_force = if mass > 0.0 { self.gravity * mass } else { glam::Vec3::ZERO };
		let total_force = node.physics.force + gravity_force;
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
		for cell in grid.cells.values() {
			if cell.len() < 2 {
				continue;
			}

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
	
						let correction = node1_aabb.get_correction(&node2_aabb);

						collisions.push(Collision {
							node1: node1_id,
							node2: node2_id,
							normal: calculate_collision_normal(&node1_aabb, &node2_aabb).into(),
							point: calculate_collision_point(&node1_aabb, &node2_aabb).into(),
							correction,
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

				if node1.physics.typ == PhycisObjectType::Static && node2.physics.typ == PhycisObjectType::Static {
					continue;
				}

				let (normal_impulse, friction_impulse) = calculate_impulse(node1, node2, &collision, 0.3, 0.2);

				if let Some(node1) = state.nodes.get_mut(collision.node1) {
					if node1.physics.typ == PhycisObjectType::Dynamic {
						node1.physics.velocity -= (normal_impulse + friction_impulse) / node1.physics.mass;
						node1.translation += collision.correction;
					}
				}
				if let Some(node2) = state.nodes.get_mut(collision.node2) {
					if node2.physics.typ == PhycisObjectType::Dynamic {
						node2.physics.velocity += (normal_impulse + friction_impulse) / node2.physics.mass;
						node2.translation -= collision.correction;
					}
				}
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
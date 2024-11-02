use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use glam::Vec3;
use crate::spatial_grid::SpatialGrid;
use crate::state::State;
use crate::ArenaId;
use crate::Node;
use crate::PhycisObjectType;
use crate::AABB;

#[derive(Debug, Clone)]
pub struct Collision {
	pub node1: ArenaId<Node>,
	pub node2: ArenaId<Node>,
	pub normal: glam::Vec3,
	pub point: glam::Vec3,
	pub correction: glam::Vec3,
}

struct Impulse {
	normal: glam::Vec3,
	tangent: glam::Vec3,
	/// Node 1 angular velocity at point of contact
	r1: glam::Vec3,
	/// Node 2 angular velocity at point of contact
	r2: glam::Vec3,
}

fn calculate_impulse(node1: &Node, node2: &Node, collision: &Collision, coeff_of_restitution: f32, coeff_of_friction: f32) -> Impulse {
    let rel_v = node2.physics.velocity - node1.physics.velocity;
    let velocity_along_normal = rel_v.dot(collision.normal);

    if velocity_along_normal < 0.0 {
        return Impulse {
			normal: glam::Vec3::ZERO,
			tangent: glam::Vec3::ZERO,
			r1: glam::Vec3::ZERO,
			r2: glam::Vec3::ZERO,
		};
    }

	let r1 = collision.point - node1.center_of_mass();
	let r2 = collision.point - node2.center_of_mass();

	let angular_velocity1 = node1.physics.angular_velocity.cross(r1);
	let angular_velocity2 = node2.physics.angular_velocity.cross(r2);

	let rel_velocity = (node2.physics.velocity + angular_velocity2) - (node1.physics.velocity + angular_velocity1);
	let vel_along_normal = rel_velocity.dot(collision.normal);

    let inv_mass1 = if node1.physics.mass == 0.0 { 0.0 } else { 1.0 / node1.physics.mass };
    let inv_mass2 = if node2.physics.mass == 0.0 { 0.0 } else { 1.0 / node2.physics.mass };

	let inv_inertia1 = if node1.physics.moment_of_inertia == glam::Vec3::ZERO { glam::Vec3::ZERO } else { node1.physics.moment_of_inertia.recip() };
	let inv_inertia2 = if node2.physics.moment_of_inertia == glam::Vec3::ZERO { glam::Vec3::ZERO } else { node2.physics.moment_of_inertia.recip() };

	let numerator = -(1.0 + coeff_of_restitution) * vel_along_normal;
	let denominator = inv_mass1 + inv_mass2 + collision.normal.dot((inv_inertia1.cross(r1.cross(collision.normal))) + (inv_inertia2.cross(r2.cross(collision.normal))));
	let impulse_scalar = if denominator == 0.0 { 0.0 } else { numerator / denominator };
	let impulse = collision.normal * impulse_scalar;
    
    let inv_mass_sum = inv_mass1 + inv_mass2;
    let impulse_magnitude = if inv_mass_sum == 0.0 { 
        0.0 
    } else {
        -(1.0 + coeff_of_restitution) * (velocity_along_normal / inv_mass_sum)
    };

    let relative_velocity_tangent = rel_v - velocity_along_normal * collision.normal;
    let tangent_velocity_magnitude = relative_velocity_tangent.length();

    let mut tangent_impulse = glam::Vec3::ZERO;
    if tangent_velocity_magnitude > 0.0 {
        let tangent_direction = relative_velocity_tangent / tangent_velocity_magnitude;
        let friction_magnitude = coeff_of_friction * -impulse_magnitude;
        tangent_impulse = tangent_direction * friction_magnitude;
    }

    Impulse {
		normal: impulse,
		tangent: tangent_impulse,
		r1,
		r2,
	}
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

fn calculate_toi(a: &AABB, b: &AABB, rel_velocity: glam::Vec3, dt: f32) -> Option<f32> {
    let mut t_enter = 0.0;
    let mut t_exit = dt;

    for i in 0..3 {
        let a_min = a.min[i];
        let a_max = a.max[i];
        let b_min = b.min[i];
        let b_max = b.max[i];

        let v = rel_velocity[i];

        if v == 0.0 {
            // Objects are not moving relative to each other on this axis
            if a_max <= b_min || b_max <= a_min {
                // No collision possible if they are not already overlapping
                return None;
            }
            // They are overlapping on this axis; set entry time to zero
            continue;
        }

        let t1 = (b_min - a_max) / v;
        let t2 = (b_max - a_min) / v;

        let (t_axis_enter, t_axis_exit) = if t1 < t2 { (t1, t2) } else { (t2, t1) };

        // Update overall entry and exit times
        if t_axis_enter > t_enter {
            t_enter = t_axis_enter;
        }
        if t_axis_exit < t_exit {
            t_exit = t_axis_exit;
        }

        // Check for separation
        if t_enter > t_exit || t_exit < 0.0 {
            return None;
        }
    }

    if t_enter >= 0.0 && t_enter <= dt {
        Some(t_enter)
    } else if t_exit >= 0.0 && t_enter <= dt {
        // Objects are already overlapping
        Some(0.0)
    } else {
        None
    }
}

fn resolve_collision_old(collision: &Collision, state: &mut State) {
	let node1 = state.nodes.get(&collision.node1).unwrap();
	let node2 = state.nodes.get(&collision.node2).unwrap();
	let impluse = calculate_impulse(node1, node2, &collision, 0.3, 0.2);
	let node1_typ = node1.physics.typ.clone();
	let node2_typ = node2.physics.typ.clone();

	if node1_typ == PhycisObjectType::Dynamic {
		let node1 = state.nodes.get_mut(&collision.node1).unwrap();
		node1.physics.velocity -= impluse.normal / node1.physics.mass;
		node1.translation += collision.correction;
		let angular_impulse = impluse.r1.cross(impluse.normal);
		node1.physics.angular_velocity -= if node1.physics.moment_of_inertia == glam::Vec3::ZERO { glam::Vec3::ZERO } else { angular_impulse / node1.physics.moment_of_inertia };
	}

	if node2_typ == PhycisObjectType::Dynamic {
		let node2 = state.nodes.get_mut(&collision.node2).unwrap();
		node2.physics.velocity += impluse.normal / node2.physics.mass;
		node2.translation -= collision.correction;
		let angular_impulse = impluse.r2.cross(impluse.normal);
		node2.physics.angular_velocity += if node2.physics.moment_of_inertia == glam::Vec3::ZERO { glam::Vec3::ZERO } else { angular_impulse / node2.physics.moment_of_inertia };
	}
}

#[derive(Debug, Default, Clone)]
pub struct PhysicsSystem {
	gravity: glam::Vec3,
	collision_cache: HashSet<(ArenaId<Node>, ArenaId<Node>)>,
	broad_phase_collisions: Vec<Collision>,
}

impl PhysicsSystem {
	pub fn new() -> Self {
		Self {
			gravity: glam::Vec3::new(0.0, -10.0, 0.0),
			collision_cache: HashSet::new(),
			broad_phase_collisions: Vec::new(),
		}
	}
	
	pub fn node_physics_update(&mut self, node: &mut Node, dt: f32) {
		let mass = node.physics.mass;
		let gravity_force = if mass > 0.0 { self.gravity * mass } else { glam::Vec3::ZERO };
		let total_force = node.physics.force + gravity_force;
		let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };
		node.physics.velocity += acceleration * dt;
		node.translation += node.physics.velocity * dt;
		node.physics.acceleration = acceleration;
	
		// Angular dynamics
		let torque = node.physics.torque;
		let moment_of_inertia = node.physics.moment_of_inertia;
		let angular_acceleration = glam::Vec3::new(
			if moment_of_inertia.x > 0.0 { torque.x / moment_of_inertia.x } else { 0.0 },
			if moment_of_inertia.y > 0.0 { torque.y / moment_of_inertia.y } else { 0.0 },
			if moment_of_inertia.z > 0.0 { torque.z / moment_of_inertia.z } else { 0.0 },
		);
		node.physics.angular_velocity += angular_acceleration * dt;
		node.rotation = node.rotation * glam::Quat::from_euler(glam::EulerRot::XYZ, 
			node.physics.angular_velocity.x * dt,
			node.physics.angular_velocity.y * dt, 
			node.physics.angular_velocity.z * dt);
		node.physics.angular_acceleration = angular_acceleration;
	}
	
	fn update_nodes(&mut self, state: &mut State, dt: f32) {
		for (_, node) in &mut state.nodes {
			if node.physics.typ == crate::PhycisObjectType::Dynamic && !node.physics.stationary {
				self.node_physics_update(node, dt);
			}
		}
	}	
	
	fn broad_phase_collisions(&mut self, state: &mut State, grid: &SpatialGrid) {
		self.broad_phase_collisions.clear();
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
						if self.broad_phase_collisions.iter().any(|c: &Collision| 
							(c.node1 == node1_id && c.node2 == node2_id) || 
							(c.node1 == node2_id && c.node2 == node1_id)) {
							continue;
						}
	
						let correction = node1_aabb.get_correction(&node2_aabb);

						self.broad_phase_collisions.push(Collision {
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
	}

	fn resolve_collision(&mut self, state: &mut State, collision: &Collision) {
		let mut impulse = Vec3::ZERO;
		let mut inv_mass1 = 0.0;
		let mut inv_mass2 = 0.0;
		let mut inv_mass_sum = 0.0;
		let mut node1_typ = PhycisObjectType::Static;
		let mut node2_typ = PhycisObjectType::Static;
		{
			let node1 = state.nodes.get(&collision.node1).unwrap();
			let node2 = state.nodes.get(&collision.node2).unwrap();
		
			// Skip if both are static
			if node1.physics.typ == PhycisObjectType::Static && node2.physics.typ == PhycisObjectType::Static {
				return;
			}

			node1_typ = node1.physics.typ.clone();
			node2_typ = node2.physics.typ.clone();
		
			// Calculate relative velocity
			let rel_vel = node2.physics.velocity - node1.physics.velocity;
			let vel_along_normal = rel_vel.dot(collision.normal);
		
			// Do not resolve if velocities are separating
			if vel_along_normal > 0.0 {
				return;
			}
		
			// Calculate restitution and impulse
			let e = 0.3; // Coefficient of restitution
			let j = -(1.0 + e) * vel_along_normal;
			inv_mass1 = if node1.physics.mass > 0.0 { 1.0 / node1.physics.mass } else { 0.0 };
			inv_mass2 = if node2.physics.mass > 0.0 { 1.0 / node2.physics.mass } else { 0.0 };
			inv_mass_sum = inv_mass1 + inv_mass2;
			impulse = collision.normal * (j / inv_mass_sum);
		}
	
		// Apply impulse
		if node1_typ == PhycisObjectType::Dynamic {
			let node1 = state.nodes.get_mut(&collision.node1).unwrap();
			node1.physics.velocity -= impulse * inv_mass1;
		}
		if node2_typ == PhycisObjectType::Dynamic {
			let node2 = state.nodes.get_mut(&collision.node2).unwrap();
			node2.physics.velocity += impulse * inv_mass2;
		}
	
		// **Positional Correction**
		// Apply correction to prevent sinking
		let percent = 0.8; // Penetration percentage to correct
		let slop = 0.01;   // Penetration allowance
		let penetration_depth = collision.correction.length();
		let correction_magnitude = (penetration_depth - slop).max(0.0) / inv_mass_sum * percent;
		let correction = collision.normal * correction_magnitude;
	
		if node1_typ == PhycisObjectType::Dynamic {
			let node1 = state.nodes.get_mut(&collision.node1).unwrap();
			node1.translation -= correction * inv_mass1;
		}
		if node2_typ == PhycisObjectType::Dynamic {
			let node2 = state.nodes.get_mut(&collision.node2).unwrap();
			node2.translation += correction * inv_mass2;
		}
	}


	pub fn physics_update(&mut self, state: &mut State, grid: &mut SpatialGrid, mut dt: f32) {
		let timer = Instant::now();
	    let min_dt = 0.0001; // Minimum time increment to prevent infinite loops
		let max_iterations = 4; // Maximum iterations to prevent infinite loops
		let mut iterations = 0;

		while dt > 0.0 && iterations < max_iterations {
			iterations += 1;

			let mut earliest_toi = dt;
			let mut earliest_collision = None;

			// Detect potential collisions without moving the nodes
			self.broad_phase_collisions(state, grid);

			if self.broad_phase_collisions.is_empty() {
				// No collisions, update nodes for remaining dt and exit
				self.update_nodes(state, dt);
				break;
			}

			let mut there_is_fast_boy = false;
			// Find the earliest collision
			for collision in &self.broad_phase_collisions {
				let node1 = state.nodes.get(&collision.node1).unwrap();
				let node2 = state.nodes.get(&collision.node2).unwrap();

				if node1.physics.typ == PhycisObjectType::Static && node2.physics.typ == PhycisObjectType::Static {
					continue;
				}

				if self.collision_cache.contains(&(collision.node1, collision.node2)) {
					resolve_collision_old(&collision, state);
					continue;
				}

				let rel_velocity = node2.physics.velocity - node1.physics.velocity;

				if rel_velocity.length() < 50.0 {
					resolve_collision_old(&collision, state);
					continue;
				}
				there_is_fast_boy = true;

				let node1_aabb = grid.get_node_rect(collision.node1).unwrap();
				let node2_aabb = grid.get_node_rect(collision.node2).unwrap();

				if let Some(toi) = calculate_toi(&node1_aabb, &node2_aabb, rel_velocity, dt) {
					if toi < earliest_toi {
						earliest_toi = toi;
						earliest_collision = Some(collision.clone());
					}
				}
			}

			self.collision_cache.retain(|(node1, node2)| {
				self.broad_phase_collisions.iter().any(|c: &Collision| 
					(c.node1 == *node1 && c.node2 == *node2) || 
					(c.node1 == *node2 && c.node2 == *node1))
			});

			if !there_is_fast_boy {
				self.update_nodes(state, dt);
				break;
			}
			log::info!("There is a fast boy, need to do toi");

			if let Some(collision) = earliest_collision {
				// Avoid zero TOI causing infinite loops
				let time_step = if earliest_toi < min_dt { min_dt } else { earliest_toi };

				// Update nodes to the time just before collision
				self.update_nodes(state, time_step);
				dt -= time_step;

				// Resolve collision
				self.resolve_collision(state, &collision);
				self.collision_cache.insert((collision.node1, collision.node2));
			} else {
				// No collisions within remaining dt, update nodes and exit
				self.update_nodes(state, dt);
				break;
			}
		}
		let elapsed = timer.elapsed();
		if elapsed > Duration::from_millis(10) {
			log::info!("Physics update took {:?}", elapsed);
		}
	}
}

/*
#[derive(Debug, Default, Clone)]
pub struct PhysicsSystem {
	gravity: glam::Vec3,
}

impl PhysicsSystem {
	pub fn new() -> Self {
		Self {
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
	
	pub fn physics_update(&mut self, state: &mut State, grid: &mut SpatialGrid, dt: f32) {
		self.update_nodes(state, dt);
		let collisions = self.broad_phase_collisions(state, grid);
	
		if collisions.len() > 0 {
			for collision in collisions {
				let node1 = state.nodes.get(&collision.node1).unwrap();
				let node2 = state.nodes.get(&collision.node2).unwrap();

				if node1.physics.typ == PhycisObjectType::Static && node2.physics.typ == PhycisObjectType::Static {
					continue;
				}

				let (normal_impulse, friction_impulse) = calculate_impulse(node1, node2, &collision, 0.3, 0.2);

				if let Some(node1) = state.nodes.get_mut(&collision.node1) {
					if node1.physics.typ == PhycisObjectType::Dynamic {
						node1.physics.velocity -= (normal_impulse + friction_impulse) / node1.physics.mass;
						node1.translation += collision.correction;
					}
				}
				if let Some(node2) = state.nodes.get_mut(&collision.node2) {
					if node2.physics.typ == PhycisObjectType::Dynamic {
						node2.physics.velocity += (normal_impulse + friction_impulse) / node2.physics.mass;
						node2.translation -= collision.correction;
					}
				}
			}
		}
	}
}*/
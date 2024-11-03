use std::collections::HashSet;
use std::time::Duration;
use std::time::Instant;

use glam::Vec3;
use crate::spatial_grid::SpatialGrid;
use crate::state::State;
use crate::ArenaId;
use crate::ContactInfo;
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

/// Represents the impulse resulting from a collision between two rigid bodies.
#[derive(Debug, Default)]
struct Impulse {
    /// The impulse in the direction of the collision normal. This is responsible for
    /// the elastic response (bounce) between the two bodies.
    normal_impulse: glam::Vec3,
    /// The impulse in the tangential direction, representing friction. This impulse
    /// acts to resist sliding between the surfaces of the colliding bodies.
    tangent_impulse: glam::Vec3,
    /// The vector from the center of mass of the first body to the collision point.
    r1: glam::Vec3, 
    /// The vector from the center of mass of the second body to the collision point.
    r2: glam::Vec3,
}

/// Calculates the impulse generated from a collision between two rigid bodies.
///
/// This function calculates both the normal (bouncing) and tangential (frictional) impulses
/// based on the relative velocity at the collision point, the coefficient of restitution, 
/// and the coefficient of friction.
///
/// # Parameters
///
/// - `node1`: The first rigid body involved in the collision, which contains physical properties
///   like mass, velocity, angular velocity, and inertia tensor.
/// - `node2`: The second rigid body involved in the collision.
/// - `collision`: The collision information, including the collision point and normal vector.
/// - `restitution`: The coefficient of restitution, representing the elasticity of the collision,
///   with values from 0.0 (perfectly inelastic) to 1.0 (perfectly elastic).
/// - `coeff_of_friction`: The coefficient of friction, representing the resistance to sliding.
///
/// # Returns
///
/// An `Impulse` struct containing the normal and tangential impulses as well as the vectors `r1`
/// and `r2` from each body's center of mass to the collision point.
///
/// # Examples
///
/// ```rust
/// let impulse = calculate_impulse(&node1, &node2, &collision, 0.8, 0.5);
/// ```
fn calculate_impulse(node1: &Node, node2: &Node, collision: &Collision, restitution: f32, coeff_of_friction: f32) -> Impulse {
	let r1 = collision.point - node1.center_of_mass();
	let r2 = collision.point - node2.center_of_mass();

	let node1_velocity = node1.physics.velocity + node1.physics.angular_velocity.cross(r1);
	let node2_velocity = node2.physics.velocity + node2.physics.angular_velocity.cross(r2);
	let rel_velocity = node2_velocity - node1_velocity;
	let vel_along_normal = rel_velocity.dot(collision.normal);

	if vel_along_normal < 0.0 {
		return Impulse::default();
	}

	let node1_inv_mass = if node1.physics.mass == 0.0 { 0.0 } else { 1.0 / node1.physics.mass };
	let node2_inv_mass = if node2.physics.mass == 0.0 { 0.0 } else { 1.0 / node2.physics.mass };
	let inv_mass_sum = node1_inv_mass + node2_inv_mass;

	if inv_mass_sum == 0.0 {
		return Impulse::default();
	}

	let node1_inertia_tensor = if node1.inertia_tensor() == glam::Mat3::ZERO { glam::Mat3::ZERO } else { node1.inertia_tensor().inverse() };
	let node2_inertia_tensor = if node2.inertia_tensor() == glam::Mat3::ZERO { glam::Mat3::ZERO } else { node2.inertia_tensor().inverse() };
	let term_a = collision.normal.dot(node1_inertia_tensor * (r1.cross(collision.normal)).cross(r1));
	let term_b = collision.normal.dot(node2_inertia_tensor * (r2.cross(collision.normal)).cross(r2));
	let j = -(1.0 + restitution) * vel_along_normal / (inv_mass_sum + term_a + term_b);
	let normal_impulse = j * collision.normal;

	let tangent = (rel_velocity - vel_along_normal * collision.normal).normalize_or_zero();
	let vel_along_tangent = rel_velocity.dot(tangent);
	let jt = -vel_along_tangent / (inv_mass_sum + term_a + term_b);

	let max_friction = coeff_of_friction * j.abs();
	let tangent_impulse = if jt.abs() < max_friction { jt * tangent } else { max_friction * tangent * jt.signum() };

	return Impulse {
		normal_impulse,
		r1,
		r2,
		tangent_impulse,
	};
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

/// Applies a given impulse to this node, updating its linear and angular velocity.
///
/// # Parameters
///
/// - `impulse`: The calculated impulse to apply.
/// - `r`: The vector from the center of mass to the collision point.
fn apply_impulse(impulse: &Impulse, node: &mut Node, r: glam::Vec3) {
	if node.physics.mass > 0.0 {
		node.physics.velocity += impulse.normal_impulse / node.physics.mass;
		node.physics.velocity += impulse.tangent_impulse / node.physics.mass;
	}

	if node.lock_rotation {
		return;
	}

	let inertia_tensor = node.inertia_tensor();
	if inertia_tensor != glam::Mat3::ZERO {
		let inertia_tensor_inverse = inertia_tensor.inverse();
		let angular_impulse_normal = r.cross(impulse.normal_impulse);
		node.physics.angular_velocity += inertia_tensor_inverse * angular_impulse_normal;
		let angular_impulse_tangent = r.cross(impulse.tangent_impulse);
		node.physics.angular_velocity += inertia_tensor_inverse * angular_impulse_tangent;
	}
}

fn resolve_collision(collision: &Collision, state: &mut State) {
	let node1 = state.nodes.get(&collision.node1).unwrap();
	let node2 = state.nodes.get(&collision.node2).unwrap();

	let node1_inv_mass = if node1.physics.mass == 0.0 { 0.0 } else { 1.0 / node1.physics.mass };
	let node2_inv_mass = if node2.physics.mass == 0.0 { 0.0 } else { 1.0 / node2.physics.mass };
	let inv_mass_sum = node1_inv_mass + node2_inv_mass;

	if inv_mass_sum == 0.0 {
		return; // Both objects are static, no correction needed
	}

	let impluse = calculate_impulse(node1, node2, &collision, 0.3, 0.2);
	let node1_typ = node1.physics.typ.clone();
	let node2_typ = node2.physics.typ.clone();

	if node1_typ == PhycisObjectType::Dynamic {
		let node1 = state.nodes.get_mut(&collision.node1).unwrap();
		apply_impulse(&impluse, node1, impluse.r1);
		let node1_correciton_ratio = node1_inv_mass / inv_mass_sum;
		let correction = collision.correction * node1_correciton_ratio;
		node1.translation += correction;
		node1.contacts.push(ContactInfo {
			normal: collision.normal,
			point: collision.point,
			node_id: collision.node2,
		});
	}

	if node2_typ == PhycisObjectType::Dynamic {
		let node2 = state.nodes.get_mut(&collision.node2).unwrap();
		/*let impulse = Impulse {
			normal_impulse: -impluse.normal_impulse,
			tangent_impulse: -impluse.tangent_impulse,
			r1: impluse.r2,
			r2: impluse.r1,
		};*/
		apply_impulse(&impluse, node2, impluse.r2);
		let node2_correciton_ratio = node2_inv_mass / inv_mass_sum;
		let correction = collision.correction * node2_correciton_ratio;
		node2.translation -= correction;
		node2.contacts.push(ContactInfo {
			normal: -collision.normal,
			point: collision.point,
			node_id: collision.node1,
		});
	}
}

#[derive(Debug, Default, Clone)]
pub struct PhysicsSystem {
	gravity: glam::Vec3,
	collision_cache: HashSet<(ArenaId<Node>, ArenaId<Node>)>,
	broad_phase_collisions: Vec<Collision>,
	broad_phase_collision_count: usize,
}

const SLEEP_LINEAR_VELOCITY: f32 = 0.01;

impl PhysicsSystem {
	pub fn new() -> Self {
		Self {
			gravity: glam::Vec3::new(0.0, -10.0, 0.0),
			collision_cache: HashSet::new(),
			broad_phase_collisions: Vec::new(),
			broad_phase_collision_count: 0,
		}
	}
	
	pub fn node_physics_update(&mut self, node: &mut Node, dt: f32) {
		// Linear dynamics
		let mass = node.physics.mass;
		let gravity_force = if mass > 0.0 { self.gravity * mass } else { glam::Vec3::ZERO };
		let mut total_force = node.physics.force + gravity_force;
		if !node.contacts.is_empty() {
			let mut net_contact_normal = glam::Vec3::ZERO;
			for contact in &node.contacts {
				net_contact_normal += contact.normal;
			}
			net_contact_normal = net_contact_normal.normalize_or_zero();
			let gravity_along_normal = self.gravity.project_onto(net_contact_normal);
			total_force -= gravity_along_normal * mass;
		}
		let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };
		node.physics.velocity += acceleration * dt;
		node.translation += node.physics.velocity * dt;
		node.physics.acceleration = acceleration;

		if node.lock_rotation {
			return;
		}
	
		// Angular dynamics
		let torque = node.physics.torque;
		let inertia_tensor = node.inertia_tensor();
	
		// Invert inertia tensor if determinant is large enough to avoid numerical instability
		let inv_inertia_tensor = if inertia_tensor.determinant().abs() > 1e-6 {
			inertia_tensor.inverse()
		} else {
			glam::Mat3::ZERO
		};
		
		// Angular acceleration = inv_inertia_tensor * torque
		let angular_acceleration = inv_inertia_tensor * torque;
		node.physics.angular_velocity += angular_acceleration * dt;

		// Update rotation by integrating the angular velocity (if angular velocity is non-zero)
		if node.physics.angular_velocity.length_squared() > 1e-6 {
			let rotation_delta = glam::Quat::from_axis_angle(
				node.physics.angular_velocity.normalize(),
				node.physics.angular_velocity.length() * dt,
			);
			node.rotation = (rotation_delta * node.rotation).normalize();
		}
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
	
						let correction = node1_aabb.get_correction(&node2_aabb) * 1.0;

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

	pub fn physics_update(&mut self, state: &mut State, grid: &mut SpatialGrid, mut dt: f32) {
		let timer = Instant::now();

		for (_, node) in &mut state.nodes {
			node.contacts.clear();
		}

	    let min_dt = 0.0001; // Minimum time increment to prevent infinite loops
		let max_iterations = 10; // Maximum iterations to prevent infinite loops
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

			if self.broad_phase_collisions.len() != self.broad_phase_collision_count {
				self.broad_phase_collision_count = self.broad_phase_collisions.len();
				log::info!("broad_phase_collision_count: {}", self.broad_phase_collision_count);
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
					resolve_collision(&collision, state);
					continue;
				}

				let rel_velocity = node2.physics.velocity - node1.physics.velocity;

				if rel_velocity.length() < 50.0 {
					resolve_collision(&collision, state);
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
				resolve_collision(&collision, state);
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
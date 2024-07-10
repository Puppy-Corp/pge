use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Instant;

use thunderdome::Index;

use crate::spatial_grid::SpatialGrid;
use crate::Node;
use crate::State;

fn sum_forces(node: &Node) -> glam::Vec3 {
	let mut total_force = glam::Vec3::ZERO;
	for force in &node.forces {
		total_force += force.force;
	}
	total_force
}

pub fn node_physics_update(node: &mut Node, dt: f32) {
	let mass = node.physics.mass;
	let gravity_force = if mass > 0.0 { glam::Vec3::new(0.0, -1.0, 0.0) * mass } else { glam::Vec3::ZERO };
	let total_force = sum_forces(node) + gravity_force;
	let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };
	node.physics.velocity += acceleration * dt;
	node.translation += node.physics.velocity * dt;
	log::debug!("[{}] total force: {:?} acceleration: {:?} velocity: {:?} translation: {:?}", node.id, total_force, acceleration, node.physics.velocity, node.translation);	
	node.physics.acceleration = acceleration;
}

fn update_nodes(state: &mut State, dt: f32) {
	for (_, node) in &mut state.nodes {
		if node.physics.typ == crate::PhycisObjectType::Dynamic && !node.physics.stationary {
			node_physics_update(node, dt);
		}
	}
}

fn broad_phase_collisions(state: &mut State, grid: &SpatialGrid) -> Vec<(Index, Index)> {
	let mut collisions = Vec::new();
	for cell in grid.get_cells() {
		for i in 0..cell.len() {
			let node1_id = cell[i];
			let node1_aabb = match grid.nodes.get(&node1_id) {
				Some(a) => a,
				None => continue
			};
			for j in i+1..cell.len() {
				let node2_id = cell[j];
				let node2_aabb = match grid.nodes.get(&node2_id) {
					Some(a) => a,
					None => continue
				};
				if node1_aabb.intersects(&node2_aabb) {
					collisions.push((cell[i], cell[j]));
				}
			}
		}
	}
	collisions
}

pub struct PhycisTiming {
	pub node_update_time: u32,
	pub broad_phase_time: u32,
	pub narrow_phase_time: u32,
	pub total_time: u32,
}

pub fn physics_update(state: &mut State, grid: &mut SpatialGrid, dt: f32) -> PhycisTiming {
	let timer = Instant::now();
	update_nodes(state, dt);
	let node_update_time = timer.elapsed().as_millis() as u32;
	let collisions = broad_phase_collisions(state, grid);
	let broad_phase_time = timer.elapsed().as_millis() as u32 - node_update_time;

	if collisions.len() > 0 {
		for (node_id1, node_id2) in collisions {
			if let Some(node1) = state.nodes.get_mut(node_id1) {
				if node1.physics.typ == crate::PhycisObjectType::Dynamic {
					node1.physics.stationary = true;
				}
			}

			if let Some(node2) = state.nodes.get_mut(node_id2) {
				if node2.physics.typ == crate::PhycisObjectType::Dynamic {
					node2.physics.stationary = true;
				}
			}
		}
	}
	let narrow_phase_time = timer.elapsed().as_millis() as u32 - broad_phase_time;
	let total_time = timer.elapsed().as_millis() as u32;
	
	PhycisTiming {
		node_update_time,
		broad_phase_time,
		narrow_phase_time,
		total_time,
	}
}
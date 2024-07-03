use std::collections::HashMap;
use std::collections::HashSet;

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
	let gravity_force = if mass > 0.0 { glam::Vec3::new(0.0, -9.81, 0.0) * mass } else { glam::Vec3::ZERO };
	let total_force = sum_forces(node) + gravity_force;
	let acceleration = if mass > 0.0 { total_force / mass } else { glam::Vec3::ZERO };

	node.physics.velocity += acceleration * dt;
	node.translation += node.physics.velocity * dt;
	node.physics.acceleration = acceleration;
}

fn update_nodes(state: &mut State, dt: f32) {
	for (_, node) in &mut state.nodes {
		node_physics_update(node, dt);
	}
}

fn broad_phase_collisions(state: &mut State, grid: &SpatialGrid) -> Vec<(Index, Index)> {
	let mut collisions = Vec::new();
	for cell in grid.get_cells() {
		for i in 0..cell.len() {
			let node1 = &state.nodes[cell[i]];
			for j in i+1..cell.len() {
				let node2 = &state.nodes[cell[j]];
				if node1.aabb.intersects(&node2.aabb) {
					collisions.push((cell[i], cell[j]));
				}
			}
		}
	}
	collisions
}

pub fn physics_update(state: &mut State, grid: &mut SpatialGrid, dt: f32) {
	update_nodes(state, dt);
	let collisions = broad_phase_collisions(state, grid);


}
use core::panic;
use std::collections::HashMap;
use std::collections::HashSet;

use thunderdome::Index;

use crate::debug::ChangePrinter;
use crate::AABB;

#[derive(Debug)]
pub struct SpatialGrid {
	size: f32,
	min: f32,
	max: f32,
	cell_size: f32,
	cell_count: usize,
	cells: Vec<Vec<Index>>,
	pub nodes: HashMap<Index, AABB>,
}

impl SpatialGrid {
	pub fn new(cell_size: f32, cell_count: usize) -> Self {
		let size = cell_size * cell_count as f32;
		log::info!("size: {}", size);
		let half_size = size / 2.0;
		let grid_size = cell_count.pow(3);
		log::info!("grid size: {}", grid_size);
		let cells = vec![Vec::new(); grid_size];
		Self {
			min: -half_size,
			max: half_size,
			size,
			cell_size,
			cell_count,
			cells,
			nodes: HashMap::new(),
		}
	}

	pub fn get_cell_inx(&self, x: usize, y: usize, z: usize) -> usize {
		x + y * self.cell_count + z * self.cell_count.pow(2)
	}

	pub fn add_node(&mut self, node: Index, rect: AABB) {
		log::debug!("add node: {:?} rect: {:?}", node, rect);

		if rect.min.x < self.min {
			log::error!("rect min x: {} is less than grid min: {}", rect.min.x, self.min);
			panic!("rect min x is less than grid min");
		}

		if rect.min.y < self.min {
			log::error!("rect min y: {} is less than grid min: {}", rect.min.y, self.min);
			panic!("rect min y is less than grid min");
		}

		if rect.min.z < self.min {
			log::error!("rect min z: {} is less than grid min: {}", rect.min.z, self.min);
			panic!("rect min z is less than grid min");
		}

		if rect.max.x > self.max {
			log::error!("rect max x: {} is greater than grid max: {}", rect.max.x, self.max);
			panic!("rect max x is greater than grid max");
		}

		if rect.max.y > self.max {
			log::error!("rect max y: {} is greater than grid max: {}", rect.max.y, self.max);
			panic!("rect max y is greater than grid max");
		}

		if rect.max.z > self.max {
			log::error!("rect max z: {} is greater than grid max: {}", rect.max.z, self.max);
			panic!("rect max z is greater than grid max");
		}

		let min_x = ((rect.min.x - self.min) / self.cell_size).floor();
		let max_x = ((rect.max.x - self.min) / self.cell_size).ceil();
		let min_y = ((rect.min.y - self.min) / self.cell_size).floor();
		let max_y = ((rect.max.y - self.min) / self.cell_size).ceil();
		let min_z = ((rect.min.z - self.min) / self.cell_size).floor();
		let max_z = ((rect.max.z - self.min) / self.cell_size).ceil();

		for x in min_x as usize..max_x as usize {
			for y in min_y as usize..max_y as usize {
				for z in min_z as usize..max_z as usize {
					let cell_inx = self.get_cell_inx(x, y, z);
					let cell = match self.cells.get_mut(cell_inx) {
						Some(c) => c,
						None => {
							log::error!("cell x: {} y: {} z: {} not found", x, y, z);
							log::info!("grid size: {}", self.cells.len());
							log::info!("rect: {:?}", rect);
							log::info!("cell count: {}", self.cell_count);
							panic!("cell not found");
						},
					};
					cell.push(node);
				}
			}
		}

		self.nodes.insert(node, rect);
	}

	pub fn get_cell(&self, x: usize, y: usize, z: usize) -> &Vec<Index> {
		let cell_inx = self.get_cell_inx(x, y, z);
		&self.cells[cell_inx]
	}

	pub fn rem_node(&mut self, node: Index) {
		for cell in &mut self.cells {
			cell.retain(|&inx| inx != node);
		}
	}

	pub fn rem_nodes(&mut self, nodes: &HashSet<Index>) {
		for cell in &mut self.cells {
			cell.retain(|&inx| !nodes.contains(&inx));
		}
	}

	pub fn move_node(&mut self, node: Index, rect: AABB) {
        self.rem_node(node);
        self.add_node(node, rect);
	}

	pub fn get_cells(&self) -> &Vec<Vec<Index>> {
		&self.cells
	}
}

#[cfg(test)]
mod tests {
	use thunderdome::Arena;
	use super::*;

	#[test]
	fn test_add_node() {
		let mut arena = Arena::new();
		let id = arena.insert(0);
		let mut grid = SpatialGrid::new(1.0, 4);
		let rect = AABB::new(glam::Vec3::new(-1.0, -1.0, -1.0), glam::Vec3::new(1.0, 1.0, 1.0));
		grid.add_node(id, rect);
		let cell = grid.get_cell(1, 1, 1);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(1, 1, 2);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(1, 2, 1);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(1, 2, 2);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(2, 1, 1);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(2, 1, 2);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(2, 2, 1);
		assert_eq!(cell.contains(&id), true);
		let cell = grid.get_cell(2, 2, 2);
		assert_eq!(cell.contains(&id), true);
	}

	#[test]
	fn test_considers_grid_cell_size() {
		let mut arena = Arena::new();
		let id = arena.insert(0);
		let mut grid = SpatialGrid::new(2.0, 2);
		let rect = AABB::new(glam::Vec3::new(-1.0, -1.0, -2.0), glam::Vec3::new(0.0, 0.0, -1.0));
		grid.add_node(id, rect);
		let cell = grid.get_cell(0, 0, 0);
		assert_eq!(cell.contains(&id), true);
	}
}
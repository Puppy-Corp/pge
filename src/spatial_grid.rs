use std::collections::HashMap;
use std::collections::HashSet;

use thunderdome::Index;

use crate::AABB;

#[derive(Debug)]
pub struct SpatialGrid {
	size: f32,
	min: f32,
	max: f32,
	cell_size: f32,
	cell_count: usize,
	cells: Vec<Vec<Index>>,
}

impl SpatialGrid {
	pub fn new(cell_size: f32, cell_count: usize) -> Self {
		let size = cell_size * cell_count as f32;
		let half_size = size / 2.0;
		let grid_size = cell_count.pow(3);
		let cells = vec![Vec::new(); grid_size];
		Self {
			min: -half_size,
			max: half_size,
			size,
			cell_size,
			cell_count,
			cells,
		}
	}

	pub fn get_cell_inx(&self, x: usize, y: usize, z: usize) -> usize {
		x + y * self.cell_count + z * self.cell_count.pow(2)
	}

	pub fn add_node(&mut self, node: Index, rect: AABB) {
		let min_x = (rect.min.x - self.min / self.cell_size as f32).floor();
		let max_x = (rect.max.x - self.min / self.cell_size as f32).ceil();
		let min_y = (rect.min.y - self.min / self.cell_size as f32).floor();
		let max_y = (rect.max.y - self.min / self.cell_size as f32).ceil();
		let min_z = (rect.min.z - self.min / self.cell_size as f32).floor();
		let max_z = (rect.max.z - self.min / self.cell_size as f32).ceil();

		for x in min_x as usize..max_x as usize {
			for y in min_y as usize..max_y as usize {
				for z in min_z as usize..max_z as usize {
					let cell_inx = self.get_cell_inx(x, y, z);
					let cell = &mut self.cells[cell_inx];
					cell.push(node);
				}
			}
		}
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
		println!("gird: {:?}", grid);
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
}
use crate::types::Mesh;


// pub fn plane(w: f32, h: f32) -> Entity {
// 	Entity {}
// }

// pub fn rect(w: f32, h: f32, d: f32) -> Entity {
// 	Entity {}
// }

pub fn cube(s: f32) -> Mesh {
	let mut m = Mesh::new();
	m.positions = vec![
		[-s, -s, -s],
		[-s, s, -s],
		[s, s, -s],
		[s, -s, -s],
		[-s, -s, s],
		[-s, s, s],
		[s, s, s],
		[s, -s, s],
	];
	m.indices = vec![
		0, 1, 2, 0, 2, 3, // Front
		4, 6, 5, 4, 7, 6, // Back
		0, 4, 1, 1, 4, 5, // Left
		3, 2, 6, 3, 6, 7, // Right
		1, 5, 6, 1, 6, 2, // Top
		4, 0, 3, 4, 3, 7, // Bottom
	];

	m
}
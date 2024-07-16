use crate::types::Mesh;


// pub fn plane(w: f32, h: f32) -> Entity {
// 	Entity {}
// }

// pub fn rect(w: f32, h: f32, d: f32) -> Entity {
// 	Entity {}
// }

pub fn cube(s: f32) -> Mesh {
    let mut m = Mesh::new();
	m.name = Some("Cube".to_string());

    // Define vertex positions for a cube
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

	m.normals = vec![
		[-1.0,-1.0,-1.0],
		[-1.0,1.0,-1.0],
		[1.0,1.0,-1.0],
		[1.0,-1.0,-1.0],
		[-1.0,-1.0,1.0],
		[-1.0,1.0,1.0],
		[1.0,1.0,1.0],
		[1.0,-1.0,1.0],
	];

    m.indices = vec![
        // Front face
        0, 1, 2, 2, 3, 0,
        // Back face
        4, 6, 5, 6, 4, 7,
        // Bottom face
        0, 7, 4, 7, 0, 3,
        // Top face
        1, 5, 6, 6, 2, 1,
        // Left face
        1, 0, 4, 4, 5, 1,
        // Right face
        3, 2, 6, 6, 7, 3,
    ];
	m.tex_coords = vec![
		[0.0, 0.0],
		[0.0, 1.0],
		[1.0, 1.0],
		[1.0, 0.0],
		[0.0, 0.0],
		[0.0, 1.0],
		[1.0, 1.0],
	];

    m
}


pub fn plane(w: f32, h: f32) -> Mesh {
	let mut m = Mesh::new();
	m.name = Some("Plane".to_string());

	m.positions = vec![
		[-w, 0.0, -h],
		[-w, 0.0, h],
		[w, 0.0, h],
		[w, 0.0, -h],
	];

	m.normals = vec![
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
		[0.0, 1.0, 0.0],
	];

	m.indices = vec![
		0, 1, 2, 2, 3, 0,
	];
	m.tex_coords = vec![
		[0.0, 0.0],
		[0.0, 1.0],
		[1.0, 1.0],
		[1.0, 0.0],
	];
	// m.tex_coords = vec![
	// 	[0.0, 0.0],
	// 	[0.0, 0.0],
	// 	[0.0, 0.0],
	// 	[0.0, 0.0],
	// ];
	m
}
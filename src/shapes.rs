use crate::types::Mesh;


// pub fn plane(w: f32, h: f32) -> Entity {
// 	Entity {}
// }

// pub fn rect(w: f32, h: f32, d: f32) -> Entity {
// 	Entity {}
// }

pub fn cube(s: f32) -> Mesh {
    let mut m = Mesh::new();

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

    // Define vertex colors for a cube
    // m.colors = vec![
    //     [1.0, 0.0, 0.0], // Red
    //     [0.0, 1.0, 0.0], // Green
    //     [0.0, 0.0, 1.0], // Blue
    //     [1.0, 1.0, 0.0], // Yellow
    //     [1.0, 0.0, 1.0], // Magenta
    //     [0.0, 1.0, 1.0], // Cyan
    //     [0.5, 0.5, 0.5], // Gray
    //     [1.0, 1.0, 1.0], // White
    // ];

    // Define indices for the cube's faces
    m.indices = vec![
        // Front face
        0, 1, 2, 2, 3, 0,
        // left face
        0, 1, 5, 5, 4, 0,
        // right face
        3, 2, 6, 6, 7, 3,
        // top face
        1, 5, 6, 6, 2, 1,
        // bottom face
        4, 0, 3, 3, 7, 4,
        // back face
        4, 5, 6, 6, 7, 4,
    ];

	// m.indices = vec![
    //     // Front face
    //     0, 1, 2, 2, 3, 0,
    //     // Back face
    //     4, 5, 6, 6, 7, 4,
    //     // Left face
    //     4, 0, 3, 3, 7, 4,
    //     // Right face
    //     1, 5, 6, 6, 2, 1,
    //     // Top face
    //     1, 5, 6, 6, 2, 1,
    //     // Bottom face
    //     4, 0, 1, 1, 5, 4,
    // ];

	m.indices = vec![
		// Front face
		0, 1, 2, 2, 3, 0,
		// Back face
		4, 5, 6, 6, 7, 4,
		// Left face
		0, 4, 7, 7, 3, 0,
		// Right face
		1, 5, 6, 6, 2, 1,
		// Top face
		1, 0, 4, 4, 5, 1,
		// Bottom face
		3, 2, 6, 6, 7, 3,
	];

    m
}
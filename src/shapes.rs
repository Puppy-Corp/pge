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

    m.normals = vec![
        // Front face normals
        [0.0, 0.0, -1.0], // Vertex 0
        [0.0, 0.0, -1.0], // Vertex 1
        [0.0, 0.0, -1.0], // Vertex 2
        [0.0, 0.0, -1.0], // Vertex 3
        // Back face normals
        [0.0, 0.0, 1.0], // Vertex 4
        [0.0, 0.0, 1.0], // Vertex 5
        [0.0, 0.0, 1.0], // Vertex 6
        [0.0, 0.0, 1.0], // Vertex 7
        // Left face normals
        [-1.0, 0.0, 0.0], // Vertex 0
        [-1.0, 0.0, 0.0], // Vertex 4
        [-1.0, 0.0, 0.0], // Vertex 5
        [-1.0, 0.0, 0.0], // Vertex 1
        // Right face normals
        [1.0, 0.0, 0.0], // Vertex 2
        [1.0, 0.0, 0.0], // Vertex 6
        [1.0, 0.0, 0.0], // Vertex 7
        [1.0, 0.0, 0.0], // Vertex 3
        // Top face normals
        [0.0, 1.0, 0.0], // Vertex 1
        [0.0, 1.0, 0.0], // Vertex 5
        [0.0, 1.0, 0.0], // Vertex 6
        [0.0, 1.0, 0.0], // Vertex 2
        // Bottom face normals
        [0.0, -1.0, 0.0], // Vertex 0
        [0.0, -1.0, 0.0], // Vertex 3
        [0.0, -1.0, 0.0], // Vertex 7
        [0.0, -1.0, 0.0], // Vertex 4
    ];

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
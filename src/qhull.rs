use std::f32;
use std::collections::{HashSet, HashMap};

/// Type alias for a 3D point.
type Point = [f32; 3];

/// Structure representing a plane in 3D space.
#[derive(Debug, Clone, PartialEq)]
struct Plane {
    point: Point,  // A point on the plane
    normal: Point, // Normal vector of the plane
}

/// Structure representing a triangular face of the convex hull.
#[derive(Debug, Clone, PartialEq)]
struct Face {
    vertices: [Point; 3], // Three vertices defining the face
    normal: Point,        // Normal vector of the face
}

impl Face {
    /// Creates a new face given three vertices.
    fn new(a: Point, b: Point, c: Point) -> Self {
        let normal = cross(subtract(b, a), subtract(c, a));
        let normal = normalize(normal);
        Face {
            vertices: [a, b, c],
            normal,
        }
    }

    /// Determines if a point is above the face (in the direction of the normal).
    fn is_point_above(&self, point: &Point, epsilon: f32) -> bool {
        let distance = distance_to_plane(*point, self.normal, self.vertices[0]);
        distance > epsilon
    }

	fn distance_to_point(&self, point: Point) -> f32 {
		distance_to_plane(point, self.normal, self.vertices[0])
	}
}

/// Subtracts two points (vectors).
fn subtract(a: Point, b: Point) -> Point {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Computes the cross product of two vectors.
fn cross(a: Point, b: Point) -> Point {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Computes the dot product of two vectors.
fn dot(a: Point, b: Point) -> f32 {
    a[0]*b[0] + a[1]*b[1] + a[2]*b[2]
}

/// Normalizes a vector.
fn normalize(a: Point) -> Point {
    let length = (a[0].powi(2) + a[1].powi(2) + a[2].powi(2)).sqrt();
    if length == 0.0 {
        [0.0, 0.0, 0.0]
    } else {
        [a[0]/length, a[1]/length, a[2]/length]
    }
}

/// Computes the distance from a point to a plane.
fn distance_to_plane(point: Point, normal: Point, plane_point: Point) -> f32 {
    let vec = subtract(point, plane_point);
    dot(vec, normal)
}

/// Finds the extreme points along each axis.
fn find_extreme_points(points: &[Point]) -> (Point, Point, Point, Point, Point, Point) {
    let mut min_x = points[0];
    let mut max_x = points[0];
    let mut min_y = points[0];
    let mut max_y = points[0];
    let mut min_z = points[0];
    let mut max_z = points[0];

    for &p in points.iter() {
        if p[0] < min_x[0] { min_x = p; }
        if p[0] > max_x[0] { max_x = p; }
        if p[1] < min_y[1] { min_y = p; }
        if p[1] > max_y[1] { max_y = p; }
        if p[2] < min_z[2] { min_z = p; }
        if p[2] > max_z[2] { max_z = p; }
    }

    (min_x, max_x, min_y, max_y, min_z, max_z)
}

/// Determines the initial tetrahedron to start the convex hull.
fn initial_tetrahedron(points: &[Point], epsilon: f32) -> Option<[Face; 4]> {
    // Find extreme points
    let (min_x, max_x, min_y, max_y, min_z, max_z) = find_extreme_points(points);

    // Choose the two points with the maximum distance
    let mut initial_points = vec![min_x, max_x, min_y, max_y, min_z, max_z];
    initial_points.sort_by(|a, b| {
        let dist_a = a[0].powi(2) + a[1].powi(2) + a[2].powi(2);
        let dist_b = b[0].powi(2) + b[1].powi(2) + b[2].powi(2);
        dist_b.partial_cmp(&dist_a).unwrap()
    });

    let p1 = initial_points[0];
    let p2 = initial_points[1];

    // Find the point with the maximum distance from the line p1-p2
    let mut max_dist = -1.0;
    let mut p3 = p1;
    for &p in points.iter() {
        let distance = distance_point_to_line(p, p1, p2);
        if distance > max_dist {
            max_dist = distance;
            p3 = p;
        }
    }

    if max_dist < epsilon {
        // Points are colinear
        return None;
    }

    // Find the point with the maximum distance from the plane p1-p2-p3
    let normal = cross(subtract(p2, p1), subtract(p3, p1));
    let normal = normalize(normal);
    let mut max_dist_plane = -1.0;
    let mut p4 = p1;
    for &p in points.iter() {
        let distance = (distance_to_plane(p, normal, p1)).abs();
        if distance > max_dist_plane {
            max_dist_plane = distance;
            p4 = p;
        }
    }

    if max_dist_plane < epsilon {
        // Points are coplanar
        return None;
    }

    // Create initial tetrahedron faces
    let face1 = Face::new(p1, p2, p3);
    let face2 = Face::new(p1, p4, p2);
    let face3 = Face::new(p2, p4, p3);
    let face4 = Face::new(p3, p4, p1);

    Some([face1, face2, face3, face4])
}

/// Computes the distance from a point to a line defined by two points.
fn distance_point_to_line(p: Point, a: Point, b: Point) -> f32 {
    let ab = subtract(b, a);
    let ap = subtract(p, a);
    let cross_prod = cross(ab, ap);
    let ab_length = dot(ab, ab).sqrt();
    if ab_length == 0.0 { 0.0 } else { cross_prod.iter().map(|x| x.powi(2)).sum::<f32>().sqrt() / ab_length }
}

fn find_visible_faces(hull: &Vec<Face>, point: &Point, epsilon: f64) -> Vec<usize> {
    let mut visible_faces = Vec::new();
    for (i, face) in hull.iter().enumerate() {
        if face.is_point_above(point, epsilon) {
            visible_faces.push(i);
        }
    }
    visible_faces
}

fn find_horizon(hull: &Vec<Face>, visible_faces: &Vec<usize>) -> Vec<(Point, Point)> {
    let mut horizon = Vec::new();
    let mut edge_count = std::collections::HashMap::new();

    for &face_idx in visible_faces {
        let face = &hull[face_idx];
        // Assuming vertices are stored in a field named `vertices` in the Face struct
        let vertices = &face.vertices; // Access the vertices
        for i in 0..vertices.len() {
            let edge = (vertices[i], vertices[(i + 1) % vertices.len()]); // Create edges from vertices
            // To ensure edges are comparable regardless of direction, sort the points
            let sorted_edge = if edge.0 < edge.1 {
                (edge.0.clone(), edge.1.clone())
            } else {
                (edge.1.clone(), edge.0.clone())
            };
            *edge_count.entry(sorted_edge).or_insert(0) += 1;
        }
    }

    for &face_idx in visible_faces {
        let face = &hull[face_idx];
        // Assuming vertices are stored in a field named `vertices` in the Face struct
        let vertices = &face.vertices; // Access the vertices
        for i in 0..vertices.len() {
            let edge = (vertices[i], vertices[(i + 1) % vertices.len()]); // Create edges from vertices
            let sorted_edge = if edge.0 < edge.1 {
                (edge.0.clone(), edge.1.clone())
            } else {
                (edge.1.clone(), edge.0.clone())
            };
            if edge_count[&sorted_edge] == 1 {
                // This edge is part of the horizon
                horizon.push(edge);
            }
        }
    }

    horizon
}

pub fn quickhull(points: &[Point], epsilon: f32) -> Vec<[Point; 3]> {
    if points.len() < 4 {
        // All points are part of the convex hull
        let mut hull = Vec::new();
        if points.len() == 3 {
            hull.push([points[0], points[1], points[2]]);
        }
        return hull;
    }

    // Initialize the convex hull with a tetrahedron
    let mut hull = match initial_tetrahedron(points, epsilon) {
        Some(tetra) => tetra.to_vec(),
        None => return Vec::new(), // Coplanar or colinear points
    };

	let mut points_outside = points.iter().filter(|&p| {
		hull.iter().any(|face| face.is_point_above(p, epsilon))
	}).cloned().collect::<Vec<_>>();

	while points_outside.len() > 0 {
		points_outside.sort_by(|a, b| {
			let dist_a = hull[0].distance_to_point(*a);
			let dist_b = hull[0].distance_to_point(*b);
			dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
		});

		let farthest_point = points_outside.pop().unwrap();

		let visible_face_indices = find_visible_faces(&hull, &farthest_point, epsilon as f64);

		if visible_face_indices.is_empty() {
			continue;
		}

		let horizon = find_horizon(&hull, &visible_face_indices);

		for i in visible_face_indices.iter().rev() {
			hull.remove(*i);
		}

		for edge in horizon {
			let new_face = Face::new(edge.0, edge.1, farthest_point);
			hull.push(new_face);
		}

		points_outside.retain(|p| {
			!hull.iter().any(|face| face.is_point_above(p, epsilon))
		});
	}

	hull.iter().map(|face| face.vertices).collect()
}

/// Helper function to check if a face contains a specific edge (order-independent).
fn f_contains_edge(face: &Face, a: Point, b: Point) -> bool {
    let mut count = 0;
    for i in 0..3 {
        if (points_equal(face.vertices[i], a) && points_equal(face.vertices[(i + 1) % 3], b)) ||
           (points_equal(face.vertices[i], b) && points_equal(face.vertices[(i + 1) % 3], a)) {
            count += 1;
        }
    }
    count > 0
}

/// Helper function to compare two points for equality.
fn points_equal(a: Point, b: Point) -> bool {
    (a[0] - b[0]).abs() < 1e-6 &&
    (a[1] - b[1]).abs() < 1e-6 &&
    (a[2] - b[2]).abs() < 1e-6
}

/// Returns a default point (used as initial furthest point).
fn p1() -> Point {
    [0.0, 0.0, 0.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quickhull_simple_tetrahedron() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];

        let hull = quickhull(&points, 1e-6);
        assert_eq!(hull.len(), 4); // A tetrahedron has 4 faces

        // Each face should consist of three of the four points
        for face in hull {
            let mut count = 0;
            for &p in &points {
                if points_equal(p, face[0]) || points_equal(p, face[1]) || points_equal(p, face[2]) {
                    count += 1;
                }
            }
            assert!(count == 3);
        }
    }

    #[test]
    fn test_quickhull_cube() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
        ];

        let hull = quickhull(&points, 1e-6);
        assert_eq!(hull.len(), 12); // A cube has 12 triangular faces

        // Additional checks can be added to verify face orientations and coverage
    }

    #[test]
    fn test_quickhull_coplanar_points() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ];

        let hull = quickhull(&points, 1e-6);
        assert_eq!(hull.len(), 0); // Coplanar points do not form a 3D convex hull
    }

    #[test]
    fn test_quickhull_duplicate_points() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0], // Duplicate
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];

        let hull = quickhull(&points, 1e-6);
        assert_eq!(hull.len(), 4); // Duplicate should not affect the hull
    }

    #[test]
    fn test_quickhull_large_epsilon() {
        let points = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.1, 0.1, 0.1],
            [0.2, 0.2, 0.2],
        ];

        let hull = quickhull(&points, 0.5);
        assert_eq!(hull.len(), 4); // Points inside epsilon are ignored
    }
}
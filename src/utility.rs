
#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

// Function to check if the point `p` is inside the triangle formed by `a`, `b`, and `c`
fn is_point_inside_triangle(p: Point, a: Point, b: Point, c: Point) -> bool {
    let area = |p1: Point, p2: Point, p3: Point| -> f32 {
        (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y)).abs()
    };

    let triangle_area = area(a, b, c);
    let area1 = area(p, a, b);
    let area2 = area(p, b, c);
    let area3 = area(p, c, a);

    (triangle_area - (area1 + area2 + area3)).abs() < f32::EPSILON
}

// Function to check if the turn formed by points `a`, `b`, and `c` is convex
fn is_convex(a: Point, b: Point, c: Point) -> bool {
    (b.y - a.y) * (c.x - b.x) - (b.x - a.x) * (c.y - b.y) < 0.0
}

// Function to perform ear clipping triangulation on a set of 2D points
pub fn ear_clipping_triangulation(points: &[Point]) -> Vec<u32> {
    let mut indices = Vec::new();
    let mut remaining = (0..points.len() as u32).collect::<Vec<_>>();

    while remaining.len() > 3 {
        let mut ear_found = false;

        for i in 0..remaining.len() {
            let prev_idx = if i == 0 { remaining.len() - 1 } else { i - 1 };
            let next_idx = (i + 1) % remaining.len();

            let prev = points[remaining[prev_idx] as usize];
            let curr = points[remaining[i] as usize];
            let next = points[remaining[next_idx] as usize];

            if is_convex(prev, curr, next) {
                let mut is_ear = true;

                for j in 0..points.len() {
                    if j != remaining[prev_idx] as usize && j != remaining[i] as usize && j != remaining[next_idx] as usize {
                        if is_point_inside_triangle(points[j], prev, curr, next) {
                            is_ear = false;
                            break;
                        }
                    }
                }

                if is_ear {
                    indices.push(remaining[prev_idx]);
                    indices.push(remaining[i]);
                    indices.push(remaining[next_idx]);
                    remaining.remove(i);
                    ear_found = true;
                    break;
                }
            }
        }

        if !ear_found {
            panic!("Failed to triangulate polygon");
        }
    }

    indices.push(remaining[0]);
    indices.push(remaining[1]);
    indices.push(remaining[2]);

    indices
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_is_point_inside_triangle() {
		let a = Point { x: 0.0, y: 0.0 };
		let b = Point { x: 4.0, y: 0.0 };
		let c = Point { x: 0.0, y: 4.0 };
		let p = Point { x: 2.0, y: 2.0 };

		assert_eq!(is_point_inside_triangle(p, a, b, c), true);
	}

	#[test]
	fn test_is_convex() {
		let a = Point { x: 0.0, y: 0.0 };
		let b = Point { x: 4.0, y: 0.0 };
		let c = Point { x: 0.0, y: 4.0 };

		assert_eq!(is_convex(a, b, c), true);
	}

	#[test]
	fn test_ear_clipping_triangulation() {
		let points = vec![
			Point { x: 0.0, y: 0.0 },
			Point { x: 4.0, y: 0.0 },
			Point { x: 4.0, y: 4.0 },
			Point { x: 0.0, y: 4.0 },
		];

		let indices = ear_clipping_triangulation(&points);
		println!("{:?}", indices);

		assert_eq!(indices, vec![3, 0, 1, 1, 2, 3]);
	}
}
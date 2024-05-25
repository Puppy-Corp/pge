
pub struct Point3D {}

pub struct Vec3 {
	pub x: f32,
	pub y: f32,
	pub z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3 {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

	pub fn sub(&self, other: &Vec3) -> Vec3 {
		Vec3 {
			x: self.x - other.x,
			y: self.y - other.y,
			z: self.z - other.z
		}
	}

    fn normalize(&self) -> Vec3 {
        let mag = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        Vec3 {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl std::ops::Sub for &Vec3 {
    type Output = Vec3;

    fn sub(self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Vec3 {
    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
}

#[derive(Debug, Clone)]
pub struct Mat4 {
	data: [[f32; 4]; 4]
}

impl Mat4 {
	fn identity() -> Mat4 {
		Mat4 {
			data: [
				[1.0, 0.0, 0.0, 0.0],
				[0.0, 1.0, 0.0, 0.0],
				[0.0, 0.0, 1.0, 0.0],
				[0.0, 0.0, 0.0, 1.0]
			]
		}
	}

    fn translation(x: f32, y: f32, z: f32) -> Self {
        let mut mat = Mat4::identity();
        mat.data[0][3] = x;
        mat.data[1][3] = y;
        mat.data[2][3] = z;
        mat
    }

	fn rotation_x(angle: f32) -> Self {
        let mut mat = Mat4::identity();
        let cos = angle.cos();
        let sin = angle.sin();
        mat.data[1][1] = cos;
        mat.data[1][2] = -sin;
        mat.data[2][1] = sin;
        mat.data[2][2] = cos;
        mat
    }

    fn rotation_y(angle: f32) -> Self {
        let mut mat = Mat4::identity();
        let cos = angle.cos();
        let sin = angle.sin();
        mat.data[0][0] = cos;
        mat.data[0][2] = sin;
        mat.data[2][0] = -sin;
        mat.data[2][2] = cos;
        mat
    }

    fn rotation_z(angle: f32) -> Self {
        let mut mat = Mat4::identity();
        let cos = angle.cos();
        let sin = angle.sin();
        mat.data[0][0] = cos;
        mat.data[0][1] = -sin;
        mat.data[1][0] = sin;
        mat.data[1][1] = cos;
        mat
    }

    fn perspective(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> Self {
        let tan_half_fovy = (fovy / 2.0).tan();
        let mut mat = Mat4::identity();
        mat.data[0][0] = 1.0 / (aspect * tan_half_fovy);
        mat.data[1][1] = 1.0 / tan_half_fovy;
        mat.data[2][2] = -(zfar + znear) / (zfar - znear);
        mat.data[2][3] = -1.0;
        mat.data[3][2] = -(2.0 * zfar * znear) / (zfar - znear);
        mat.data[3][3] = 0.0;
        mat
    }

    fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Self {
        let f = center.sub(&eye).normalize();
        let s = f.cross(&up.normalize()).normalize();
        let u = s.cross(&f);

        let mut mat = Mat4::identity();
        mat.data[0][0] = s.x;
        mat.data[0][1] = u.x;
        mat.data[0][2] = -f.x;
        mat.data[1][0] = s.y;
        mat.data[1][1] = u.y;
        mat.data[1][2] = -f.y;
        mat.data[2][0] = s.z;
        mat.data[2][1] = u.z;
        mat.data[2][2] = -f.z;
        mat.data[0][3] = -s.dot(&eye);
        mat.data[1][3] = -u.dot(&eye);
        mat.data[2][3] = f.dot(&eye);
        mat
    }

    fn multiply(&self, other: &Mat4) -> Self {
        let mut result = Mat4::identity();
        for i in 0..4 {
            for j in 0..4 {
                result.data[i][j] = 0.0;
                for k in 0..4 {
                    result.data[i][j] += self.data[i][k] * other.data[k][j];
                }
            }
        }
        result
    }

    fn transform_point(&self, point: Vec3) -> Vec3 {
        let x = self.data[0][0] * point.x + self.data[0][1] * point.y + self.data[0][2] * point.z + self.data[0][3];
        let y = self.data[1][0] * point.x + self.data[1][1] * point.y + self.data[1][2] * point.z + self.data[1][3];
        let z = self.data[2][0] * point.x + self.data[2][1] * point.y + self.data[2][2] * point.z + self.data[2][3];
        let w = self.data[3][0] * point.x + self.data[3][1] * point.y + self.data[3][2] * point.z + self.data[3][3];
        Vec3 {
            x: x / w,
            y: y / w,
            z: z / w,
        }
    }
}

struct Vec3 {}

fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
	Vec3 { }
}

fn uniform() -> Uniform {
	Uniform {}
}

struct Uniform {

}

struct Vec4 {

}

struct Program {

}

pub struct Buffer {

}

impl Buffer {
	pub fn new() -> Self {
		Buffer {}
	}

	pub fn write(&self, data: Vec<u8>) {

	}
}

fn buffer() -> Buffer {
	Buffer::new()
}

fn input<T>() -> Input<T> {
	Input::<T> {
		_t: std::marker::PhantomData
	}
}

struct Input<T> {
	_t: std::marker::PhantomData<T>
}

struct ClipPosition {
	
}

fn normalize(v: Vec3) -> Vec3 {
	vec3(0.0,0.0,0.0)
}

struct LazyOP {

}

fn max() {

}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fragment() {
		struct VertexOutput {
			clip_position: Vec4,
			world_position: Vec3,
		}
		let input = input::<VertexOutput>();
		let point_lights = buffer();
		let light_count = uniform();
		let mut light_color = vec3(1.0, 1.0, 1.0);
		let mut result = vec3(0.0, 0.0, 0.0);
		for i in 0..2 {
			// let point_light = point_lights[i];
			// let light_position = point_light.position;
			// let light_direction = normalize(light_position - world_position);
			// let diffuse_stregth = max(dot(normal, light_direction), 0.0);
			// let diffuse_color = light_color * diffuse_stregth;
			// result += diffuse_color;
		}
	}
}

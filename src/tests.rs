
#[cfg(test)]
mod tests {
    use crate::cube;
    use crate::engine_state::EngineState;
    use crate::Mesh;
    use crate::Primitive;
    use crate::State;

	#[test]
	fn it_works() {
		let light_position = glam::Vec3::new(-2.0, 1.5, 0.0);
		let world_position = glam::Vec3::new(-2.0, 0.0, 0.0);
		let light_direction = (light_position - world_position).normalize();
		let normal = glam::Vec3::new(0.0, 1.0, 0.0);
		println!("light direction: {:?}", light_direction);

		let diffuce = normal.dot(light_direction).max(0.0);
		println!("diffuse: {:?}", diffuce);

		let light_color = glam::Vec3::new(1.0, 1.0, 1.0);

		let diffuse_color = light_color * diffuce;
		println!("diffuse color: {:?}", diffuse_color);

		let incolor = glam::Vec3::new(1.0, 0.0, 0.0);
		let result = diffuse_color * incolor;
		println!("result: {:?}", result);
	}

	#[test]
	pub fn test_create_geometry() {
		let mesh = cube(3.0);
		let mut engine_state = EngineState::new();
		engine_state.state.meshes.insert(mesh);
		engine_state.update(0.5);

		println!("{:?}", engine_state.triangles);
	}
}
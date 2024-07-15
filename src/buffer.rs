use crate::wgpu_types::BufferRecipe;


pub struct Buffer {
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub size: u64,
}

impl Buffer {
	pub fn new<T: BufferRecipe>(device: &wgpu::Device, size: u64) -> Buffer {
		let layout = T::create_bind_group_layout(&device);
		let buffer = T::create_buffer(&device, size);
		let bind_group = T::create_bind_group(&device, &buffer, &layout);

		Buffer {
			buffer,
			bind_group,
			size: size,
		}
	}
}
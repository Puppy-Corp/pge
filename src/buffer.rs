use std::sync::Arc;

use crate::wgpu_types::BufferRecipe;


pub struct Bindablebuffer {
	pub buffer: wgpu::Buffer,
	pub bind_group: wgpu::BindGroup,
	pub size: u64,
}

impl Bindablebuffer {
	pub fn new<T: BufferRecipe>(device: &wgpu::Device, size: u64) -> Bindablebuffer {
		let layout = T::create_bind_group_layout(&device);
		let buffer = T::create_buffer(&device, size);
		let bind_group = T::create_bind_group(&device, &buffer, &layout);

		Bindablebuffer {
			buffer,
			bind_group,
			size: size,
		}
	}
}

// pub struct Buffer {
// 	device: Arc<wgpu::Device>,
// 	queue: Arc<wgpu::Queue>,
// 	buffer: wgpu::Buffer,
// 	size: u64,
// }

// impl Buffer {
// 	pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, size: u64) -> Buffer {
// 		Buffer {
// 			device: device,
// 			queue: queue,
// 			buffer: device.create_buffer(&wgpu::BufferDescriptor {
// 				label: None,
// 				size: size,
// 				usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
// 				mapped_at_creation: false,
// 			}),
// 			size: size,
// 		}
// 	}
// }
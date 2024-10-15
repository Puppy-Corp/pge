use std::path::Path;
use wgpu::Origin3d;
use winit::event_loop::EventLoopProxy;

use crate::internal_types::EngineEvent;
use crate::wgpu_types::TextureBuffer;
use crate::ArenaId;
use crate::Texture;

pub fn load_image<P: AsRef<Path>>(proxy: EventLoopProxy<EngineEvent>, path: P, texture_id: ArenaId<Texture>) {
    let path = path.as_ref().to_owned();
    log::info!("load_image: {:?}", path);

    std::thread::spawn(move || {
        match image::open(&path) {
            Ok(img) => {
                let img: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = img.to_rgba8();
                let (width, height) = img.dimensions();
                let data = img.into_raw();
                log::info!("Image loaded: {}x{}", width, height);

                match proxy.send_event(EngineEvent::ImageLoaded {
                    texture_id,
                    width,
                    height,
                    data,
                }) {
                    Ok(_) => log::info!("Event sent successfully"),
                    Err(e) => log::error!("Failed to send event: {:?}", e),
                }
            }
            Err(e) => {
                log::error!("Failed to load image: {:?}", e);
            }
        }
    });
}

pub fn create_texture_with_uniform_color(
	device: &wgpu::Device,
	queue: &wgpu::Queue,
) -> wgpu::BindGroup {
    let size = wgpu::Extent3d {
        width: 1,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Uniform Color Texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
		view_formats: Default::default(),
    });
	let data: [u8; 4] = [255, 100, 200, 255]; // red
	queue.write_texture(
		wgpu::ImageCopyTexture {
			texture: &texture,
			mip_level: 0,
			origin: Origin3d::ZERO,
			aspect: wgpu::TextureAspect::All,
		},
		&data,
		wgpu::ImageDataLayout {
			offset: 0,
			bytes_per_row: Some(4),
			rows_per_image: None,
		},
		size
	);

	let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
	let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
		address_mode_u: wgpu::AddressMode::Repeat,
		address_mode_v: wgpu::AddressMode::Repeat,
		address_mode_w: wgpu::AddressMode::Repeat,
		mag_filter: wgpu::FilterMode::Linear,
		min_filter: wgpu::FilterMode::Linear,
		mipmap_filter: wgpu::FilterMode::Nearest,
		..Default::default()
	});

	let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
		layout: &TextureBuffer::create_bind_group_layout(device),
		entries: &[
			wgpu::BindGroupEntry {
				binding: 0,
				resource: wgpu::BindingResource::TextureView(&texture_view),
			},
			wgpu::BindGroupEntry {
				binding: 1,
				resource: wgpu::BindingResource::Sampler(&sampler),
			},
		],
		label: Some("texture_bind_group"),
	});

	texture_bind_group
}
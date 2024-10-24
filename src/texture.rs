use std::path::Path;
use winit::event_loop::EventLoopProxy;
use crate::internal_types::EngineEvent;
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
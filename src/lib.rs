pub mod engine;
pub mod types;
pub mod shapes;
pub mod gui;
mod buffer;
mod internal_types;
mod tests;
mod compositor;
mod utility;
//mod renderer;
mod physics;
mod spatial_grid;
//mod engine_state;
mod debug;
//mod texture;
mod gltf;
mod arena;
mod log;
mod hardware;
#[cfg(feature = "wgpu_winit")]
mod wgpu;
pub mod text;
pub use types::*;
pub use shapes::*;
pub use gui::*;
pub use arena::*;
pub use glam::*;
pub use log::*;
pub use gltf::load_gltf;


pub fn run<T>(app: T) -> anyhow::Result<()>
where
    T: App,
{
    todo!()
}
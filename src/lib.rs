pub mod engine;
pub mod types;
pub mod shapes;
pub mod gui;
mod buffer;
mod internal_types;
mod tests;
mod compositor;
mod physics;
mod spatial_grid;
//mod engine_state;
mod debug;
//mod texture;
mod gltf;
mod arena;
mod log;
mod hardware;
mod state;
#[cfg(feature = "wgpu_winit")]
mod wgpu;
pub mod utility;
pub mod text;
pub use types::*;
pub use shapes::*;
pub use gui::*;
pub use arena::*;
pub use glam::*;
pub use log::*;
pub use state::*;
pub use gltf::load_gltf;

#[cfg(not(feature = "wgpu_winit"))]
pub fn run<T>(app: T) -> anyhow::Result<()>
where
    T: App,
{
    todo!()
}

#[cfg(feature = "wgpu_winit")]
pub use crate::wgpu::run;
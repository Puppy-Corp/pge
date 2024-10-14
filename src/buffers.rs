use std::collections::HashMap;
use std::ops::Range;

use crate::compositor::Compositor;
use crate::engine_state::DrawCall;
use crate::engine_state::UIRenderArgs;
use crate::engine_state::View;
use crate::state::State;
use crate::ArenaId;
use crate::PrimitiveTopology;
use crate::Projection;
use crate::Scene;
use crate::Element;
use crate::Window;
use glam::*;

#[derive(Debug, Clone, Default)]
pub struct Triangles {
	pub vertices: Vec<f32>,
	pub normals: Vec<f32>,
	pub tex_coords: Vec<f32>,
	pub indices: Vec<u32>,
}

pub struct EngineState {
	pub triangles: Triangles,
	pub scene_instance_buffers: HashMap<ArenaId<Scene>, Vec<u8>>,
	pub scene_draw_calls: HashMap<ArenaId<Scene>, Vec<DrawCall>>,
	pub compisotrs: HashMap<ArenaId<Element>, Compositor>,
	pub render_args: HashMap<ArenaId<Element>, UIRenderArgs>,
}
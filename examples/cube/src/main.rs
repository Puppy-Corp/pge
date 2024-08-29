use std::time::Duration;

use glam::Quat;
use pge::*;
use text::FontMesh;
use tokio::time::sleep;

struct Cube {
	editor: Option<NodeEditor>
}

impl Cube {
	pub fn new() -> Self {
		Self {
			editor: None
		}
	}
}

impl pge::App for Cube {
	fn on_create(&mut self, state: &mut State) {
		let editor = NodeEditor::new(state);
		self.editor = Some(editor);

		let cube = cube(0.1);
		let mesh_id = state.meshes.insert(cube);

		let scene = Scene::new();
		let scene_id = state.scenes.insert(scene);

		let mut node = Node::new();
		node.parent = NodeParent::Scene(scene_id);
		node.mesh = Some(mesh_id);
		let node_id = state.nodes.insert(node);
	}

	fn on_process(&mut self, state: &mut State, delta: f32) {
		if let Some(editor) = &mut self.editor {
			editor.on_process(state, delta);
		}
	}
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	init_logging();

    let app = Cube::new();
	Ok(pge::run(app).await?)
}
 
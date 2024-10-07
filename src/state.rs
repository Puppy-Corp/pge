use std::path::Path;

use crate::load_gltf;
use crate::types::*;
use crate::Arena;
use crate::ArenaId;
use crate::UIElement;
use crate::Window;

#[derive(Debug, Clone, Default)]
pub struct State {
	pub scenes: Arena<Scene>,
	pub meshes: Arena<Mesh>,
	pub nodes: Arena<Node>,
	pub cameras: Arena<Camera>,
	pub windows: Arena<Window>,
	pub ui_elements: Arena<UIElement>,
	pub ui_nodes: Arena<UINode>,
	pub point_lights: Arena<PointLight>,
	pub textures: Arena<Texture>,
	pub raycasts: Arena<RayCast>,
	pub models: Arena<Model3D>,
	pub animations: Arena<Animation>,
	pub materials: Arena<Material>,
}

impl State {
	pub fn load_3d_model<P: AsRef<Path> + Clone>(&mut self, path: P) -> ArenaId<Model3D> {
		let model = load_gltf(path, self);
		self.models.insert(model)
	}

	/// Deep clones node and it's children
	pub fn clone_node(&mut self, node_id: ArenaId<Node>) -> ArenaId<Node> {
		let node = self.nodes.get(&node_id).expect("Node not found");
		let mut new_node = node.clone();
		new_node.parent = NodeParent::Orphan;
		let new_node_id = self.nodes.insert(new_node);
		let mut stack = vec![(node_id, new_node_id)];
		while let Some((orig_id, new_parent_id)) = stack.pop() {
			let children: Vec<_> = self.nodes.iter()
				.filter_map(|(id, n)| if n.parent == NodeParent::Node(orig_id) { Some(id) } else { None })
				.collect();
			for child_id in children {
				let child = self.nodes.get(&child_id).expect("Child node not found");
				let mut new_child = child.clone();
				new_child.parent = NodeParent::Node(new_parent_id);
				let new_child_id = self.nodes.insert(new_child);
				stack.push((child_id, new_child_id));
			}
		}
	
		new_node_id
	}

	pub fn mem_size(&self) -> usize {
		self.scenes.mem_size() + self.meshes.mem_size() + self.nodes.mem_size() + self.cameras.mem_size() + self.windows.mem_size() + self.ui_elements.mem_size() + self.point_lights.mem_size() + self.textures.mem_size() + self.raycasts.mem_size()
	}

	pub fn print_state(&self) {
		log::info!("scene count: {:?}", self.scenes.len());
		log::info!("mesh count: {:?}", self.meshes.len());
		log::info!("node count: {:?}", self.nodes.len());
		log::info!("camera count: {:?}", self.cameras.len());
		log::info!("window count: {:?}", self.windows.len());
		log::info!("gui count: {:?}", self.ui_elements.len());
		log::info!("point light count: {:?}", self.point_lights.len());
		log::info!("texture count: {:?}", self.textures.len());
		log::info!("raycast count: {:?}", self.raycasts.len());
	}

	pub fn get_mesh_nodes(&self, mesh: ArenaId<Mesh>) -> Vec<ArenaId<Node>> {
		self.nodes.iter()
			.filter_map(|(id, node)| if node.mesh == Some(mesh) { Some(id) } else { None })
			.collect()
	}

	/// Gets node's final transformation matrix after all parent transformations
	pub fn get_node_model(&self, node_id: ArenaId<Node>) -> glam::Mat4 {
		// let node = self.nodes.get(&node_id).expect("Node not found");
		// let mut model = glam::Mat4::IDENTITY;
		// let mut current_node = node_id;
		// while current_node != ArenaId::new(0) {
		// 	let current_node_data = self.nodes.get(&current_node).expect("Node not found");
		// 	model = current_node_data.transform * model;
		// 	current_node = match current_node_data.parent {
		// 		NodeParent::Node(parent) => parent,
		// 		NodeParent::Orphan => ArenaId::new(0),
		// 	};
		// }
		// model
		todo!()
	}

	pub fn get_scene_id(&self, node_id: ArenaId<Node>) -> ArenaId<Scene> {
		todo!()
	}
}

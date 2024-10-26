use std::collections::HashMap;
use std::path::Path;
use crate::load_gltf;
use crate::arena::*;
use crate::types::*;
use crate::GUIElement;
use crate::Window;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub scenes: Arena<Scene>,
    pub meshes: Arena<Mesh>,
    pub nodes: Arena<Node>,
    pub cameras: Arena<Camera>,
    pub windows: Arena<Window>,
    pub guis: Arena<GUIElement>,
    pub point_lights: Arena<PointLight>,
    pub textures: Arena<Texture>,
    pub raycasts: Arena<RayCast>,
    pub models: Arena<Model3D>,
    pub animations: Arena<Animation>,
    pub materials: Arena<Material>,
    pub keyboard: Option<Keyboard>,
	transformation_cache: HashMap<ArenaId<Node>, glam::Mat4>,
	scene_id_cache: HashMap<ArenaId<Node>, ArenaId<Scene>>,
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
        let mut stack = vec![(node_id, new_node_id.clone())];
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
        self.scenes.mem_size() + self.meshes.mem_size() + self.nodes.mem_size() + self.cameras.mem_size() + self.windows.mem_size() + self.guis.mem_size() + self.point_lights.mem_size() + self.textures.mem_size() + self.raycasts.mem_size()
    }

    pub fn print_state(&self) {
        log::info!("scene count: {:?}", self.scenes.len());
        log::info!("mesh count: {:?}", self.meshes.len());
        log::info!("node count: {:?}", self.nodes.len());
        log::info!("camera count: {:?}", self.cameras.len());
        log::info!("window count: {:?}", self.windows.len());
        log::info!("gui count: {:?}", self.guis.len());
        log::info!("point light count: {:?}", self.point_lights.len());
        log::info!("texture count: {:?}", self.textures.len());
        log::info!("raycast count: {:?}", self.raycasts.len());
    }


	pub fn get_mesh_nodes(&self, mesh: ArenaId<Mesh>) -> Vec<ArenaId<Node>> {
		self.nodes.iter()
			.filter_map(|(id, node)| if node.mesh == Some(mesh) { Some(id) } else { None })
			.collect()
	}

	/// Prepares the transformation and scene ID caches
	pub fn prepare_cache(&mut self) {
		self.transformation_cache.clear();
		self.scene_id_cache.clear();

		let node_ids: Vec<ArenaId<Node>> = self.nodes.iter().map(|(id, _)| id).collect();

		for node_id in node_ids {
			// Compute and cache the transformation matrix
			let model_matrix = self.compute_model_matrix(node_id);
			self.transformation_cache.insert(node_id, model_matrix);

			// Compute and cache the scene ID
			let scene_id = self.compute_scene_id(node_id);
			if let Some(scene_id) = scene_id {
				self.scene_id_cache.insert(node_id, scene_id);
			}
		}
	}

	/// Helper method to compute the transformation matrix for a node
	fn compute_model_matrix(&mut self, node_id: ArenaId<Node>) -> glam::Mat4 {
		let mut stack = Vec::new();
		let mut current_id = node_id;

		// Traverse up the hierarchy to collect nodes that need their transforms computed
		while let Some(node) = self.nodes.get(&current_id) {
			if self.transformation_cache.contains_key(&current_id) {
				break;
			}
			stack.push(current_id);
			match node.parent {
				NodeParent::Orphan | NodeParent::Scene(_) => break,
				NodeParent::Node(parent_id) => current_id = parent_id,
			}
		}

		// Initialize the final transformation matrix
		let mut final_transform = if let Some(matrix) = self.transformation_cache.get(&current_id) {
			*matrix
		} else {
			let node = self.nodes.get(&current_id).expect("Node not found");
			node.model_matrix()
		};

		// Compute the transformation matrices from the top of the stack down
		while let Some(n_id) = stack.pop() {
			let node = self.nodes.get(&n_id).expect("Node not found");
			final_transform = final_transform * node.model_matrix();
			self.transformation_cache.insert(n_id, final_transform);
		}

		final_transform
	}

	/// Helper method to compute the scene ID for a node
	fn compute_scene_id(&self, node_id: ArenaId<Node>) -> Option<ArenaId<Scene>> {
		let mut current_id = node_id;
		let mut scene_id = None;

		while let Some(node) = self.nodes.get(&current_id) {
			match node.parent {
				NodeParent::Scene(s_id) => {
					scene_id = Some(s_id);
					break;
				},
				NodeParent::Node(parent_id) => current_id = parent_id,
				NodeParent::Orphan => break,
			}
		}

		if let Some(s_id) = scene_id {
			Some(s_id)
		} else {
			None
		}
	}

	/// Gets node's final transformation matrix after all parent transformations
	pub fn get_node_final_transformation(&self, node_id: ArenaId<Node>) -> glam::Mat4 {
		self.transformation_cache.get(&node_id).cloned().expect("Transformation cache not prepared for this node")
	}

	pub fn get_node_scene(&self, node_id: ArenaId<Node>) -> Option<ArenaId<Scene>> {
		self.scene_id_cache.get(&node_id).cloned()
	}

	pub fn clear_cache(&mut self) {
		self.transformation_cache.clear();
		self.scene_id_cache.clear();
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    #[test]
    fn test_load_3d_model() {
        let mut state = State::default();
        let model_id = state.load_3d_model("test_model.gltf");
        assert!(state.models.contains(&model_id));
    }

    #[test]
    fn test_clone_node() {
        let mut state = State::default();
        let original_node = Node::new();
        let original_id = state.nodes.insert(original_node);
        
        let cloned_id = state.clone_node(original_id);
        
        assert_ne!(original_id, cloned_id);
        assert!(state.nodes.contains(&cloned_id));
    }

    #[test]
    fn test_mem_size() {
        let state = State::default();
        assert!(state.mem_size() > 0);
    }

    #[test]
    fn test_get_mesh_nodes() {
        let mut state = State::default();
        let mesh_id = state.meshes.insert(Mesh::default());
        let node1 = Node { mesh: Some(mesh_id), ..Default::default() };
        let node2 = Node { mesh: Some(mesh_id), ..Default::default() };
        let node3 = Node::default();
        
        let id1 = state.nodes.insert(node1);
        let id2 = state.nodes.insert(node2);
        state.nodes.insert(node3);
        
        let mesh_nodes = state.get_mesh_nodes(mesh_id);
        assert_eq!(mesh_nodes.len(), 2);
        assert!(mesh_nodes.contains(&id1));
        assert!(mesh_nodes.contains(&id2));
    }

    #[test]
    fn test_prepare_cache() {
        let mut state = State::default();
        let scene_id = state.scenes.insert(Scene::default());
        let node1 = Node { parent: NodeParent::Scene(scene_id), ..Default::default() };
        let node1_id = state.nodes.insert(node1);
        let node2 = Node { parent: NodeParent::Node(node1_id), ..Default::default() };
        let node2_id = state.nodes.insert(node2);
        
        state.prepare_cache();
        
        assert!(state.transformation_cache.contains_key(&node1_id));
        assert!(state.transformation_cache.contains_key(&node2_id));
        assert!(state.scene_id_cache.contains_key(&node1_id));
        assert!(state.scene_id_cache.contains_key(&node2_id));
    }

    #[test]
    fn test_get_node_transformation() {
        let mut state = State::default();
        let node = Node { translation: Vec3::new(1.0, 2.0, 3.0), ..Default::default() };
        let node_id = state.nodes.insert(node);
        state.prepare_cache();
        
        let transform = state.get_node_final_transformation(node_id);
        assert_eq!(transform.to_scale_rotation_translation().2, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_get_node_scene() {
        let mut state = State::default();
        let scene_id = state.scenes.insert(Scene::default());
        let node = Node { parent: NodeParent::Scene(scene_id), ..Default::default() };
        let node_id = state.nodes.insert(node);
        state.prepare_cache();
        
        let retrieved_scene_id = state.get_node_scene(node_id);
        assert_eq!(retrieved_scene_id, Some(scene_id));
    }

    #[test]
    fn test_clear_cache() {
        let mut state = State::default();
        let node_id = state.nodes.insert(Node::default());
        state.prepare_cache();
        
        assert!(!state.transformation_cache.is_empty());
        assert!(state.scene_id_cache.is_empty());
        
        state.clear_cache();
        
        assert!(state.transformation_cache.is_empty());
        assert!(state.scene_id_cache.is_empty());
    }
}

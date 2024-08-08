use std::path::Path;

use gltf::buffer::Data;

use crate::Mesh;
use crate::Node;
use crate::NodeParent;
use crate::Primitive;
use crate::PrimitiveTopology;
use crate::Scene;
use crate::State;


pub fn load_node(n: &gltf::Node, buffers: &[Data], state: &mut State, parent: NodeParent) {
	let mut node = Node {
		name: Some(n.name().unwrap_or_default().to_string()),
		parent,
		..Default::default()
	};

	match n.mesh() {
		Some(gltf_mesh) => {
			log::info!("Mesh: {}", gltf_mesh.name().unwrap_or("Unnamed"));
			let mut mesh = Mesh::new();
			for p in gltf_mesh.primitives() {
				let mut primitive = Primitive::new(PrimitiveTopology::from_mode(p.mode()));

				log::info!("- Primitive #{}", p.index());

				// for (semantic, acc) in p.attributes() {
				// 	println!("Semantic: {:?}", semantic);
				// }

				let reader = p.reader(|buffer| {
					let buffer_data = &buffers[buffer.index()];
					Some(&buffer_data.0[..])
				});
				if let Some(iter) = reader.read_positions() {
					for vertex_position in iter {
						primitive.vertices.push([vertex_position[0], vertex_position[1], vertex_position[2]]);
					}
				}

				reader.read_indices().map(|iter| {
					for index in iter.into_u32() {
						// println!("{:?}", index);
						primitive.indices.push(index as u16);
					}
				});

				reader.read_normals().map(|iter| {
					for normal in iter {
						// println!("{:?}", normal);
						primitive.normals.push([normal[0], normal[1], normal[2]]);
					}
				});

				// reader.read_tex_coords()

				mesh.primitives.push(primitive);
			}

			let mesh_id = state.meshes.insert(mesh);
			node.mesh = Some(mesh_id);
		},
		None => {}
	}
	
	let node_id = state.nodes.insert(node);

	for child in n.children() {
		load_node(&child, buffers, state, NodeParent::Node(node_id));
	}
}

pub fn load_scene(s: &gltf::Scene, buffers: &[Data], state: &mut State) {
	let scene = Scene {
		name: Some(s.name().unwrap_or_default().to_string()),
		..Default::default()
	};

	let scene_id = state.scenes.insert(scene);

	let parent = NodeParent::Scene(scene_id);

	for node in s.nodes() {
		load_node(&node, buffers, state, parent);
	}	
}

pub fn load_gltf<P: AsRef<Path>>(p: P, state: &mut State) {
	let p = p.as_ref();
	log::info!("loading {:?}", p.to_string_lossy());

	let (document, buffers, _images) = match gltf::import(p) {
		Ok(r) => r,
		Err(_) => {
			log::error!("Failed to load gltf file");
			return;
		},
	};

	for s in document.scenes() {
		log::info!("Scene: {}", s.name().unwrap_or("Unnamed"));
		load_scene(&s, &buffers, state)
	}
}
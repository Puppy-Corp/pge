use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use glam::Quat;
use glam::Vec3;
use gltf::animation::util::MorphTargetWeights;
use gltf::animation::util::ReadOutputs;
use gltf::animation::util::Rotations;
use gltf::buffer::Data;
use crate::AnimationOutput;
use crate::ArenaId;
use crate::Material;
use crate::Mesh;
use crate::Model3D;
use crate::Node;
use crate::NodeParent;
use crate::Primitive;
use crate::PrimitiveTopology;
use crate::Scene;
use crate::State;
use crate::Animation;
use crate::AnimationChannel;
use crate::AnimationSampler;
use crate::AnimationTarget;
use crate::AnimationTargetPath;
use crate::Interpolation;
use crate::Texture;

struct ParserState {
	node_map: HashMap<usize, ArenaId<Node>>,
	texture_map: HashMap<usize, ArenaId<Texture>>,
}

impl ParserState {
	fn new() -> Self {
		ParserState {
			node_map: HashMap::new(),
			texture_map: HashMap::new(),
		}
	}
}

pub fn load_node(n: &gltf::Node, buffers: &[Data], state: &mut State, parser_state: &mut ParserState, parent: NodeParent) {
	log::info!("Loading node: {}", n.name().unwrap_or("Unnamed"));

	let mut node = Node {
		name: Some(n.name().unwrap_or_default().to_string()),
		parent,
		..Default::default()
	};

	// Set the node's transform
	let (translation, rotation, scale) = n.transform().decomposed();
	node.translation = translation.into();
	node.rotation = Quat::from_array(rotation);
	node.scale = scale.into();

	match n.mesh() {
		Some(gltf_mesh) => {
			log::info!("Mesh: {}", gltf_mesh.name().unwrap_or("Unnamed"));
			let mut mesh = Mesh::new();
			for p in gltf_mesh.primitives() {
				let mut primitive = Primitive::new(PrimitiveTopology::from_mode(p.mode()));

				log::info!("- Primitive #{}", p.index());

				let reader = p.reader(|buffer| {
					let buffer_data = &buffers[buffer.index()];
					Some(&buffer_data.0[..])
				});
				if let Some(iter) = reader.read_positions() {
					for vertex_position in iter {
						primitive.vertices.push([vertex_position[0], vertex_position[1], vertex_position[2]]);
					}
				} else {
					log::warn!("Primitive #{} is missing position data", p.index());
				}

				if let Some(iter) = reader.read_indices() {
					for index in iter.into_u32() {
						primitive.indices.push(index as u16);
					}
				} else {
					log::warn!("Primitive #{} is missing index data", p.index());
				}

				if let Some(iter) = reader.read_normals() {
					for normal in iter {
						primitive.normals.push([normal[0], normal[1], normal[2]]);
					}
				} else {
					log::warn!("Primitive #{} is missing normal data", p.index());
				}

				if let Some(iter) = reader.read_tex_coords(0) {
					for tex_coord in iter.into_f32() {
						primitive.tex_coords.push([tex_coord[0], tex_coord[1]]);
					}
				} else {
					log::warn!("Primitive #{} is missing texture coordinate data", p.index());
				}

				if reader.read_colors(0).is_none() {
					log::warn!("Primitive #{} is missing color data", p.index());
				}

				if reader.read_tangents().is_none() {
					log::warn!("Primitive #{} is missing tangent data", p.index());
				}

				mesh.primitives.push(primitive);
			}

			let mesh_id = state.meshes.insert(mesh);
			node.mesh = Some(mesh_id);
		},
		None => {
			log::info!("Node does not contain a mesh");
		}
	}
	
	let node_id = state.nodes.insert(node);
	parser_state.node_map.insert(n.index(), node_id); // Store the mapping

	for child in n.children() {
		load_node(&child, buffers, state, parser_state, NodeParent::Node(node_id));
	}
}

pub fn load_scene(s: &gltf::Scene, buffers: &[Data], state: &mut State, parser_state: &mut ParserState) -> ArenaId<Scene> {
	let scene = Scene {
		name: s.name().unwrap_or_default().to_string(),
		..Default::default()
	};

	let scene_id = state.scenes.insert(scene);
	let parent = NodeParent::Scene(scene_id);

	for node in s.nodes() {
		load_node(&node, buffers, state, parser_state, parent);
	}

	scene_id
}

pub fn load_animation(anim: &gltf::Animation, buffers: &[Data], state: &mut State, parser_state: &mut ParserState) {
	log::info!("Loading animation: {}", anim.name().unwrap_or("Unnamed"));

	let mut animation = Animation::new();

	for channel in anim.channels() {
		let target_node = channel.target().node();


		let target_node_id = match parser_state.node_map.get(&target_node.index()) { // Use parser_state
			Some(id) => id,
			None => {
				log::warn!("Animation target node not found: {}", target_node.name().unwrap_or_default().to_string());
				continue;
			}
		};

		let target_path = match channel.target().property() {
			gltf::animation::Property::Translation => AnimationTargetPath::Translation,
			gltf::animation::Property::Rotation => AnimationTargetPath::Rotation,
			gltf::animation::Property::Scale => AnimationTargetPath::Scale,
			gltf::animation::Property::MorphTargetWeights => AnimationTargetPath::Weights,
		};

		let target = AnimationTarget {
			node_id: target_node_id.clone(),
			path: target_path,
		};

		let sampler = channel.sampler();
		let sampler_index = animation.samplers.len();

		let interpolation = match sampler.interpolation() {
			gltf::animation::Interpolation::Linear => Interpolation::Linear,
			gltf::animation::Interpolation::Step => Interpolation::Stepm,
			gltf::animation::Interpolation::CubicSpline => Interpolation::Cubicspline,
		};

		let reader = channel.reader(|buffer| Some(&buffers[buffer.index()]));

		let input: Vec<f32> = reader.read_inputs().unwrap().collect();
		let output = match reader.read_outputs().unwrap() {
			ReadOutputs::Translations(output) => {
				//let m = output.map(|o| Vec3::from(o));
				//let m = m.collect::<Vec<Vec3>>();
				AnimationOutput::Translation(output.map(|p| Vec3::from(p)).collect())
			},
			ReadOutputs::Rotations(output) => {
				AnimationOutput::Rotation(match output {
					Rotations::I8(d) => d.map(|o| Quat::from_array([
						o[0] as f32 / 127.0,
						o[1] as f32 / 127.0,
						o[2] as f32 / 127.0,
						o[3] as f32 / 127.0
					])).collect(),
					Rotations::U8(d) => d.map(|o| Quat::from_array([
						o[0] as f32 / 255.0,
						o[1] as f32 / 255.0,
						o[2] as f32 / 255.0,
						o[3] as f32 / 255.0
					])).collect(),
					Rotations::I16(d) => d.map(|o| Quat::from_array([
						o[0] as f32 / 32767.0,
						o[1] as f32 / 32767.0,
						o[2] as f32 / 32767.0,
						o[3] as f32 / 32767.0
					])).collect(),
					Rotations::U16(d) => d.map(|o| Quat::from_array([
						o[0] as f32 / 65535.0,
						o[1] as f32 / 65535.0,
						o[2] as f32 / 65535.0,
						o[3] as f32 / 65535.0
					])).collect(),
					Rotations::F32(d) => d.map(|o| Quat::from_array(o)).collect(),
				})
			},
			ReadOutputs::Scales(output) => {
				AnimationOutput::Scale(output.map(|p| Vec3::from(p)).collect())
			},
			ReadOutputs::MorphTargetWeights(output) => {
				AnimationOutput::MorphWeights(match output {
					MorphTargetWeights::I8(d) => crate::types::WorphTargetWeight::I8(d.collect()),
					MorphTargetWeights::U8(d) => crate::types::WorphTargetWeight::U8(d.collect()),
					MorphTargetWeights::I16(d) => crate::types::WorphTargetWeight::I16(d.collect()),
					MorphTargetWeights::U16(d) => crate::types::WorphTargetWeight::U16(d.collect()),
					MorphTargetWeights::F32(d) => crate::types::WorphTargetWeight::F32(d.collect()),
				})
			}
		};

		animation.samplers.push(AnimationSampler {
			input,
			output,
			interpolation,
		});

		animation.channels.push(AnimationChannel {
			sampler: sampler_index,
			target,
		});
	}

	state.animations.insert(animation);
}

pub fn load_gltf<P: AsRef<Path>>(p: P, state: &mut State) -> Model3D {
	let mut model = Model3D::default();

	let mut parser_state = ParserState::new();

	let p = p.as_ref();
	log::info!("loading {:?}", p.to_string_lossy());

	let (document, buffers, images) = match gltf::import(p) {
		Ok(r) => r,
		Err(e) => {
			log::error!("Failed to load gltf file: {:?}", e);
			return model;
		},
	};

	if let Some(s) = document.default_scene() {
		log::info!("Default scene: {}", s.name().unwrap_or("Unnamed"));
		let scene_id = load_scene(&s, &buffers, state, &mut parser_state);
		model.default_scene = Some(scene_id);
	}

	for s in document.scenes() {
		log::info!("Scene: {}", s.name().unwrap_or("Unnamed"));
		let scene_id = load_scene(&s, &buffers, state, &mut parser_state);
		model.scenes.push(scene_id);
	}

	for image in images {
		log::info!("Image: {}x{}", image.width, image.height);
	}

	for animation in document.animations() {
		load_animation(&animation, &buffers, state, &mut parser_state);
	}

	for texture in document.textures() {
		let texture_id = state.textures.insert(Texture {
			name: texture.name().unwrap_or_default().to_string(),
			..Default::default()
		});
		parser_state.texture_map.insert(texture.index(), texture_id);
	}

	for m in document.materials() {
		log::info!("Material: {}", m.name().unwrap_or("Unnamed"));
		let pmr = m.pbr_metallic_roughness();
		let normal = m.normal_texture();
		let occlusion = m.occlusion_texture();
		let emissive = m.emissive_texture();

		let mut mat = Material {
			name: m.name().map(|p| p.to_string()),
			..Default::default()
		};


		if let Some(normal_texture) = normal {
			// normal_texture.tex_coord() TODO how to handle ?
			let texture_id = match parser_state.texture_map.get(&normal_texture.texture().index()) {
				Some(id) => id,
				None => {
					continue;
				}
			};
			mat.normal_texture = Some(*texture_id);
		}

		if let Some(occlusion_texture) = occlusion {
			let texture_id = match parser_state.texture_map.get(&occlusion_texture.texture().index()) {
				Some(id) => id,
				None => {
					continue;
				}
			};
			mat.occlusion_texture = Some(*texture_id);
		}

		if let Some(emissive_texture) = emissive {
			let texture_id = match parser_state.texture_map.get(&emissive_texture.texture().index()) {
				Some(id) => id,
				None => {
					continue;
				}
			};
			mat.emissive_texture = Some(*texture_id);
		}

		if let Some(base_color_texture) = pmr.base_color_texture() {
			let pbr_texture_id = match parser_state.texture_map.get(&base_color_texture.texture().index()) {
				Some(id) => id,
				None => {
					continue;
				}
			};

			mat.pbr_metallic_roughness.base_color_texture = Some(*pbr_texture_id);
		}

		mat.pbr_metallic_roughness.base_color_factor = pmr.base_color_factor();
		mat.pbr_metallic_roughness.metallic_factor = pmr.metallic_factor();
		mat.pbr_metallic_roughness.roughness_factor = pmr.roughness_factor();

		state.materials.insert(mat);
	}

	for skin in document.skins() {
		log::info!("Skin: {}", skin.name().unwrap_or("Unnamed"));
	}

	model
}


#[cfg(test)]
mod tests {
	use crate::init_logging;
	use super::*;

	#[test]
	fn test_load_gltf() {
		init_logging();

		let mut state = State::default();
		load_gltf("./assets/orkki.glb", &mut state);
		state.print_state();
	}
}
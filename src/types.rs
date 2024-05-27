use std::time::Duration;

use tokio::time::sleep;

use crate::idgen::gen_id;
use crate::math::Point3D;


#[derive(Debug)]
pub enum MouseEvent {
	Moved { dx: f32, dy: f32 }
}

#[derive(Debug)]
pub enum KeyboardKey {
	Up,
	Down,
	Left,
	Right,
}

#[derive(Debug)]
pub struct KeyboardEvent {
	pub key: KeyboardKey,
}

#[derive(Debug)]
pub enum InputEvent {
	MouseEvent(MouseEvent),
	KeyboardEvent(KeyboardEvent) ,
}

pub enum PhycicsEvent {
	Collision { id: usize }
}

#[derive(Debug)]
pub enum Event {
	InputEvent(InputEvent),
	Redraw,
}

pub enum PhysicsType {
	Static,
	Dynamic,
}

pub struct PhycicsProps {

}

pub struct Rotation {
	
}

impl Rotation {
	pub fn new() -> Self {
		Self {}
	}

	pub fn rotate(&mut self, x: f32, y: f32) {
		println!("Rotating: x: {}, y: {}", x, y);
	}
}

#[derive(Debug)]
pub enum UserEvent {
	CreateWindow
}

#[derive(Debug, Clone)]
pub struct Node {
	pub mesh: Option<Mesh>,
}

impl Node {
	pub fn new() -> Self {
		Self {
			mesh: None,
		}
	}

	pub fn set_mesh(&mut self, mesh: Mesh) {
		println!("Setting mesh");
		self.mesh = Some(mesh);
	}

	pub fn set_camera(&self, camera: Camera) {
		println!("Setting camera");
	}

	pub fn add_node(&mut self, node: Node) {
		println!("Adding node");
	}

	pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
		println!("Setting translation: x: {}, y: {}, z: {}", x, y, z);
	}

	pub fn looking_at(&mut self, x: f32, y: f32, z: f32) {
		println!("Looking at: x: {}, y: {}, z: {}", x, y, z);
	}
}

#[derive(Debug, Clone)]
pub struct Mesh {
	pub id: usize,
	pub material: Option<Material>,
	pub positions: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub text_coords: Vec<[f32; 2]>,
	pub indices: Vec<u16>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			material: None,
			positions: vec![],
			normals: vec![],
			text_coords: vec![],
			indices: vec![],
		}
	}

	pub fn set_material(&mut self, material: Material) {
		self.material = Some(material);
	}
}

pub struct Asset {
	ascenes: Vec<Scene>,
}

#[derive(Debug, Clone)]
pub struct Scene {
	pub nodes: Vec<Node>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			nodes: vec![],
		}
	}

	pub fn add_node(&mut self, node: Node) {
		println!("Adding node");
		self.nodes.push(node);
	}
}

// impl World3D {
// 	pub fn new() -> Self {
// 		Self {}
// 	}

// 	pub fn create_scene() -> Scene {
// 		Scene {}
// 	}

// 	pub fn create_node() -> Node {
// 		Node {}
// 	}

// 	pub fn create_mesh() -> Mesh {
// 		Mesh {}
// 	}

// 	pub fn create_animation() -> Animation {
// 		Animation {}
// 	}

// 	pub fn create_texture() -> Texture {
// 		Texture {}
// 	}

// 	pub fn add_camera(&self, camera: Camera) {
// 		println!("Adding camera");
// 	}
// }

pub struct Recevier {

}

impl Recevier {
	pub fn new() -> Self {
		Self {}
	}

	pub async fn recv(&self) -> Option<Event> {
		sleep(Duration::from_secs(5)).await;
		Some(Event::Redraw)
	}
}

pub struct EngineContext {

}

impl EngineContext {
	pub fn new() -> Self {
		Self {}
	}

	// pub fn create_3dworld(&self) -> World3D {
	// 	World3D::new()
	// }
}

pub struct Location {

}

impl Location {
	pub fn new() -> Self {
		Self {}
	}

	pub fn animate(&self) {
		println!("Animating location");
	}
}


pub struct Camera {
	pub id: usize,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
	pub fn new() -> Self {
		Self {
			id: 0,
			aspect: 1.0,
			fovy: 1.0,
			znear: 1.0,
			zfar: 1.0,
		}
	}

	pub fn set_location(&self, location: Location) {
		println!("Setting location");
	}
}

pub struct Channel {
	pub sampler: usize
}

pub enum Interpolation {
	Linear,
	Stepm,
	Cubicspline
}

pub struct Sampler {
	pub input: usize,
	pub output: usize,
	pub interpolation: Interpolation,
}

pub struct Animation {
	pub id: usize,
	pub channels: Vec<Channel>,
	pub samplers: Vec<Sampler>,
}

impl Animation {
	pub fn play() {
		println!("Playing animation");
	}
}

#[derive(Debug, Clone)]
pub struct Texture {
    name: String,
    source: String, // URI to the texture image
}

#[derive(Debug, Clone)]
pub struct PbrMetallicRoughness {
    base_color_factor: [f32; 4],
    metallic_factor: f32,
    roughness_factor: f32,
    base_color_texture: Option<Texture>, // Optional base color texture
}

#[derive(Debug, Clone)]
pub struct Material {
    name: Option<String>,
    pbr_metallic_roughness: PbrMetallicRoughness,
    normal_texture: Option<Texture>, // Optional normal texture
    occlusion_texture: Option<Texture>, // Optional occlusion texture
    emissive_texture: Option<Texture>, // Optional emissive texture
    emissive_factor: [f32; 3],
}

impl Material {
	pub fn new() -> Self {
		Self {
			name: None,
			pbr_metallic_roughness: PbrMetallicRoughness {
				base_color_factor: [1.0, 1.0, 1.0, 1.0],
				metallic_factor: 1.0,
				roughness_factor: 1.0,
				base_color_texture: None,
			},
			normal_texture: None,
			occlusion_texture: None,
			emissive_texture: None,
			emissive_factor: [1.0, 1.0, 1.0],
		}
	}
}
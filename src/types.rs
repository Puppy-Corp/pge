use std::time::Duration;

use glam::DQuat;
use glam::Vec3;
use tokio::time::sleep;

use crate::idgen::gen_id;
use crate::math::Point3D;


#[derive(Debug, Clone)]
pub enum MouseEvent {
	Moved { dx: f32, dy: f32 }
}

#[derive(Debug, Clone)]
pub enum KeyboardKey {
	Up,
	Down,
	Left,
	Right,
	W,
	A,
	S,
	D,
}

#[derive(Debug, Clone)]
pub enum KeyAction {
	Pressed,
	Released
}

#[derive(Debug, Clone)]
pub struct KeyboardEvent {
	pub key: KeyboardKey,
	pub action: KeyAction
}

#[derive(Debug, Clone)]
pub enum InputEvent {
	MouseEvent(MouseEvent),
	KeyboardEvent(KeyboardEvent) ,
}

pub enum PhycicsEvent {
	Collision { id: usize }
}

#[derive(Debug, Clone)]
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
	pub id: usize,
	pub mesh: Option<Mesh>,
	pub camera: Option<Camera>,
	pub translation: Vec3,
	pub rotation: glam::Quat,
	pub children: Vec<Node>,
	pub animation: Animation
}

impl Node {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			mesh: None,
			camera: None,
			translation: Vec3::ZERO,
			rotation: glam::Quat::IDENTITY,
			children: vec![],
			animation: Animation::new()
		}
	}

	pub fn set_mesh(&mut self, mesh: Mesh) {
		println!("Setting mesh");
		self.mesh = Some(mesh);
	}

	pub fn set_camera(&mut self, camera: Camera) {
		println!("Setting camera");
		self.camera = Some(camera);
	}

	pub fn add_node(&mut self, node: Node) {
		println!("Adding node");
		self.children.push(node);
	}

	pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
		println!("Setting translation: x: {}, y: {}, z: {}", x, y, z);
		self.translation = Vec3::new(x, y, z);
	}

	pub fn looking_at(&mut self, target_x: f32, target_y: f32, target_z: f32) {
        println!("Looking at: x: {}, y: {}, z: {}", target_x, target_y, target_z);
        let target = Vec3::new(target_x, target_y, target_z);
        let direction = (target - self.translation).normalize();
        
        // Assuming the node's up vector is (0, 1, 0)
        let up = Vec3::Y;
        
        // Compute the quaternion rotation
        self.rotation = glam::Quat::from_rotation_arc(Vec3::Z, direction);
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


#[derive(Debug, Clone)]
pub struct Camera {
	pub id: usize,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
	pub fn new() -> Self {
		Self {
			id: 0,
			aspect: 16.0 / 9.0,
			fovy: std::f32::consts::PI / 3.0,
			znear: 0.1,
			zfar: 100.0,
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

#[derive(Debug, Clone)]
pub struct Animation {
	pub id: usize,
	pub transform: glam::Mat4,
	// pub channels: Vec<Channel>,
	// pub samplers: Vec<Sampler>,
}

impl Animation {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			transform: glam::Mat4::IDENTITY,
			// channels: vec![],
			// samplers: vec![],
		}
	}

	pub fn play() {
		println!("Playing animation");
	}

	pub fn every(mut self, duration: Duration) -> Self {
		self
	}

	pub fn with(mut self, interpolation: Interpolation) -> Self {
		self
	}

	pub fn transform(mut self, mat: glam::Mat4) -> Self {
		self.transform = mat;
		self
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
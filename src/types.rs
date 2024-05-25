use std::time::Duration;

use tokio::time::sleep;

use crate::wgpu::renderer::Renderer;


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

pub struct Node {

}

impl Node {
	pub fn new() -> Self {
		Self {}
	}

	pub fn set_mesh(&mut self, mesh: Mesh) {
		println!("Setting mesh");
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

pub struct Mesh {
	pub material: Option<Material>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			material: None,
		}
	}

	pub fn set_material(&mut self, material: Material) {
		self.material = Some(material);
	}
}

pub struct Asset {
	ascenes: Vec<Scene>,
}

pub struct Entity {

}

impl Entity {
	pub fn set_node(&self, node: Node) {
		println!("Setting node");
	}
}

pub struct Scene {

}

impl Scene {
	pub fn new() -> Self {
		Self {}
	}

	pub fn add_node(&self, node: Node) {
		println!("Adding node");
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

struct Id(u32);

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
	pub id: usize
}

impl Camera {
	pub fn new() -> Self {
		Self {
			id: 0
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

#[derive(Debug)]
pub struct Texture {
    name: String,
    source: String, // URI to the texture image
}

#[derive(Debug)]
pub struct PbrMetallicRoughness {
    base_color_factor: [f32; 4],
    metallic_factor: f32,
    roughness_factor: f32,
    base_color_texture: Option<Texture>, // Optional base color texture
}

#[derive(Debug)]
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
use std::collections::HashMap;
use std::time::Duration;
use glam::Vec3;
use thunderdome::Arena;
use thunderdome::Index;
use tokio::time::sleep;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

use crate::idgen::gen_id;
use crate::GUIElement;
use crate::Window;


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
	Unknow
}

impl From<KeyCode> for KeyboardKey {
	fn from(key: KeyCode) -> Self {
		match key {
			KeyCode::KeyW => Self::W,
			KeyCode::KeyA => Self::A,
			KeyCode::KeyS => Self::S,
			KeyCode::KeyD => Self::D,
			_ => Self::Unknow
		}
	}
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

#[derive(Debug, Clone, PartialEq)]
pub enum PhycisObjectType {
	Static,
	Dynamic,
	None
}

impl Default for PhycisObjectType {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, Clone, Default)]
pub struct PhycicsProps {
	pub typ: PhycisObjectType,
	pub position: glam::Vec3,
	pub velocity: glam::Vec3,
	pub acceleration: glam::Vec3,
	pub mass: f32,
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
pub enum Flex {
	Horizontal,
	Vertical,
	None
}

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: glam::Vec3, // minimum point of the box (x, y, z)
    pub max: glam::Vec3, // maximum point of the box (x, y, z)
}

impl AABB {
    fn new(min: glam::Vec3, max: glam::Vec3) -> AABB {
        AABB { min, max }
    }

	pub fn empty() -> AABB {
		AABB {
			min: glam::Vec3::ZERO,
			max: glam::Vec3::ZERO,
		}
	}

    pub fn contains(&self, point: glam::Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }
}

#[derive(Debug, Clone)]
pub struct Node {
	pub id: usize,
	pub name: Option<String>,
	pub parent: Option<Index>,
	pub mesh: Option<Index>,
	pub translation: glam::Vec3,
	pub rotation: glam::Quat,
	pub scale: glam::Vec3,
	pub animation: Animation,
	pub point_light: Option<PointLight>,
	pub texture: Option<Texture>,
	pub physics: PhycicsProps,
	pub forces: Vec<PhysicsForce>,
	pub flex: Flex,
	pub aabb: AABB
}

impl Node {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			name: None,
			parent: None,
			mesh: None,
			point_light: None,
			translation: glam::Vec3::ZERO,
			rotation: glam::Quat::IDENTITY,
			scale: glam::Vec3::ONE,
			animation: Animation::new(),
			texture: None,
			physics: PhycicsProps::default(),
			forces: Vec::new(),
			flex: Flex::None,
			aabb: AABB::empty()
		}
	}

	pub fn set_mesh(mut self, mesh_id: Index) -> Node {
		self.mesh = Some(mesh_id);
		self
	}

    pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
        self.translation = glam::Vec3::new(x, y, z);
    }

    pub fn looking_at(&mut self, x: f32, y: f32, z: f32) {
		let target = glam::Vec3::new(x, y, z);
		let direction = (target - self.translation).normalize();
		self.rotation = glam::Quat::from_rotation_arc(glam::Vec3::Z, direction);
    }

	pub fn mov(&mut self, x: f32, y: f32, z: f32) {
		
	}

    pub fn rotate(&mut self, dx: f32, dy: f32) {
		let rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, dy, dx, 0.0);
		self.rotation = rotation * self.rotation;
    }

    pub fn scale(&mut self, x: f32, y: f32, z: f32) {
        self.scale = glam::Vec3::new(x, y, z);
    }

	pub fn flex(mut self, flex: Flex) -> Self {
		self.flex = flex;
		self
	}
}

pub fn vstack() -> Node {
	Node::new().flex(Flex::Vertical)
}

pub fn hstack() -> Node {
	Node::new().flex(Flex::Horizontal)
}

#[derive(Debug, Clone)]
pub struct Mesh {
	// pub id: usize,
	pub name: Option<String>,
	pub material: Option<Material>,
	pub positions: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub text_coords: Vec<[f32; 2]>,
	pub colors: Vec<[f32; 4]>,
	pub indices: Vec<u16>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			// id: gen_id(),
			name: None,
			material: None,
			positions: vec![],
			normals: vec![],
			text_coords: vec![],
			colors: vec![],
			indices: vec![],
		}
	}

	pub fn set_name(mut self, name: &str) -> Self {
		self.name = Some(name.to_string());
		self
	}

	pub fn set_material(&mut self, material: Material) {
		self.material = Some(material);
	}

	pub fn add_mesh(&mut self, mesh: Mesh) {
		self.positions.extend(mesh.positions);
		self.normals.extend(mesh.normals);
		self.text_coords.extend(mesh.text_coords);
		self.indices.extend(mesh.indices);
	}
}

pub struct Asset {
	ascenes: Vec<Scene>,
}

#[derive(Debug, Clone)]
pub struct Scene {
	pub nodes: Vec<Index>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			nodes: vec![],
		}
	}
}

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
	pub node_id: Option<Index>
}

impl Camera {
	pub fn new() -> Self {
		Self {
			id: 0,
			aspect: 16.0 / 9.0,
			fovy: std::f32::consts::PI / 3.0,
			znear: 0.1,
			zfar: 100.0,
			node_id: None
		}
	}

	pub fn set_location(&self, location: Location) {
		println!("Setting location");
	}
}

pub struct Channel {
	pub sampler: usize
}

#[derive(Debug, Clone)]
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
pub struct Step {
	pub duration: Duration,
	pub transform: glam::Mat4,
	pub interpolation: Interpolation,
}

#[derive(Debug, Clone)]
pub struct Animation {
	pub id: usize,
	pub steps: Vec<Step>,
	pub transform: glam::Mat4,
	// pub channels: Vec<Channel>,
	// pub samplers: Vec<Sampler>,
}

impl Animation {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			steps: vec![],
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

#[derive(Debug, Clone)]
pub struct PointLight {
	pub id: usize,
	pub color: [f32; 3],
	pub intensity: f32,
}

impl PointLight {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			color: [1.0, 1.0, 1.0],
			intensity: 1.0,
		}
	}
}

// pub strut PhycicsProperties {
	
// }

#[derive(Debug, Clone)]
pub struct FontHandle {
	pub id: usize
}

impl FontHandle {
	pub fn new(id: usize) -> Self {
		Self {
			id
		}
	}
}

pub struct WorldHandle {

}

impl WorldHandle {
	pub fn add_node(&self, node: Node) {
		println!("Adding node");
	}

	pub fn create_node(&self) -> Node {
		Node::new()
	}

	pub fn create_camera(&self) -> Camera {
		Camera::new()
	}
}

pub struct World3D {
	pub nodes: HashMap<usize, Node>
}

impl World3D {
	pub fn new() -> Self {
		Self {
			nodes: HashMap::new()
		}
	}

	pub fn add_node(&mut self, node: Node) {
		self.nodes.insert(node.id, node);
	}
}

pub struct WindowHandle {

}

impl WindowHandle {
	pub fn set_gui(&self, gui: GUIElement) {
		println!("Setting GUI");
	}
}

#[derive(Debug, Clone)]
pub struct PhysicsForce {
	pub id: usize,
	pub force: glam::Vec3,
	/// If objects velocity is greater than max_velocity to the direction, 
	/// this force will not be applied
	pub max_velocity: f32
}

impl PhysicsForce {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			force: glam::Vec3::ZERO,
			max_velocity: 0.0,
		}
	}

	pub fn set_force(mut self, force: glam::Vec3) -> Self {
		self.force = force;
		self
	}
}

#[derive(Debug, Clone, Default)]
pub struct State {
	pub scenes: Arena<Scene>,
	pub meshes: Arena<Mesh>,
	pub nodes: Arena<Node>,
	pub cameras: Arena<Camera>,
	pub windows: Arena<Window>,
	pub guis: Arena<GUIElement>,
}

pub trait App {
	fn on_create(&mut self, state: &mut State) {}
	fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {}
	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {}
	/// Run before rendering
	fn on_process(&mut self, state: &mut State, delta: f32) {}
	/// Run before physics properties are updated
	fn on_phycis_update(&mut self, state: &mut State, delta: f32) {}
}


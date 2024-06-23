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
}

impl From<KeyCode> for KeyboardKey {
	fn from(key: KeyCode) -> Self {
		match key {
			KeyCode::KeyW => Self::W,
			KeyCode::KeyA => Self::A,
			KeyCode::KeyS => Self::S,
			KeyCode::KeyD => Self::D,
			_ => todo!("key {:?} not supported", key)
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

#[derive(Debug, Clone)]
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
	pub parent: Option<Index>,
	pub mesh: Option<Index>,
	pub camera: Option<Index>,
	pub translation: Vec3,
	pub rotation: glam::Quat,
	pub scale: Vec3,
	pub animation: Animation,
	pub point_light: Option<PointLight>,
	pub texture: Option<Texture>,
	pub physics: PhycicsProps,
	pub forces: Vec<PhycicsForce>,
}

impl Node {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			parent: None,
			mesh: None,
			camera: None,
			point_light: None,
			translation: Vec3::ZERO,
			rotation: glam::Quat::IDENTITY,
			scale: Vec3::ONE,
			animation: Animation::new(),
			texture: None,
			physics: PhycicsProps::default(),
			forces: Vec::new(),
		}
	}

	pub fn set_mesh(&mut self, mesh_id: Index) {
		self.mesh = Some(mesh_id);
	}

	pub fn set_translation(&mut self, x: f32, y: f32, z: f32) {
		self.translation = Vec3::new(x, y, z);
	}

	pub fn looking_at(&mut self, target_x: f32, target_y: f32, target_z: f32) {
        let target = Vec3::new(target_x, target_y, target_z);
        let direction = (target - self.translation).normalize();
        
        // Assuming the node's up vector is (0, 1, 0)
        let up = Vec3::Y;
        
        // Compute the quaternion rotation
        self.rotation = glam::Quat::from_rotation_arc(Vec3::Z, direction);
    }

	pub fn rotate(&mut self, x: f32, y: f32, z: f32) {
		// self.rotation = glam::Quat::from_rotation_ypr(y, x, z);
	}

	pub fn scale(&mut self, x: f32, y: f32, z: f32) {
		self.scale = Vec3::new(x, y, z);
	}

	pub fn set_point_light(&mut self, light: PointLight) {
		self.point_light = Some(light);
	}

	pub fn add_force(&mut self, force: PhycicsForce) {
		println!("Adding force");
	}

	pub fn update_force(&mut self, force: PhycicsForce) {
		println!("Updating force");
	}

	pub fn set_phycis_props(&self, props: PhycicsProps) {
		println!("Setting physics props");
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
pub struct PhycicsForce {
	pub id: usize,
	pub direction: glam::Vec3,
	pub force: f32,
	/// If objects velocity is greater than max_velocity to the direction, 
	/// this force will not be applied
	pub max_velocity: f32
}

impl PhycicsForce {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			direction: glam::Vec3::ZERO,
			force: 0.0,
			max_velocity: 0.0,
		}
	}

	pub fn set_direction(&mut self, direction: glam::Vec3) {
		self.direction = direction;
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
	fn on_create(&mut self, state: &mut State) {
		println!("Creating app");
	}

	fn on_keyboard_input(&mut self, key: KeyboardKey, action: KeyAction, state: &mut State) {
		println!("Keyboard input: {:?}, {:?}", key, action);
	}

	fn on_mouse_input(&mut self, event: MouseEvent, state: &mut State) {
		println!("Mouse input: {:?}", event);
	}
}
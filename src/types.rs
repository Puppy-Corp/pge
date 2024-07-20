use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use glam::Vec3;
use thunderdome::Arena;
use thunderdome::Index;
use winit::keyboard::KeyCode;
use winit::keyboard::PhysicalKey;

use crate::idgen::gen_id;
use crate::physics::PhysicsSystem;
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
	Space,
	ShiftLeft,
	Unknow
}

impl From<KeyCode> for KeyboardKey {
	fn from(key: KeyCode) -> Self {
		match key {
			KeyCode::KeyW => Self::W,
			KeyCode::KeyA => Self::A,
			KeyCode::KeyS => Self::S,
			KeyCode::KeyD => Self::D,
			KeyCode::Space => Self::Space,
			KeyCode::ShiftLeft => Self::ShiftLeft,
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
pub struct PhysicsProps {
	pub typ: PhycisObjectType,
	pub position: glam::Vec3,
	pub velocity: glam::Vec3,
	pub acceleration: glam::Vec3,
	pub mass: f32,
	pub stationary: bool,
	pub force: Vec3
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
    pub fn new(min: glam::Vec3, max: glam::Vec3) -> AABB {
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

    pub fn get_correction(&self, other: &AABB) -> Vec3 {
        let mut correction = Vec3::ZERO;

        // Calculate penetration depths
        let dx = (self.max.x - other.min.x).min(other.max.x - self.min.x);
        let dy = (self.max.y - other.min.y).min(other.max.y - self.min.y);
        let dz = (self.max.z - other.min.z).min(other.max.z - self.min.z);

        if dx.abs() < dy.abs() && dx.abs() < dz.abs() {
            // Move along x-axis
            if self.max.x - other.min.x < other.max.x - self.min.x {
                // Move self to the left
                correction.x = -dx;
            } else {
                // Move self to the right
                correction.x = dx;
            }
        } else if dy.abs() < dx.abs() && dy.abs() < dz.abs() {
            // Move along y-axis
            if self.max.y - other.min.y < other.max.y - self.min.y {
                // Move self down
                correction.y = -dy;
            } else {
                // Move self up
                correction.y = dy;
            }
        } else {
            // Move along z-axis
            if self.max.z - other.min.z < other.max.z - self.min.z {
                // Move self backward
                correction.z = -dz;
            } else {
                // Move self forward
                correction.z = dz;
            }
        }

        correction
    }
}

pub struct ConvexHull {
	vertices: Vec<glam::Vec3>,
}

#[derive(Debug, Clone)]
pub enum CollisionShape {
	Sphere { radius: f32 },
	Box { size: glam::Vec3 },
	Capsule { radius: f32, height: f32 },
	ConvexHull { vertices: Vec<glam::Vec3> }
}

impl CollisionShape {
    pub fn aabb(&self, translation: glam::Vec3) -> AABB {
        match self {
            Self::Sphere { radius } => AABB {
                min: translation + glam::Vec3::splat(-*radius),
                max: translation + glam::Vec3::splat(*radius),
            },
            Self::Box { size } => AABB {
                min: translation - *size,
                max: translation + *size,
            },
            Self::Capsule { radius, height } => AABB {
                min: translation + glam::Vec3::new(-*radius, -(*height / 2.0 + *radius), -*radius),
                max: translation + glam::Vec3::new(*radius, *height / 2.0 + *radius, *radius),
            },
            Self::ConvexHull { vertices } => {
                if vertices.is_empty() {
                    return AABB {
                        min: translation,
                        max: translation,
                    };
                }

                let mut min = translation + vertices[0];
                let mut max = translation + vertices[0];

                for vertex in vertices.iter().skip(1) {
                    let translated_vertex = translation + *vertex;
                    min = min.min(translated_vertex);
                    max = max.max(translated_vertex);
                }

                AABB { min, max }
            }
        }
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
	pub physics: PhysicsProps,
	pub flex: Flex,
	pub collision_shape: Option<CollisionShape>
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
			physics: PhysicsProps::default(),
			flex: Flex::None,
			collision_shape: None
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
		let forward = (glam::Vec3::new(x, y, z) - self.translation).normalize_or_zero();
		let right = glam::Vec3::Y.cross(forward).normalize_or_zero();
		let up = forward.cross(right).normalize_or_zero();
		let rotation_matrix = glam::Mat3::from_cols(right, up, forward);
		self.rotation = glam::Quat::from_mat3(&rotation_matrix);
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

#[derive(Debug, Clone)]
pub struct Mesh {
	// pub id: usize,
	pub name: Option<String>,
	pub material: Option<Material>,
	pub positions: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub tex_coords: Vec<[f32; 2]>,
	pub colors: Vec<[f32; 4]>,
	pub indices: Vec<u16>,
	pub texture: Option<Index>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			// id: gen_id(),
			name: None,
			material: None,
			positions: vec![],
			normals: vec![],
			tex_coords: vec![],
			colors: vec![],
			indices: vec![],
			texture: None,
		}
	}

	pub fn set_name(mut self, name: &str) -> Self {
		self.name = Some(name.to_string());
		self
	}

	pub fn set_material(&mut self, material: Material) {
		self.material = Some(material);
	}

	pub fn set_texture(mut self, texture: Index) -> Self {
		self.texture = Some(texture);
		self
	}

	pub fn add_mesh(&mut self, mesh: Mesh) {
		self.positions.extend(mesh.positions);
		self.normals.extend(mesh.normals);
		self.tex_coords.extend(mesh.tex_coords);
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
			fovy: 45.0,
			znear: 0.1,
			zfar: 100.0,
			node_id: None
		}
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
    pub name: String,
    pub source: String, // URI to the texture image
}

impl Texture {
	pub fn new<P: AsRef<Path>>(path: P) -> Self {
		let path = path.as_ref();

		Self {
			name: "".to_string(),
			source: path.to_str().unwrap().to_string(),
		}
	}
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
	pub node_id: Option<Index>
}

impl PointLight {
	pub fn new() -> Self {
		Self {
			id: gen_id(),
			color: [1.0, 1.0, 1.0],
			intensity: 1.0,
			node_id: None
		}
	}
}

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


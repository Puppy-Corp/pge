use std::collections::HashMap;
use std::path::Path;
use glam::Quat;
use glam::Vec3;
use crate::arena::Arena;
use crate::arena::ArenaId;
use crate::gltf::load_gltf;
use crate::state::State;
use crate::GUIElement;
use crate::Window;

#[derive(Debug, Clone)]
pub enum MouseButton {
	Left,
	Right,
	Middle,
}

/*impl From<winit::event::MouseButton> for MouseButton {
	fn from(button: winit::event::MouseButton) -> Self {
		match button {
			winit::event::MouseButton::Left => Self::Left,
			winit::event::MouseButton::Right => Self::Right,
			winit::event::MouseButton::Middle => Self::Middle,
			_ => panic!("Unknown mouse button"),
		}
	}
}*/

#[derive(Debug, Clone)]
pub enum MouseEvent {
	Moved { dx: f32, dy: f32 },
	Pressed { button: MouseButton },
	Released { button: MouseButton },
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
	F,
	G,
	R,
	Z,
	E,
	ControlLeft,
	Space,
	ShiftLeft,
	Digit1,
	Digit2,
	Digit3,
	Digit4,
	Digit5,
	Digit6,
	Unknow
}

/*impl From<KeyCode> for KeyboardKey {
	fn from(key: KeyCode) -> Self {
		match key {
			KeyCode::KeyW => Self::W,
			KeyCode::KeyA => Self::A,
			KeyCode::KeyS => Self::S,
			KeyCode::KeyD => Self::D,
			KeyCode::Space => Self::Space,
			KeyCode::ShiftLeft => Self::ShiftLeft,
			KeyCode::KeyF => Self::F,
			KeyCode::KeyG => Self::G,
			KeyCode::Digit1 => Self::Digit1,
			KeyCode::Digit2 => Self::Digit2,
			KeyCode::Digit3 => Self::Digit3,
			KeyCode::Digit4 => Self::Digit3,
			KeyCode::Digit5 => Self::Digit4,
			KeyCode::Digit6 => Self::Digit6,
			KeyCode::KeyR => Self::R,
			KeyCode::KeyE => Self::E,
			KeyCode::ControlLeft => Self::ControlLeft,
			KeyCode::KeyZ => Self::Z,
			_ => Self::Unknow
		}
	}
}*/

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

#[derive(Debug, Clone)]
pub struct RayCast {
	pub node_id: ArenaId<Node>,
	pub len: f32,
	pub intersects: Vec<ArenaId<Node>>,
}

impl RayCast {
	pub fn new(node_inx: ArenaId<Node>, len: f32) -> Self {
		Self {
			node_id: node_inx,
			len,
			intersects: vec![]
		}
	}
}

pub struct SphereCast {
	pub origin: glam::Vec3,
	pub radius: f32,
	pub length: f32,
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

	pub fn intersect_ray(&self, start: Vec3, end: Vec3) -> Option<(f32, f32)> {
		let dir = end - start;
		let inv_dir = Vec3::new(1.0 / dir.x, 1.0 / dir.y, 1.0 / dir.z);
		let len = dir.length();
	
		let t1 = (self.min.x - start.x) * inv_dir.x;
		let t2 = (self.max.x - start.x) * inv_dir.x;
		let t3 = (self.min.y - start.y) * inv_dir.y;
		let t4 = (self.max.y - start.y) * inv_dir.y;
		let t5 = (self.min.z - start.z) * inv_dir.z;
		let t6 = (self.max.z - start.z) * inv_dir.z;
	
		let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
		let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
	
		if tmax >= tmin && tmin <= len && tmax >= 0.0 {
			Some((tmin, tmax))
		} else {
			None
		}
    }

	pub fn intersect_sphere(&self, sphere: &SphereCast) -> bool {
        let mut closest_point = Vec3::ZERO;

        // Find the closest point on the AABB to the sphere's origin
        closest_point.x = sphere.origin.x.max(self.min.x).min(self.max.x);
        closest_point.y = sphere.origin.y.max(self.min.y).min(self.max.y);
        closest_point.z = sphere.origin.z.max(self.min.z).min(self.max.z);

        // Calculate the distance between the sphere's origin and the closest point
        let distance = closest_point.distance(sphere.origin);

        // Check if the distance is less than or equal to the sphere's radius
        distance <= sphere.radius
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NodeParent {
	Node(ArenaId<Node>),
	Scene(ArenaId<Scene>),
	Orphan
}

impl Default for NodeParent {
	fn default() -> Self {
		Self::Orphan
	}
}

pub struct NodeId;

#[derive(Debug, Clone)]
pub struct Node {
	pub name: Option<String>,
	pub parent: NodeParent,
	pub mesh: Option<ArenaId<Mesh>>,
	pub translation: glam::Vec3,
	pub rotation: glam::Quat,
	pub scale: glam::Vec3,
	pub physics: PhysicsProps,
	pub collision_shape: Option<CollisionShape>,
	pub global_transform: glam::Mat4,
	pub scene_id: Option<ArenaId<Scene>>,
}

impl Default for Node {
	fn default() -> Self {
		Self {
			name: None,
			parent: NodeParent::Orphan,
			mesh: None,
			translation: glam::Vec3::ZERO,
			rotation: glam::Quat::IDENTITY,
			scale: glam::Vec3::splat(1.0),
			physics: PhysicsProps::default(),
			collision_shape: None,
			global_transform: glam::Mat4::IDENTITY,
			scene_id: None,
		}
	}
}

impl Node {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn set_mesh(mut self, mesh_id: ArenaId<Mesh>) -> Node {
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

	pub fn matrix(&self) -> glam::Mat4 {
		let translation = glam::Mat4::from_translation(self.translation);
		let rotation = glam::Mat4::from_quat(self.rotation);
		let scale = glam::Mat4::from_scale(self.scale);

		translation * rotation * scale
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveTopology {
	PointList,
	LineList,
	LineStrip,
	TriangleList,
	TriangleStrip
}

impl PrimitiveTopology {
	pub fn from_mode(mode: gltf::mesh::Mode) -> Self {
		match mode {
			gltf::mesh::Mode::Points => PrimitiveTopology::PointList,
			gltf::mesh::Mode::Lines => PrimitiveTopology::LineList,
			gltf::mesh::Mode::LineStrip => PrimitiveTopology::LineStrip,
			gltf::mesh::Mode::Triangles => PrimitiveTopology::TriangleList,
			gltf::mesh::Mode::TriangleStrip => PrimitiveTopology::TriangleStrip,
			_ => panic!("Invalid primitive topology")
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub struct Primitive {
	pub topology: PrimitiveTopology,
	pub vertices: Vec<[f32; 3]>,
	pub indices: Vec<u16>,
	pub normals: Vec<[f32; 3]>,
	pub tex_coords: Vec<[f32; 2]>,
	pub material: Option<ArenaId<Material>>,
}

impl Primitive {
	pub fn new(topology: PrimitiveTopology) -> Self {
		Self {
			topology,
			vertices: vec![],
			indices: vec![],
			normals: vec![],
			tex_coords: vec![],
			material: None,
		}
	}
}

#[derive(Debug, Clone)]
pub struct MeshId;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
	pub name: Option<String>,
	pub primitives: Vec<Primitive>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			name: None,
			primitives: vec![],
		}
	}

	pub fn set_name(mut self, name: &str) -> Self {
		self.name = Some(name.to_string());
		self
	}
}

pub struct Asset {
	ascenes: Vec<Scene>,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
	pub name: Option<String>,
	pub scale: glam::Vec3,
	pub _3d_model: Option<ArenaId<Model3D>>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			name: None,
			scale: glam::Vec3::splat(1.0),
			_3d_model: None,
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct Camera {
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
	pub node_id: Option<ArenaId<Node>>
}

impl Camera {
	pub fn new() -> Self {
		Self {
			aspect: 16.0 / 9.0,
			fovy: 45.0,
			znear: 0.1,
			zfar: 100.0,
			node_id: None
		}
	}
}

#[derive(Debug, Clone)]
pub enum AnimationTargetPath {
	Translation,
	Rotation,
	Scale,
	Weights,
}

#[derive(Debug, Clone)]
pub struct AnimationTarget {
	pub node_id: ArenaId<Node>,
	pub path: AnimationTargetPath,
}

#[derive(Debug, Clone)]
pub struct AnimationChannel {
	pub sampler: usize,
	pub target: AnimationTarget,
}

#[derive(Debug, Clone)]
pub enum Interpolation {
	Linear,
	Stepm,
	Cubicspline
}

#[derive(Debug, Clone)]
pub enum WorphTargetWeight {
	I8(Vec<i8>),
	U8(Vec<u8>),
	I16(Vec<i16>),
	U16(Vec<u16>),
	I32(Vec<i32>),
	U32(Vec<u32>),
	F32(Vec<f32>),
}

#[derive(Debug, Clone)]
pub enum AnimationOutput {
	Translation(Vec<Vec3>),
	Rotation(Vec<Quat>),
	Scale(Vec<Vec3>),
	MorphWeights(WorphTargetWeight),
}

#[derive(Debug, Clone)]
pub struct AnimationSampler {
	pub input: Vec<f32>,
	pub output: AnimationOutput,
	pub interpolation: Interpolation,
}

#[derive(Debug, Clone)]
pub struct Animation {
	pub channels: Vec<AnimationChannel>,
	pub samplers: Vec<AnimationSampler>,
}

impl Animation {
	pub fn new() -> Self {
		Self {
			channels: vec![],
			samplers: vec![],
		}
	}
}

#[derive(Debug, Clone)]
pub enum TextureSource {
	None,
	File(String),
	Buffer {
		data: Vec<u8>,
		width: u32,
		height: u32,
	},
}

impl Default for TextureSource {
	fn default() -> Self {
		Self::None
	}
}

#[derive(Debug, Clone, Default)]
pub struct Texture {
    pub name: String,
    pub source: TextureSource,
}

impl Texture {
	pub fn new<P: AsRef<Path>>(path: P) -> Self {
		let path = path.as_ref();

		Self {
			name: "".to_string(),
			source: TextureSource::File(path.to_str().unwrap().to_string()),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Material {
    pub name: Option<String>,
	pub base_color_texture: Option<ArenaId<Texture>>,
	pub base_color_tex_coords: Option<Vec<[f32; 2]>>,
	pub base_color_factor: [f32; 4],
	pub metallic_roughness_texture: Option<ArenaId<Texture>>,
	pub metallic_roughness_tex_coords: Option<Vec<[f32; 2]>>,
	pub metallic_factor: f32,
	pub roughness_factor: f32,
	pub normal_texture: Option<ArenaId<Texture>>,
	pub normal_tex_coords: Option<Vec<[f32; 2]>>,
	pub normal_texture_scale: f32,
	pub occlusion_texture: Option<ArenaId<Texture>>,
	pub occlusion_tex_coords: Option<Vec<[f32; 2]>>,
	pub occlusion_strength: f32,
	pub emissive_texture: Option<ArenaId<Texture>>,
	pub emissive_tex_coords: Option<Vec<[f32; 2]>>,
	pub emissive_factor: [f32; 3],
}

impl Default for Material {
	fn default() -> Self {
		Self {
			name: None,
			base_color_texture: None,
			base_color_tex_coords: None,
			base_color_factor: [1.0, 1.0, 1.0, 1.0],
			metallic_roughness_texture: None,
			metallic_roughness_tex_coords: None,
			metallic_factor: 0.0,
			roughness_factor: 1.0,
			normal_texture: None,
			normal_tex_coords: None,
			normal_texture_scale: 1.0,
			occlusion_texture: None,
			occlusion_tex_coords: None,
			occlusion_strength: 1.0,
			emissive_texture: None,
			emissive_tex_coords: None,
			emissive_factor: [0.0, 0.0, 0.0],
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct PointLight {
	pub color: [f32; 3],
	pub intensity: f32,
	pub node_id: Option<ArenaId<Node>>
}

impl PointLight {
	pub fn new() -> Self {
		Self {
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

#[derive(Debug, Clone, Default)]
pub struct Model3D {
	pub path: String,
	pub default_scene: Option<ArenaId<Scene>>,
	pub scenes: Vec<ArenaId<Scene>>,
	pub animations: Vec<Animation>,
}

#[derive(Debug, Clone, Default)]
pub struct Keyboard {
	pub pressed: Vec<KeyboardKey>,
}

pub trait App {
	fn on_create(&mut self, state: &mut State) {}
	fn on_keyboard_input(&mut self, window_id: ArenaId<Window>, key: KeyboardKey, action: KeyAction, state: &mut State) {}
	fn on_mouse_input(&mut self, window_id: ArenaId<Window>, event: MouseEvent, state: &mut State) {}
	/// Run before rendering
	fn on_process(&mut self, state: &mut State, delta: f32) {}
	/// Run before physics properties are updated
	fn on_phycis_update(&mut self, state: &mut State, delta: f32) {}
}




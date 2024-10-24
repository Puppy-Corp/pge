use std::path::Path;
use glam::Quat;
use glam::Vec3;
use crate::arena::Arena;
use crate::arena::ArenaId;
use crate::gltf::load_gltf;
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

	pub fn model_matrix(&self) -> glam::Mat4 {
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

#[derive(Debug, Clone)]
pub struct Primitive {
	pub topology: PrimitiveTopology,
	pub vertices: Vec<[f32; 3]>,
	pub indices: Vec<u16>,
	pub normals: Vec<[f32; 3]>,
	pub tex_coords: Vec<[f32; 2]>,	
}

impl Primitive {
	pub fn new(topology: PrimitiveTopology) -> Self {
		Self {
			topology,
			vertices: vec![],
			indices: vec![],
			normals: vec![],
			tex_coords: vec![],
		}
	}
}

#[derive(Debug, Clone)]
pub struct MeshId;

#[derive(Debug, Clone, Default)]
pub struct Mesh {
	// pub id: usize,
	pub name: Option<String>,
	// pub material: Option<Material>,
	// pub positions: Vec<[f32; 3]>,
	// pub normals: Vec<[f32; 3]>,
	// pub tex_coords: Vec<[f32; 2]>,
	// pub colors: Vec<[f32; 4]>,
	// pub indices: Vec<u16>,
	pub texture: Option<ArenaId<Texture>>,
	pub primitives: Vec<Primitive>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			// id: gen_id(),
			name: None,
			// material: None,
			// positions: vec![],
			// normals: vec![],
			// tex_coords: vec![],
			// colors: vec![],
			// indices: vec![],
			texture: None,
			primitives: vec![],
		}
	}

	pub fn set_name(mut self, name: &str) -> Self {
		self.name = Some(name.to_string());
		self
	}

	pub fn set_texture(mut self, texture: ArenaId<Texture>) -> Self {
		self.texture = Some(texture);
		self
	}
}

pub struct Asset {
	ascenes: Vec<Scene>,
}

#[derive(Debug, Clone, Default)]
pub struct Scene {
	pub name: Option<String>,
	pub _3d_model: Option<ArenaId<Model3D>>,
}

impl Scene {
	pub fn new() -> Self {
		Self {
			name: None,
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

#[derive(Debug, Clone, Default)]
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

#[derive(Debug, Clone, Default)]
pub struct PbrMetallicRoughness {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub base_color_texture: Option<ArenaId<Texture>>, // Optional base color texture
}

#[derive(Debug, Clone, Default)]
pub struct Material {
    pub name: Option<String>,
    pub pbr_metallic_roughness: PbrMetallicRoughness,
    pub normal_texture: Option<ArenaId<Texture>>, // Optional normal texture
    pub occlusion_texture: Option<ArenaId<Texture>>, // Optional occlusion texture
    pub emissive_texture: Option<ArenaId<Texture>>, // Optional emissive texture
    pub emissive_factor: [f32; 3],
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

	pub fn get_mesh_nodes(&self, mesh_id: ArenaId<Mesh>) -> Vec<ArenaId<Node>> {
		todo!()
	}

	pub fn get_node_transformation(&self, node_id: ArenaId<Node>) -> glam::Mat4 {
		todo!()
	}

	pub fn get_node_scene(&self, node_id: ArenaId<Node>) -> ArenaId<Scene> {
		todo!()
	}

	// pub fn clone_node(&mut self, node_id: ArenaId<Node>) -> ArenaId<Node> {
	// 	let node = self.nodes.get(node_id).unwrap();
	// 	let mut new_node = node.clone();
	// 	new_node.parent = NodeParent::Orphan;
	// 	self.nodes.insert(new_node)
	// }
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




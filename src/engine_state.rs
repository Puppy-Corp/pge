use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Range;
use std::time::Instant;

use bytemuck::bytes_of;
use glam::Vec2;
use glam::Vec3;
use thunderdome::Arena;
use thunderdome::Index;

use crate::buffer::DirtyBuffer;
use crate::compositor::UICompositor;
use crate::debug::ChangePrinter;
use crate::gltf::load_gltf;
use crate::internal_types::CamView;
use crate::physics::PhysicsSystem;
use crate::spatial_grid::SpatialGrid;
use crate::wgpu_types::*;
use crate::Mesh;
use crate::NodeParent;
use crate::PrimitiveTopology;
use crate::State;
use crate::Texture;
use crate::AABB;

const REM_NODE_SLOT: u32 = 0;
const ADD_NODE_SLOT: u32 = 1;
const NODE_UPDATE_TIME_SLOT: u32 = 2;
const BROAD_PHASE_TIME_SLOT: u32 = 3;
const NARROW_PHASE_TIME_SLOT: u32 = 4;
const NODES_COUNT_SLOT: u32 = 5;
const MESHES_COUNT_SLOT: u32 = 6;
const CAMERAS_SLOT: u32 = 7;
const POINT_LIGHTS_COUNT_SLOT: u32 = 8;
const SCENES_COUNT_SLOT: u32 = 9;
const DRAW_CALLS_SLOT: u32 = 10;
const MESH_NODES_SLOT: u32 = 11;
const UI_RENDER_ARGS_SLOT: u32 = 12;

#[derive(Debug, Clone, Default)]
pub struct Gemometry {
    pub vertices: DirtyBuffer,
    pub normals: DirtyBuffer,
    pub tex_coords: DirtyBuffer,
    pub indices: DirtyBuffer,
}

impl Gemometry {
    pub fn new() -> Self {
        Self {
            vertices: DirtyBuffer::new("vertices"),
            normals: DirtyBuffer::new("normals"),
            tex_coords: DirtyBuffer::new("tex_coords"),
            indices: DirtyBuffer::new("indices"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrawCall {
    pub texture: Option<Index>,
    pub vertices: Range<u64>,
    pub indices: Range<u64>,
    pub normals: Range<u64>,
    pub tex_coords: Range<u64>,
    pub instances: Range<u32>,
    pub indices_range: Range<u32>,
}

pub struct UIDrawCalls {
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
    pub colors: Vec<[f32; 3]>,
}

pub struct DrawCalls3D {
    pub camera: Vec<u8>,
    pub nodes: Vec<u8>,
    pub point_lights: Vec<u8>,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub calls: Vec<DrawCall>,
}

// #[derive(Debug, Clone)]
// pub struct WindowDrawData {
// 	pub ui: UIDrawCalls,
// 	pub draw_calls: DrawCalls3D,
// }

struct GuiBuffers {
    vertices: DirtyBuffer,
    indices: DirtyBuffer,
    colors: DirtyBuffer,
}

struct SceneData {
    nodes: Vec<Index>,
}

#[derive(Debug, Clone)]
struct NodeMetadata {
    model: glam::Mat4,
    scene_id: Index,
}

#[derive(Debug, Clone)]
struct CollisionNode {
    node_id: Index,
    aabb: AABB,
}

#[derive(Debug, Clone)]
struct MeshPointer {
    positions: Range<u64>,
    normals: Range<u64>,
    tex_coords: Range<u64>,
    indices: Range<u64>,
    indice_count: u32,
}

struct SceneDrawInstruction {
    draw_calls: Vec<DrawCall>,
}

#[derive(Debug, Clone)]
pub struct View {
    pub camview: CamView,
    pub scene_id: Index,
}

#[derive(Debug, Clone)]
pub struct UIRenderArgs {
    pub ui: Index,
    pub views: Vec<View>,
}

#[derive(Debug, Clone)]
pub struct SceneCollection {
    collision_nodes: Vec<CollisionNode>,
    draw_calls: Vec<DrawCall>,
    grid: SpatialGrid,
    physics_system: PhysicsSystem,
}

#[derive(Debug, Clone)]
pub struct Buffer {
    pub data: Vec<u8>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Default)]
pub struct EngineState {
    pub state: State,
    grids: HashMap<Index, SpatialGrid>,
    nodes: HashMap<Index, NodeMetadata>,
    instances: HashMap<Index, RawInstance>,
    meshes: HashMap<Index, Mesh>,
    cameras: HashMap<Index, RawCamera>,
    printer: ChangePrinter,
    pub all_instances_data: Vec<u8>,
    pub camera_buffers: HashMap<Index, DirtyBuffer>,
    // camera_draw_calls: HashMap<Index, Vec<DrawCall>>,
    pub all_point_lights_data: Vec<u8>,
    pub triangles: Gemometry,
    pub move_nodes: Vec<(Index, AABB)>,
    // rem_nodes: HashSet<Index>,
    // add_nodes: Vec<(Index, AABB)>,
    assets: HashSet<String>,
    pub ui_compositors: HashMap<Index, UICompositor>,
    ui_render_args: HashMap<Index, UIRenderArgs>,
    pub textures: HashMap<Index, Texture>,
    // pub physics_system: PhysicsSystem,
    // pub window_draw_data: HashMap<Index, WindowDrawData>,
    mesh_pointers: HashMap<Index, MeshPointer>,
    mesh_nodes: HashMap<Index, Vec<Index>>,
    scene_collision_nodes: HashMap<Index, Vec<CollisionNode>>,
    scene_draw_calls: HashMap<Index, Vec<DrawCall>>,
    pub scene_point_lights: HashMap<Index, DirtyBuffer>,
    // scene_meshes: HashMap<Index, Vec<Index>>,
    pub scene_instance_buffers: HashMap<Index, DirtyBuffer>,
    scene_collections: HashMap<Index, SceneCollection>,
    pub buffers: Arena<Buffer>,
    point_lights: HashMap<Index, RawPointLight>,
}

impl EngineState {
    pub fn new() -> Self {
        Default::default()
    }

    fn process_nodes(&mut self) {
        self.printer.print(
            NODES_COUNT_SLOT,
            format!("nodes count: {}", self.state.nodes.len()),
        );
        let mut processed_nodes: HashSet<Index> = HashSet::new();
        for (_, nodes) in &mut self.mesh_nodes {
            nodes.clear();
        }

        for (node_id, node) in &self.state.nodes {
            if processed_nodes.contains(&node_id) {
                continue;
            }

            let mut stack = vec![node_id];

            while let Some(node_id) = stack.last() {
                let node_id = *node_id;

                let node = match self.state.nodes.get(node_id) {
                    Some(node) => node,
                    None => continue,
                };

                match node.parent {
                    NodeParent::Node(parent_node_id) => {
                        match processed_nodes.contains(&parent_node_id) {
                            true => {
                                let parent = match self.nodes.get(&parent_node_id) {
                                    Some(model) => model,
                                    None => {
                                        stack.push(parent_node_id);
                                        continue;
                                    }
                                };
                                let model = parent.model
                                    * glam::Mat4::from_translation(node.translation)
                                    * glam::Mat4::from_quat(node.rotation)
                                    * glam::Mat4::from_scale(node.scale);

                                let node_metadata = NodeMetadata {
                                    model,
                                    scene_id: parent.scene_id,
                                };

                                if let Some(collision_shape) = &node.collision_shape {
                                    match self.nodes.get(&node_id) {
                                        Some(old) => {
                                            if old.model != model {
                                                let collision_node = CollisionNode {
                                                    node_id,
                                                    aabb: collision_shape.aabb(node.translation),
                                                };

                                                self.scene_collision_nodes
                                                    .entry(parent.scene_id)
                                                    .or_insert(Vec::new())
                                                    .push(collision_node);
                                            }
                                        }
                                        None => {
                                            let collision_node = CollisionNode {
                                                node_id,
                                                aabb: collision_shape.aabb(node.translation),
                                            };

                                            self.scene_collision_nodes
                                                .entry(parent.scene_id)
                                                .or_insert(Vec::new())
                                                .push(collision_node);
                                        }
                                    }
                                }

                                self.nodes.insert(node_id, node_metadata);
                            }
                            false => {
                                stack.push(parent_node_id);
                                continue;
                            }
                        }
                    }
                    NodeParent::Scene(scene_id) => {
                        let model = glam::Mat4::from_translation(node.translation)
                            * glam::Mat4::from_quat(node.rotation)
                            * glam::Mat4::from_scale(node.scale);
                        let node = NodeMetadata { scene_id, model };
                        self.nodes.insert(node_id, node);
                    }
                    NodeParent::Orphan => {}
                }

                if let Some(mesh_id) = node.mesh {
                    self.mesh_nodes
                        .entry(mesh_id)
                        .or_insert(Vec::new())
                        .push(node_id);
                }

                stack.pop();
                processed_nodes.insert(node_id);
            }
        }

        // log::info!("mesh_nodes: {:?}", self.mesh_nodes);
        self.printer.print(
            MESH_NODES_SLOT,
            format!("mesh nodes: {:?}", self.mesh_nodes),
        );
    }

    fn process_meshes(&mut self) {
        // log::info!("meshes: {:?}", self.state.meshes.len());
        self.printer.print(
            MESHES_COUNT_SLOT,
            format!("meshes count: {}", self.state.meshes.len()),
        );
        self.triangles.vertices.reset_offset();
        self.triangles.normals.reset_offset();
        self.triangles.tex_coords.reset_offset();
        self.triangles.indices.reset_offset();
        for (_, s) in &mut self.scene_instance_buffers {
            s.reset_offset();
        }
        for (_, s) in &mut self.scene_draw_calls {
            s.clear();
        }
        for (mesh_id, mesh) in &self.state.meshes {
            for primitive in &mesh.primitives {
                if primitive.topology == PrimitiveTopology::TriangleList {
                    if primitive.vertices.len() == 0 || primitive.indices.len() == 0 {
                        continue;
                    }

                    let vertices_start = self.triangles.vertices.len() as u64;
                    self.triangles
                        .vertices
                        .extend_from_slice(bytemuck::cast_slice(&primitive.vertices));
                    let vertices_end = self.triangles.vertices.len() as u64;
                    let normals_start = self.triangles.normals.len() as u64;
                    self.triangles
                        .normals
                        .extend_from_slice(bytemuck::cast_slice(&primitive.normals));
                    let normals_end = self.triangles.normals.len() as u64;
                    let indices_start = self.triangles.indices.len() as u64;
                    self.triangles
                        .indices
                        .extend_from_slice(bytemuck::cast_slice(&primitive.indices));
                    let indices_end = self.triangles.indices.len() as u64;
                    let tex_coords_start = self.triangles.tex_coords.len() as u64;
                    if primitive.tex_coords.len() > 0 {
                        self.triangles
                            .tex_coords
                            .extend_from_slice(bytemuck::cast_slice(&primitive.tex_coords));
                    } else {
                        let tex_coords = vec![[0.0, 0.0]; primitive.vertices.len()];
                        self.triangles
                            .tex_coords
                            .extend_from_slice(bytemuck::cast_slice(&tex_coords));
                    }
                    let tex_coords_end = self.triangles.tex_coords.len() as u64;

                    let pointer = MeshPointer {
                        positions: vertices_start..vertices_end,
                        normals: normals_start..normals_end,
                        tex_coords: tex_coords_start..tex_coords_end,
                        indices: indices_start..indices_end,
                        indice_count: primitive.indices.len() as u32,
                    };
                    self.mesh_pointers.insert(mesh_id, pointer);

                    let node_ids = match self.mesh_nodes.get(&mesh_id) {
                        Some(nodes) => nodes,
                        None => continue,
                    };

                    let mut checkpoints: HashMap<Index, Range<u32>> = HashMap::new();

                    for node_id in node_ids {
                        let node = match self.nodes.get(node_id) {
                            Some(node) => node,
                            None => continue,
                        };

                        let instance = RawInstance {
                            model: node.model.to_cols_array_2d(),
                        };

                        let buffer = self
                            .scene_instance_buffers
                            .entry(node.scene_id)
                            .or_insert(DirtyBuffer::new("instances"));
                        let instance_start = buffer.len() as u32;
                        buffer.extend_from_slice(bytemuck::bytes_of(&instance));
                        let instance_end = buffer.len() as u32;

                        let checkpoint = checkpoints
                            .entry(node.scene_id)
                            .or_insert(instance_start..instance_end);
                        checkpoint.end = instance_end;
                    }

                    for (scene_id, instances) in checkpoints {
                        let draw_calls =
                            self.scene_draw_calls.entry(scene_id).or_insert(Vec::new());

                        // log::info!("draw_calls: {:?}", draw_calls.len());

                        draw_calls.push(DrawCall {
                            texture: mesh.texture,
                            vertices: vertices_start..vertices_end,
                            indices: indices_start..indices_end,
                            normals: normals_start..normals_end,
                            tex_coords: tex_coords_start..tex_coords_end,
                            instances,
                            indices_range: 0..primitive.indices.len() as u32,
                        });
                    }
                }
            }
        }

        // log::info!("scene_draw_calls: {:?}", self.scene_draw_calls.len());
        self.printer.print(
            DRAW_CALLS_SLOT,
            format!("scene draw calls: {:?}", self.scene_draw_calls),
        );
    }

    fn process_cameras(&mut self) {
        for (_, buf) in &mut self.camera_buffers {
            buf.reset_offset();
        }

        for (cam_id, cam) in &self.state.cameras {
            let cam_node = match cam.node_id {
                Some(id) => match self.nodes.get(&id) {
                    Some(node) => node,
                    None => continue,
                },
                None => continue,
            };

            let model = glam::Mat4::perspective_lh(cam.fovy, cam.aspect, cam.znear, cam.zfar)
                * cam_node.model;

            let cam = RawCamera {
                model: model.to_cols_array_2d(),
            };

            match self.cameras.get(&cam_id) {
                Some(camera) => {
                    self.cameras.insert(cam_id, *camera);
                }
                None => {
                    log::info!("can_node: {:?}", cam_node);
                    log::info!("model: {:?}", model);
                    log::info!("new camera cam_id: {:?} camera: {:?}", cam_id, cam);
                    self.cameras.insert(cam_id, cam);
                }
            }

            let buffer = self
                .camera_buffers
                .entry(cam_id)
                .or_insert(DirtyBuffer::new("cameras"));

            buffer.extend_from_slice(bytemuck::bytes_of(&cam));
        }

        self.printer
            .print(CAMERAS_SLOT, format!("cameras: {:?}", self.cameras));
    }

    fn process_point_lights(&mut self) {
        for (_, s) in &mut self.scene_point_lights {
            s.reset_offset();
        }

        for (light_id, light) in &self.state.point_lights {
        	let node = match light.node_id {
        		Some(id) => {
        			match self.nodes.get(&id) {
        				Some(node) => node,
        				None => continue,
        			}
        		}
        		None => continue,
        	};

        	// let light = RawPointLight {
        	// 	color: light.color.into(),
        	// 	intensity: light.intensity,
        	// 	position: node.model.w_axis.truncate().into(),

        	// };

			let light = RawPointLight::new(light.color, light.intensity, node.model.w_axis.truncate().into());

        	match self.point_lights.get(&light_id) {
        		Some(old_light) => {
        			if old_light != &light {
        				//log::info!("point light modified {:?} {:?}", light_id, light);
        				self.point_lights.insert(light_id, light);
        			}
        		},
        		None => {
        			log::info!("new point light {:?} {:?}", light_id, light);
        			self.point_lights.insert(light_id, light);
        		}
        	}

        	self.scene_point_lights.entry(node.scene_id).or_insert(DirtyBuffer::new("pointlight")).extend_from_slice(bytes_of(&light));
        }
    }

    fn process_scenes(&mut self) {
        for (scene_id, scene) in &self.state.scenes {
            self.grids.entry(scene_id).or_insert(SpatialGrid::new(5.0));
        }
    }

    fn process_assets(&mut self) {
        let paths = self
            .state
            .assets_3d
            .iter()
            .map(|p| p.1.path.clone())
            .collect::<Vec<String>>();

        for path in paths {
            if self.assets.contains(&path) {
                continue;
            }

            self.assets.insert(path.clone());

            load_gltf(&path, &mut self.state);
        }
    }

    // pub fn update_guis(&mut self) {
    // 	for (gui_id, gui) in &self.state.guis {
    // 		let compositor = self.ui_compositors.entry(gui_id).or_insert(UICompositor::new());draw_calls
    // 		compositor.process(gui);
    // 	}
    // }

    fn process_ui(&mut self) {
        for (ui_id, gui) in &self.state.guis {
            let compositor = self
                .ui_compositors
                .entry(ui_id)
                .or_insert(UICompositor::new());
            compositor.process(gui);

            let render_args = self.ui_render_args.entry(ui_id).or_insert(UIRenderArgs {
                ui: ui_id,
                views: Vec::new(),
            });

            render_args.views.clear();

            for view in &compositor.views {
                let camera = match self.state.cameras.get(view.camera_id) {
                    Some(camera) => camera,
                    None => continue,
                };

                let camera_node = match camera.node_id {
                    Some(node_id) => match self.nodes.get(&node_id) {
                        Some(node) => node,
                        None => continue,
                    },
                    None => continue,
                };

                render_args.views.push(View {
                    camview: view.clone(),
                    scene_id: camera_node.scene_id,
                });
            }
        }

        self.printer.print(
            UI_RENDER_ARGS_SLOT,
            format!("ui_render_args: {:?}", self.ui_render_args),
        );
    }

    fn process_phycis(&mut self, dt: f32) {
        for (_, c) in &mut self.scene_collections {
            let timings = c
                .physics_system
                .physics_update(&mut self.state, &mut c.grid, dt);
            if timings.node_update_time > 3 {
                self.printer.print(
                    NODE_UPDATE_TIME_SLOT,
                    format!("node_update_time: {}", timings.node_update_time),
                );
            }
            if timings.broad_phase_time > 10 {
                self.printer.print(
                    BROAD_PHASE_TIME_SLOT,
                    format!("broad_phase_time: {}", timings.broad_phase_time),
                );
            }
            if timings.narrow_phase_time > 3 {
                self.printer.print(
                    NARROW_PHASE_TIME_SLOT,
                    format!("narrow_phase_time: {}", timings.narrow_phase_time),
                );
            }
            if timings.resolve_collision_time > 0 {
                self.printer.print(
                    NARROW_PHASE_TIME_SLOT,
                    format!("resolve_collision_time: {}", timings.resolve_collision_time),
                );
            }

            for (_, ray_cast) in &mut self.state.raycasts {
                ray_cast.intersects.clear();

                let node = match self.state.nodes.get(ray_cast.node_inx) {
                    Some(node) => node,
                    None => continue,
                };

                let start = node.translation;
                let end = start + node.rotation * Vec3::new(0.0, 0.0, 1.0) * ray_cast.len;
                let nodes = c.grid.get_line_ray_nodes(start, end);

                let mut intersections = Vec::new();

                for node_inx in nodes {
                    if node_inx == ray_cast.node_inx {
                        continue;
                    }

                    let aabb = match c.grid.get_node_rect(node_inx) {
                        Some(aabb) => aabb,
                        None => continue,
                    };

                    if let Some((tmin, _tmax)) = aabb.intersect_ray(start, end) {
                        intersections.push((tmin, node_inx));
                    }
                }

                // Sort the intersections by tmin
                intersections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                // Store the sorted node indices
                ray_cast.intersects = intersections
                    .into_iter()
                    .map(|(_, node_inx)| node_inx)
                    .collect();
            }
        }
    }

    pub fn process(&mut self, dt: f32) {
        let timer = Instant::now();
        self.all_instances_data.clear();
        // self.all_positions_data.clear();
        // self.all_tex_coords_data.clear();
        // self.all_normals_data.clear();
        // self.all_indices_data.clear();
        self.all_point_lights_data.clear();

        self.process_nodes();
        self.process_meshes();
        self.process_cameras();
        self.process_point_lights();
        self.process_ui();
        self.process_scenes();
        self.process_phycis(dt);
    }

    pub fn get_window_render_args(&self, window_id: Index) -> Option<&UIRenderArgs> {
        let window = match self.ui_render_args.get(&window_id) {
            Some(window) => window,
            None => return None,
        };

        let ui_id = window.ui;

        self.ui_render_args.get(&ui_id)
    }

    pub fn get_camera_draw_calls(&self, camera_id: Index) -> Option<&Vec<DrawCall>> {
        let camera = self.state.cameras.get(camera_id)?;
        let scene_id = match camera.node_id {
            Some(node_id) => {
                let node = self.nodes.get(&node_id)?;
                node.scene_id
            }
            None => return None,
        };
        self.scene_draw_calls.get(&scene_id)
    }
}

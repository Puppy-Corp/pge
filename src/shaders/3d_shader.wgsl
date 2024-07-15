struct NodeTransform {
    model: mat4x4<f32>,
    parent_index: i32
};

@group(1) @binding(0)
var<storage, read_write> node_transforms: array<NodeTransform>;

struct InstanceInput {
    @location(5) node_inx: i32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	// @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
	@location(1) world_position: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct Camera {
    proj: mat4x4<f32>,
	node_inx: i32
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct PointLight {
	color: vec3<f32>,
	intensity: f32,
	node_inx: i32,
};

@group(2) @binding(0)
var<storage, read> point_lights: array<PointLight>;

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    var cube_transform = node_transforms[instance.node_inx].model;
	var view_proj_matrix = camera.proj;

    var out: VertexOutput;
    let world_position = (cube_transform * vec4<f32>(input.position, 1.0)).xyz;
    out.clip_position = view_proj_matrix * vec4<f32>(world_position, 1.0);
    out.color = vec3(1.0, 0.0, 0.0); // Placeholder for color, to be modified by lighting calculation
    out.world_position = world_position;
	let normal = input.normal;
	out.normal = normal;
    return out;
}

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	let light_color = vec3<f32>(1.0, 1.0, 1.0);
	var result = vec3<f32>(0.0, 0.0, 0.0);

	for (var i = 0u; i < 2; i = i + 1u) {
		let point_light = point_lights[i];
		let point_light_model = node_transforms[point_light.node_inx].model;
		let light_position = (point_light_model * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
		let light_direction = normalize(light_position - in.world_position);

		let diffuse_strength = max(dot(in.normal, light_direction), 0.0);
		let diffuse_color = light_color * diffuse_strength;
		result = result + diffuse_color * in.color;
	}
	return vec4<f32>(result, 1.0);
}
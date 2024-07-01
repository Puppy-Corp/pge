struct NodeTransform {
    model: mat4x4<f32>,
    parent_index: i32
};

@group(1) @binding(0)
var<storage, read_write> node_transforms: array<NodeTransform>;

struct InstanceInput {
    @location(5) node_index: i32,
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
@group(0) @binding(1)
var<uniform> camera: Camera;

struct PointLight {
	color: vec3<f32>,
	intensity: f32,
	node_inx: i32,
};

@group(2) @binding(0)
var<storage, read> point_lights: array<PointLight>;

fn get_cumulative_transform(node_index: i32) -> mat4x4<f32> {
    var cumulative_transform = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    var current_index = node_index;
    while (current_index != -1) {
        let current_node = node_transforms[current_index];
        cumulative_transform = current_node.model * cumulative_transform;
        current_index = current_node.parent_index;
    }
    return cumulative_transform;
}

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    var camera_transform = get_cumulative_transform(camera.node_inx);
    // var cube_transform = get_cumulative_transform(instance.node_index);
	var cube_transform = node_transforms[instance.node_index].model;
	// var cube_transform = mat4x4<f32>(
    //     vec4<f32>(1.0, 0.0, 0.0, 0.0),
    //     vec4<f32>(0.0, 1.0, 0.0, 0.0),
    //     vec4<f32>(0.0, 0.0, 1.0, 0.0),
    //     vec4<f32>(0.0, 0.0, 0.0, 1.0)
    // );

    var out: VertexOutput;
    let world_position = (cube_transform * vec4<f32>(input.position, 1.0)).xyz;
    out.clip_position = camera.proj * camera_transform * vec4<f32>(world_position, 1.0);
    out.color = vec3(1.0, 0.0, 0.0); // Placeholder for color, to be modified by lighting calculation
    out.world_position = world_position;
	let normal = input.normal;
	out.normal = normal;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(in.color, 1.0);

	// let light_color = vec3<f32>(1.0, 1.0, 1.0);
	// // let ambient_strength = 0.0;
	// // let ambient_color = light_color * ambient_strength;
	// var result = vec3<f32>(0.0, 0.0, 0.0);

	// for (var i = 0u; i < 2; i = i + 1u) {
	// 	let point_light = point_lights[i];
	// 	let light_position = (get_cumulative_transform(point_light.node_inx) * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
	// 	let light_direction = normalize(light_position - in.world_position);

	// 	let diffuse_strength = max(dot(in.normal, light_direction), 0.0);
	// 	let diffuse_color = light_color * diffuse_strength;
	// 	result = result + diffuse_color * in.color;
	// }
	// return vec4<f32>(result, 1.0);




    // var ambient_color = vec3<f32>(0.1, 0.1, 0.1); // Ambient light color
    // var diffuse_color = vec3<f32>(0.0, 0.0, 0.0); // Base color of the object

    // // Initialize the resulting color with ambient light
    // var result_color = diffuse_color;

    // //for (var i = 0u; i < arrayLength(&point_lights); i = i + 1u) {
    //     let point_light = point_lights[0];
    //     let light_position = (get_cumulative_transform(point_light.node_inx) * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
	// 	//let light_position = vec3<f32>(1.5, 0.0, 0.0);
    //     let light_direction = normalize(light_position - in.world_position);
    //     // let normal = normalize(in.normal);

	// 	let light_color = vec3<f32>(1.0, 1.0, 1.0);

    //     // Diffuse shading
    //     let diffuse_intensity = max(dot(in.normal, light_direction), 0.0);
    //     let diffuse = light_color * 1.0;

    //     // Add the diffuse component to the result color
    //     // result_color = result_color + diffuse * diffuse_color;
    // //}

    // return vec4<f32>(result_color, 1.0);
}
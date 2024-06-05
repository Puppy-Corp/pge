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
	@location(2) uv: vec2<f32>,
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
    var cube_transform = get_cumulative_transform(instance.node_index);

    var out: VertexOutput;
    let world_position = (cube_transform * vec4<f32>(input.position, 1.0)).xyz;
    out.clip_position = camera.proj * camera_transform * vec4<f32>(world_position, 1.0);
    out.color = vec3(1.0, 0.0, 0.0); // Placeholder for color, to be modified by lighting calculation
    out.world_position = world_position;
	// let normal = vec3(0.0, 0.0, 1.0);
	//let normal = input.normal;
	// let normal_matrix = transpose(inverse(mat3x3<f32>(
    //     cube_transform[0].xyz,
    //     cube_transform[1].xyz,
    //     cube_transform[2].xyz
    // )));
    // out.normal = normalize(normal_matrix * input.normal);
    // out.normal = normalize((cube_transform * vec4<f32>(normal, 0.0)).xyz);
	    // Simplified normal transformation assuming orthogonal matrix
    let normal_matrix = transpose(mat3x3<f32>(
        cube_transform[0].xyz,
        cube_transform[1].xyz,
        cube_transform[2].xyz
    ));
    out.normal = normalize(normal_matrix * input.normal);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ambient_color = vec3<f32>(0.1, 0.1, 0.1); // Ambient light color
    var diffuse_color = vec3<f32>(1.0, 0.0, 0.0); // Base color of the object

    // Initialize the resulting color with ambient light
    var result_color = ambient_color * diffuse_color;

    // for (var i = 0u; i < arrayLength(&point_lights); i = i + 1u) {
    //     let point_light = point_lights[i];
    //     let light_position = (get_cumulative_transform(point_light.node_inx) * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
    //     let light_direction = normalize(light_position - in.world_position);
    //     let normal = normalize(in.normal);

    //     // Diffuse shading
    //     let diffuse_intensity = max(dot(normal, light_direction), 0.0);
    //     let diffuse = diffuse_intensity * point_light.color * point_light.intensity;

    //     // Add the diffuse component to the result color
    //     result_color = result_color + diffuse * diffuse_color;
    // }

	let point_light = point_lights[0];
	let node_inx = point_light.node_inx;
	let light_mat = get_cumulative_transform(node_inx);
	let light_position = (get_cumulative_transform(node_inx) * vec4<f32>(0.0, 0.0, 0.0, 1.0)).xyz;
	//let light_position = vec3<f32>(2.0, 0.0, 2.0);
	let light_direction = normalize(light_position - in.world_position);
	let normal = normalize(in.normal);

	let light_color = point_light.color;
	let light_intensity = point_light.intensity;
	//let light_intensity = 1.0;
    //let light_color = vec3<f32>(1.0, 1.0, 1.0);

	// Diffuse shading
	let diffuse_intensity = max(dot(normal, light_direction), 0.0);
	let diffuse = diffuse_intensity * light_color * light_intensity;

	// Add the diffuse component to the result color
	result_color = result_color + diffuse * diffuse_color;

	    // Hardcoded point light properties
    // let light_intensity = 1.0;
    // let light_color = vec3<f32>(1.0, 1.0, 1.0);
    // let light_position = vec3<f32>(0.0, 0.0, -2.0);

    // // Calculate the light direction and normalize the normal
    // let light_direction = normalize(light_position - in.world_position);
    // let normal = normalize(in.normal);

    // // Diffuse shading
    // let diffuse_intensity = max(dot(normal, light_direction), 0.0);
    // let diffuse = diffuse_intensity * light_color * light_intensity;

    // // Add the diffuse component to the result color
    // result_color = result_color + diffuse * diffuse_color;

    return vec4<f32>(result_color, 1.0);
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
	@location(1) world_position: vec3<f32>,
    @location(2) normal: vec3<f32>,
	@location(3) tex_coords: vec2<f32>,
};

struct Camera {
    model: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: Camera;

struct PointLight {
	color: vec3<f32>,
	intensity: f32,
	position: vec3<f32>,
};

@group(1) @binding(0)
var<storage, read> point_lights: array<PointLight>;

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
	var out: VertexOutput;
	out.clip_position = camera.model * vec4<f32>(input.position, 1.0);
	out.color = vec3<f32>(1.0, 0.0, 0.0);
	// out.world_position = input.position;
	// out.normal = input.normal;
	// out.tex_coords = input.tex_coords;
	return out;


	// let instace_model = mat4x4<f32>(
    //     instance.model_matrix_0,
    //     instance.model_matrix_1,
    //     instance.model_matrix_2,
    //     instance.model_matrix_3,
    // );

    // var out: VertexOutput;
    // let world_position = (vec4<f32>(input.position, 1.0)).xyz;
    // out.clip_position = camera.model * vec4<f32>(world_position, 1.0);
    // out.color = vec3(1.0, 0.0, 0.0); // Placeholder for color, to be modified by lighting calculation
    // out.world_position = world_position;
	// let normal = input.normal;
	// out.normal = normal;
	// out.tex_coords = input.tex_coords;
    // return out;
}

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 1.0, 0.0, 1.0);

    // let light_color = vec3<f32>(1.0, 1.0, 1.0);
    // var result = vec3<f32>(0.0, 0.0, 0.0);

    // // Sample the texture at the given texture coordinates
    // let texture_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    // for (var i = 0u; i < 2; i = i + 1u) {
    //     let point_light = point_lights[i];
    //     let light_position = point_light.position;
    //     let light_direction = normalize(light_position - in.world_position);
        
    //     let diffuse_strength = max(dot(in.normal, light_direction), 0.0);
    //     let diffuse_color = light_color * diffuse_strength;

    //     // Multiply diffuse color with the texture color and accumulate to the result
    //     result = result + (diffuse_color * texture_color.rgb);
    // }

    // return vec4<f32>(result, 1.0);
}
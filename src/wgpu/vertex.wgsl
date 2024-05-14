// Uniform buffer for the camera
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

// Uniform buffer for the model matrix
struct ModelUniform {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
};

// Input attributes from vertex buffer
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

// Output structure to the fragment shader
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) frag_position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

// Bind groups for uniform buffers
@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(0) @binding(1) var<uniform> model: ModelUniform;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let position_world = model.model * vec4<f32>(vertex.position, 1.0);
    output.position = camera.view_proj * position_world;
    output.frag_position = position_world.xyz;

    // Transform normal with the normal matrix
    output.normal = normalize((model.normal_matrix * vec4<f32>(vertex.normal, 0.0)).xyz);

    output.tex_coords = vertex.tex_coords;
    return output;
}

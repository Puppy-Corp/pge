// // @vertex
// // fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
// //     let x = f32(i32(in_vertex_index) - 1);
// //     let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
// //     return vec4<f32>(x, y, 0.0, 1.0);
// // }

// // @fragment
// // fn fs_main(@builtin(position) coords: vec4<f32>) -> @location(0) vec4<f32> {
// //     var color: vec4<f32>;

// //     let screen_width = 1200;
// //     let screen_height = 800;

// //     var normalized_x = coords.x / f32(screen_width);

// //     return vec4<f32>(normalized_x, normalized_x, normalized_x, 1.0);
// // }

// // struct NodeTransform {
// //     model: mat4x4<f32>,
// //     parent_index: i32
// // };

// // @group(0) @binding(0)
// // var<storage, read> node_transforms: array<NodeTransform>;

// // struct Camera {
// //     view_proj: mat4x4<f32>,
// // }
// // @group(0) @binding(0)
// // var<uniform> camera: Camera;

// struct VertexInput {
//     @location(0) position: vec3<f32>,
//     // @location(1) normal: vec3<f32>,
// 	// @location(2) text_coords: vec2<f32>,
// };
// struct InstanceInput {
//     @location(5) node_index: u32,
// }

// struct VertexOutput {
//     @builtin(position) clip_position: vec4<f32>,
//     @location(0) color: vec3<f32>,
// };

// @vertex
// fn vs_main(
//     model: VertexInput,
//     instance: InstanceInput,
// ) -> VertexOutput {
//     // let model_matrix = mat4x4<f32>(
//     //     instance.model_matrix_0,
//     //     instance.model_matrix_1,
//     //     instance.model_matrix_2,
//     //     instance.model_matrix_3,
//     // );
//     var out: VertexOutput;
//     out.color = vec3(1.0, 1.0, 0.0);
//     // //out.clip_position = vec4<f32>(model.position, 1.0);
//     // out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
//     return out;
// }

// // Fragment shader

// @fragment
// fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
//     return vec4<f32>(in.color, 1.0);
// }

// struct NodeTransform {
//     model: mat4x4<f32>,
//     parent_index: i32
// };

// @group(1) @binding(0)
// var<storage, read> node_transforms: array<NodeTransform>;

struct InstanceInput {
    @location(5) node_index: u32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct Camera {
    view: mat4x4<f32>,
	proj: mat4x4<f32>,
}
@group(0) @binding(1)
var<uniform> camera: Camera;

// @vertex
// fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
// 	var model = node_transforms[0].model;

// 	let model_matrix = mat4x4<f32>(
// 		1.0, 0.0, 0.0, -0.5,
// 		0.0, 1.0, 0.0, -5.0,
// 		0.0, 0.0, 1.0, 0.0,
// 		0.0, 0.0, 0.0, 1.0
// 	);

//     var out: VertexOutput;
//     out.clip_position = model_matrix * vec4<f32>(input.position, 1.0);
//     out.color = vec3(1.0, 1.0, 0.0); // Fixed yellow color
//     return out;
// }

// @vertex
// fn vs_main(input: VertexInput) -> VertexOutput {
//     // Define a simple translation matrix
//     let model_matrix = mat4x4<f32>(
//         1.0, 0.0, 0.0, 0.0,  // No translation on x-axis
//         0.0, 1.0, 0.0, 0.0,  // No translation on y-axis
//         0.0, 0.0, 1.0, 0.0,  // No translation on z-axis
//         0.0, 0.0, 0.0, 1.0
//     );

//     // Initialize the output structure
//     var out: VertexOutput;

//     // Apply the model matrix to the input position
//     out.clip_position = camera.proj * camera.view * model_matrix * vec4<f32>(input.position, 1.0);

//     // Set the output color to yellow
//     out.color = vec3(1.0, 1.0, 0.0);

//     return out;
// }

// @vertex
// fn vs_main(input: VertexInput) -> VertexOutput {
//     // Hardcoded view matrix
//     let view: mat4x4<f32> = mat4x4<f32>(
//         vec4<f32>(0.7071, 0.0, -0.7071, 0.0),
//         vec4<f32>(-0.4082, 0.8165, -0.4082, 0.0),
//         vec4<f32>(0.5774, 0.5774, 0.5774, -3.4641),
//         vec4<f32>(0.0, 0.0, 0.0, 1.0)
//     );

//     // Hardcoded projection matrix
//     let proj: mat4x4<f32> = mat4x4<f32>(
//         vec4<f32>(1.2990, 0.0, 0.0, 0.0),
//         vec4<f32>(0.0, 2.3094, 0.0, 0.0),
//         vec4<f32>(0.0, 0.0, -1.001, -0.2001),
//         vec4<f32>(0.0, 0.0, -1.0, 0.0)
//     );

//     // Define a simple model matrix (identity in this case)
//     let model_matrix = mat4x4<f32>(
//         1.0, 0.0, 0.0, 0.0,
//         0.0, 1.0, 0.0, 0.0,
//         0.0, 0.0, 1.0, 0.0,
//         0.0, 0.0, 0.0, 1.0
//     );

//     // Initialize the output structure
//     var out: VertexOutput;

//     // Apply the model, view, and projection matrices to the input position
//     out.clip_position = proj * view * model_matrix * vec4<f32>(input.position, 1.0);

//     // Set the output color to yellow
//     out.color = vec3(1.0, 1.0, 0.0);

//     return out;
// }

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    // Model matrix (identity in this simple case)
    let model_matrix = mat4x4<f32>(
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    // Initialize the output structure
    var out: VertexOutput;
    // Apply the model, view, and projection matrices to the input position
    out.clip_position = camera.proj * camera.view * model_matrix * vec4<f32>(input.position, 1.0);
    // Set the output color
    out.color = vec3(1.0, 1.0, 0.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
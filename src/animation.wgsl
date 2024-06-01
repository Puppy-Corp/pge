struct Keyframe {
    //time: f32,
    value: mat4x4<f32>,
    // start_time: f32,
    // repeat: u32,
    // animation_id: u32,
    is_running: u32,
    //node_id: u32
};

struct NodeTransform {
    model: mat4x4<f32>,
    parent_index: i32
};

@group(0) @binding(0) 
var<storage, read> keyframes: array<Keyframe>;
@group(0) @binding(1)
var<uniform> current_time: f32;
@group(1) @binding(0)
var<storage, read_write> node_transforms: array<NodeTransform>;

// Linear Interpolation Function for mat4x4
fn mat4_lerp(a: mat4x4<f32>, b: mat4x4<f32>, t: f32) -> mat4x4<f32> {
    return mat4x4<f32>(
        mix(a[0], b[0], t),
        mix(a[1], b[1], t),
        mix(a[2], b[2], t),
        mix(a[3], b[3], t)
    );
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;

    if index >= arrayLength(&node_transforms) {
        return;
    }

	// var i = 0u;
	// while (i < arrayLength(&keyframes) - 1u && keyframes[i + 1u].start_time <= current_time) {
	// 	i = i + 1u;
	// }
	let keyframe = keyframes[0];

	if keyframe.is_running == 0u {
		return;
	}

	// if keyframe.node_id != index {
	// 	return;
	// }

	// let v = keyframe.value;
	node_transforms[index].model *= keyframe.value;

    // var i = 0u;
    // while (i < arrayLength(&keyframes) - 1u && keyframes[i + 1u].start_time <= current_time) {
    //     i = i + 1u;
    // }

    // let kf1 = keyframes[i];
    // let kf2 = keyframes[i + 1u];

    // if kf1.is_running == 0u || kf1.node_id != index {
    //     // If the animation is not running or the keyframe does not belong to this node, skip
    //     return;
    // }

    // // Adjust current time relative to the start time of the animation
    // let relative_time = current_time - kf1.start_time;

    // // Handle repeating logic
    // var adjusted_time = relative_time;
    // if (kf1.repeat == 1u) {
    //     let duration = kf2.time - kf1.time;
    //     adjusted_time = relative_time % duration;
    // }

    // if (relative_time < 0.0) {
    //     // Animation hasn't started yet
    //     return;
    // }

    // // Calculate the interpolation factor
    // let t = (adjusted_time - kf1.time) / (kf2.time - kf1.time);

    // // Interpolate between the keyframes
    // let interpolated_value = mat4_lerp(kf1.value, kf2.value, t);

    // // Apply the interpolated value to the node transformation
    // node_transforms[index].model = interpolated_value;
}

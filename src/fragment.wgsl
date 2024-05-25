struct PointLight {
    position: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    range: f32,
};

struct SpotLight {
    position: vec3<f32>,
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
    range: f32,
    inner_cone_angle: f32,
    outer_cone_angle: f32,
};

struct DirectionalLight {
    direction: vec3<f32>,
    intensity: f32,
    color: vec3<f32>,
};

struct AmbientLight {
    color: vec3<f32>,
    intensity: f32,
};

struct LightingUniform {
    ambient_light: AmbientLight,
    num_point_lights: u32,
    num_spot_lights: u32,
    num_directional_lights: u32,
    padding: u32,
    point_lights: array<PointLight, 10>,
    spot_lights: array<SpotLight, 5>,
    directional_lights: array<DirectionalLight, 2>,
};

@group(0) @binding(2) var<uniform> lighting: LightingUniform;

@group(0) @binding(3) var my_texture: texture_2d<f32>;
@group(0) @binding(4) var my_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) normal: vec3<f32>,
    @location(1) frag_position: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
};

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let norm = normalize(input.normal);
    var color = lighting.ambient_light.color * lighting.ambient_light.intensity;

    // Point lights
    for (var i = 0u; i < lighting.num_point_lights; i = i + 1u) {
        let light = lighting.point_lights[i];
        let light_dir = normalize(light.position - input.frag_position);
        let distance = length(light.position - input.frag_position);
        let attenuation = max(0.0, 1.0 - (distance / light.range));
        let diff = max(dot(norm, light_dir), 0.0);
        let diffuse = diff * light.intensity * light.color * attenuation;
        color = color + diffuse;
    }

    // Spot lights
    for (var i = 0u; i < lighting.num_spot_lights; i = i + 1u) {
        let light = lighting.spot_lights[i];
        let light_dir = normalize(light.position - input.frag_position);
        let distance = length(light.position - input.frag_position);
        let attenuation = max(0.0, 1.0 - (distance / light.range));
        let diff = max(dot(norm, light_dir), 0.0);

        // Spot cone angle calculation
        let spot_effect = dot(normalize(light.direction), -light_dir);
        let spot_factor = smoothstep(light.outer_cone_angle, light.inner_cone_angle, spot_effect);
        let diffuse = diff * light.intensity * light.color * attenuation * spot_factor;
        color = color + diffuse;
    }

    // Directional lights
    for (var i = 0u; i < lighting.num_directional_lights; i = i + 1u) {
        let light = lighting.directional_lights[i];
        let light_dir = normalize(-light.direction);
        let diff = max(dot(norm, light_dir), 0.0);
        let diffuse = diff * light.intensity * light.color;
        color = color + diffuse;
    }

    let base_color = textureSample(my_texture, my_sampler, input.tex_coords).rgb;
    return vec4<f32>(base_color * color, 1.0);
}

// Vertex shader

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;


struct Light {
    position: vec3<f32>, // xyz + padding
    color: vec3<f32>,  // rgb + padding
};

struct LightBlock {
    lights: array<Light, 16>,
    light_count: u32,
};

@group(1) @binding(0)
var<uniform> u_lights: LightBlock;

@group(2) @binding(0)
var life_tex: texture_2d<f32>;

@group(2) @binding(1)
var life_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) quad_id: u32,
}
struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
    @location(9) instance_color: vec3<f32>,
    @location(10) normal_matrix_0: vec3<f32>,
    @location(11) normal_matrix_1: vec3<f32>,
    @location(12) normal_matrix_2: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1)  world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) quad_id: u32,
}

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );
    var out: VertexOutput;
    out.color = vec3<f32>(instance.instance_color.x, instance.instance_color.y, instance.instance_color.z);
    out.world_normal = normalize(normal_matrix * model.normal); 

    var world_position: vec4<f32> = model_matrix * vec4<f32>(model.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    out.quad_id = model.quad_id;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let ambient_strength = 0.1;
    let shininess = 32.0;

    var result: vec3<f32> = vec3<f32>(0.0);

    let N = normalize(in.world_normal);
    let V = normalize(camera.view_pos.xyz - in.world_position);

    for (var i: u32 = 0u; i < u_lights.light_count; i = i + 1u) {
        let light = u_lights.lights[i];

        let L = normalize(light.position.xyz - in.world_position);
        let H = normalize(V + L);

        // Ambient
        let ambient = light.color.xyz * ambient_strength;

        // Diffuse
        let diff = max(dot(N, L), 0.0);
        let diffuse = diff * light.color.xyz;

        // Specular (Blinn–Phong)
        let spec = pow(max(dot(N, H), 0.0), shininess);
        let specular = spec * light.color.xyz;

        result += ambient + diffuse + specular;
    }
    let grid_width : u32 = 100u - 1u;
    let grid_height : u32 = 600u * 2u;

    let quad_id = in.quad_id;

    let x = quad_id % grid_width;
    let y = quad_id / grid_width;

    let uv = vec2<f32>(
        (f32(x) + 0.5) / f32(grid_width),
        (f32(y) + 0.5) / f32(grid_height)
    );

    let alive = textureSample(life_tex, life_sampler, uv).r;


    result *= select(
        vec3<f32>(0.05),
        vec3<f32>(1.0),
        alive > 0.5
    );

    return vec4<f32>(result, 1.0);
}

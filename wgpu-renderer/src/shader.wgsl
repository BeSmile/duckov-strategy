enable f16;  // 必须在最顶部

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) tex_coords: vec2<f32>,
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec4<f32>,
    @location(1) tangent: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
}

struct CameraUniform{
    view_proj: mat4x4<f32>,
    view_position: vec3<f32>
}

struct SceneUniforms{
    ambient_light: vec3<f32>,
    ambient_intensity: f32,
    fog_color: vec3<f32>,
    fog_density: f32,
}

@group(0) @binding(0)
var<uniform> camera:CameraUniform;

@group(1) @binding(0)
var<uniform> scene:SceneUniforms;


@vertex
fn vs_main(input: VertexInput) -> VertexOutput{
    var out: VertexOutput;
    var position  = vec4<f32>(input.position, 1.0);
    out.normal = input.normal;

    out.clip_position = camera.view_proj * position;
//    out.tangent = input.tangent;
//    out.tex_coords = input.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var ba_color = vec4(0.3, 0.4, 0.5,1.0);

//    var ambient =  scene.ambient_light * scene.ambient_intensity * ba_color.rgb;

//    return vec4(ambient, 1.0);
    return  vec4(0.1, 0.2,0.3,1.0);
}

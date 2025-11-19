enable f16;  // 必须在最顶部

struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) tangent: vec4<f32>,
    //   @location(3) color: vec4<f32>,
    @location(4) uv0: vec2<f32>,
//   @location(5) uv1: vec2<f32>,
}

struct VertexOutput{
    @builtin(position) clip_position: vec4<f32>,
    @location(0) normal: vec4<f32>,
    @location(1) tangent: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(4) uv0: vec2<f32>,  // ✅ 添加这一行

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

struct TransformUniform{
    orbit_proj: mat4x4<f32>
}

@group(0) @binding(0)
var<uniform> camera:CameraUniform;

@group(1) @binding(0)
var<uniform> scene:SceneUniforms;

@group(2) @binding(0)
var<uniform> tranforms:mat4x4<f32>;

@group(3) @binding(0)
var albedo_texture: texture_2d<f32>;
@group(3) @binding(1)
var normal_texture: texture_2d<f32>;
@group(3) @binding(2)
var metallic_texture: texture_2d<f32>;
@group(3) @binding(3)
var ao_texture: texture_2d<f32>;
@group(3) @binding(4)
var s_diffuse: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput{
    var out: VertexOutput;
    var position  = vec4<f32>(input.position, 1.0);

    out.normal = input.normal;

    out.clip_position = camera.view_proj * tranforms * position;

    out.uv0 = vec2<f32>(input.uv0.x,  1.0 - input.uv0.y);
//    out.uv0 = fract(out.uv0);

//    out.tangent = input.tangent;
//    out.tex_coords = input.tex_coords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv0;

     // 直接采样主纹理
    let base_color = textureSample(albedo_texture, s_diffuse, uv);

    // 可选：添加法线贴图
    // let normal_sample = textureSample(t_normal, s_normal, in.tex_coords);

    // 可选：添加自发光
    // let emission = textureSample(t_emission, s_emission, in.tex_coords);

    return base_color;
}

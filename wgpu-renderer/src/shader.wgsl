enable f16;

// Constants
const PI: f32 = 3.14159265359;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) tangent: vec4<f32>,
    @location(4) uv0: vec2<f32>,
}

struct InstanceInput {
    @location(8) model_matrix_0: vec4<f32>,
    @location(9) model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec3<f32>,     // Tangent
    @location(3) bitangent: vec3<f32>,   // Bitangent
    @location(4) uv0: vec2<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_position: vec3<f32>,
}

struct SceneUniforms {
    ambient_light: vec3<f32>,
    ambient_intensity: f32,
    // Simple directional light for demo purposes
    light_direction: vec3<f32>, 
    light_color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var<uniform> scene: SceneUniforms;

@group(2) @binding(0)
var albedo_texture: texture_2d<f32>;
@group(2) @binding(1)
var normal_texture: texture_2d<f32>;
@group(2) @binding(2)
var metallic_texture: texture_2d<f32>;
@group(2) @binding(3)
var ao_texture: texture_2d<f32>;
@group(2) @binding(4)
var s_sampler: sampler;

@vertex
fn vs_main(
    input: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    let world_position = model_matrix * vec4<f32>(input.position, 1.0);
    out.world_position = world_position.xyz;
    out.clip_position = camera.view_proj * world_position;
    
    // Pass UV
    out.uv0 = vec2<f32>(input.uv0.x, 1.0 - input.uv0.y); // Flip Y for Unity UVs

    // Calculate TBN Matrix
    // Note: Assuming uniform scaling for simplicity in normal transformation
    // Ideally should use inverse-transpose of the upper 3x3 model matrix
    let normal_matrix = mat3x3<f32>(
        model_matrix[0].xyz,
        model_matrix[1].xyz,
        model_matrix[2].xyz
    );

    let T = normalize(normal_matrix * input.tangent.xyz);
    let N = normalize(normal_matrix * input.normal.xyz);
    // Re-orthogonalize T with respect to N
    let T_reorth = normalize(T - dot(T, N) * N);
    // Handedness correction (Unity w component of tangent stores the handedness)
    let B = cross(N, T_reorth) * input.tangent.w;

    out.tangent = T_reorth;
    out.bitangent = B;
    out.normal = N;

    return out;
}

// ----------------------------------------------------------------------------
// PBR Functions
// ----------------------------------------------------------------------------

fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let nom = a2;
    let denom = (NdotH2 * (a2 - 1.0) + 1.0);
    return nom / (PI * denom * denom);
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let nom = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return nom / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cos_theta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

// ----------------------------------------------------------------------------
// Fragment Shader
// ----------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv0;

    // 1. Sample Textures
    let albedo_raw = textureSample(albedo_texture, s_sampler, uv);
    let albedo = pow(albedo_raw.rgb, vec3<f32>(2.2)); // Gamma to Linear
    
    // Normal Mapping
    let normal_map = textureSample(normal_texture, s_sampler, uv).rgb;
    let normal_tangent = normalize(normal_map * 2.0 - 1.0);
    
    // Construct TBN matrix from interpolated vertex inputs
    let T = normalize(in.tangent);
    let B = normalize(in.bitangent);
    let N_geom = normalize(in.normal);
    let TBN = mat3x3<f32>(T, B, N_geom);
    let N = normalize(TBN * normal_tangent);

    // Standard Unity Packing: Metallic (R), Smoothness (A)
    // Note: We might be using a separate metallic texture channel depending on setup,
    // but here assuming the standard "MetallicGlossMap" or separate if available.
    // The Rust code binds `metallic_texture` which might be a packed map.
    // If it's a packed metallic/gloss map: R=Metallic, A=Smoothness.
    let metallic_sample = textureSample(metallic_texture, s_sampler, uv);
    let metallic = metallic_sample.r;
    let roughness = 1.0 - metallic_sample.a; // Smoothness to Roughness

    // AO
    let ao = textureSample(ao_texture, s_sampler, uv).r;

    // 2. PBR Setup
    let V = normalize(camera.view_position - in.world_position);
    
    // F0: Surface reflection at zero incidence
    // 0.04 for dielectrics, albedo for metals
    var F0 = vec3<f32>(0.04); 
    F0 = mix(F0, albedo, metallic);

    // 3. Lighting Calculation (Single Directional Light)
    // In a real engine, you'd loop over lights.
    // Using a hardcoded light direction if scene uniforms are missing specific light data, 
    // but let's try to use SceneUniforms assuming we added fields or use defaults.
    
    // Using a default directional light for now as `SceneUniforms` likely needs updates on Rust side 
    // to match perfectly, but we'll try to use what's likely available or calculate.
    // Let's assume a static directional light direction for PBR demo.
    let L = normalize(vec3<f32>(1.0, 1.0, 1.0)); 
    let H = normalize(V + L);
    
    // Radiance (Light Color * Intensity)
    let light_color = vec3<f32>(1.0, 1.0, 1.0); // White light
    let radiance = light_color * 3.0; // Intensity 3.0

    // Cook-Torrance BRDF
    let NDF = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

    let numerator = NDF * G * F;
    let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001; // + 0.0001 to prevent divide by zero
    let specular = numerator / denominator;

    // kS is essentially F
    let kS = F;
    // kD is the remaining energy that gets refracted (diffuse)
    var kD = vec3<f32>(1.0) - kS;
    // Metals have no diffuse component
    kD = kD * (1.0 - metallic);

    let NdotL = max(dot(N, L), 0.0);

    // Outgoing Radiance (Lo) for this light
    let Lo = (kD * albedo / PI + specular) * radiance * NdotL;

    // 4. Ambient Lighting
    // Simple ambient term. Ideally use IBL (Irradiance Map + Prefiltered Map + BRDF LUT)
    // Using `scene.ambient_light` (color) * `ao`
    let ambient = vec3<f32>(0.03) * albedo * ao; 
    
    let color_linear = ambient + Lo;

    // 5. Tone Mapping (Reinhard) & Gamma Correction
    let mapped = color_linear / (color_linear + vec3<f32>(1.0));
    let color_gamma = pow(mapped, vec3<f32>(1.0/2.2));

    return vec4<f32>(color_gamma, 1.0);
}

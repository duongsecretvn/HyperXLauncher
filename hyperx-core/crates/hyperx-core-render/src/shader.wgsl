struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    fog_color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) world_pos: vec3<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = in.position;
    out.clip_position = uniforms.view_proj * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    out.normal = in.normal;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Directional sunlight
    let light_dir = normalize(vec3<f32>(0.3, 1.0, 0.4));
    let diffuse = max(dot(in.normal, light_dir), 0.0) * 0.6 + 0.4;
    
    // Base block shading
    var final_color = in.color.rgb * diffuse;
    
    // Distance fog
    let dist = length(in.world_pos - uniforms.camera_pos.xyz);
    let fog_start = 64.0;
    let fog_end = 192.0;
    let fog_factor = clamp((dist - fog_start) / (fog_end - fog_start), 0.0, 1.0);
    
    final_color = mix(final_color, uniforms.fog_color.rgb, fog_factor);
    
    return vec4<f32>(final_color, in.color.a);
}

#import bevy_pbr::{
    mesh_functions,
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions,
    pbr_bindings,
    view_transformations
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{FragmentOutput},
}
#endif

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> colors: array<vec3<f32>, 256>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var<uniform> emissives: array<vec3<f32>, 256>;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color_index: u32
}

#ifndef PREPASS_PIPELINE
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) @interpolate(flat) instance_index: u32,
    @location(3) color: vec4<f32>,
    @location(4) emissive: vec3<f32>
}
#endif

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    
    var color = colors[vertex.color_index];
    out.color = vec4(color.x, color.y, color.z, 1.);
    out.emissive = emissives[vertex.color_index];
 
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        vertex.instance_index
    );

    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = view_transformations::position_world_to_clip(out.world_position.xyz);

    out.instance_index = vertex.instance_index;

    return out;
}

@fragment
fn fragment(@builtin(front_facing) is_front: bool, mesh: VertexOutput) -> FragmentOutput {
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = mesh.color;
    pbr_input.material.emissive = vec4(
        mesh.color.x * mesh.emissive.x,
        mesh.color.y * mesh.emissive.x,
        mesh.color.z * mesh.emissive.x,
        mesh.emissive.y
    );
    
    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;

    pbr_input.frag_coord = mesh.position;
    pbr_input.world_position = mesh.world_position;
    pbr_input.world_normal = pbr_functions::prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

    pbr_input.is_orthographic = view.clip_from_view[3].w == 1.0;
    pbr_input.N = normalize(pbr_input.world_normal);
    
    pbr_input.V = pbr_functions::calculate_view(mesh.world_position, pbr_input.is_orthographic);

#ifdef VERTEX_TANGENTS
    let Nt = textureSampleBias(pbr_bindings::normal_map_texture, pbr_bindings::normal_map_sampler, mesh.uv, view.mip_bias).rgb;
    let TBN = pbr_functions::calculate_tbn_mikktspace(mesh.world_normal, mesh.world_tangent);
    pbr_input.N = pbr_functions::apply_normal_mapping(
        pbr_input.material.flags,
        TBN,
        double_sided,
        is_front,
        Nt,
    );
#endif

#ifdef PREPASS_PIPELINE
    let out = deferred_output(mesh, pbr_input);
#else
    var out: FragmentOutput;
    out.color = pbr_functions::apply_pbr_lighting(pbr_input);
    out.color = pbr_functions::main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;
}

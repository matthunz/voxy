#import bevy_pbr::{
    mesh_view_bindings::view,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions,
    pbr_bindings,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
}
#endif

@fragment
fn fragment(@builtin(front_facing) is_front: bool, mesh: VertexOutput) -> FragmentOutput {
    var pbr_input: PbrInput = pbr_input_new();

    pbr_input.material.base_color = mesh.color;
    pbr_input.material.emissive = vec4(
        mesh.color.x * mesh.uv.x,
        mesh.color.y * mesh.uv.x,
        mesh.color.z * mesh.uv.x,
        mesh.uv.y
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

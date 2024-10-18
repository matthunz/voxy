use crate::ATTRIBUTE_COLOR_INDEX;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};
use uuid::Uuid;

pub const VOXEL_MATERIAL_SHADER_HANDLE: Handle<Shader> = Handle::Weak(AssetId::Uuid {
    uuid: Uuid::from_bytes([
        152, 99, 215, 179, 144, 131, 70, 105, 133, 171, 80, 205, 43, 117, 234, 20,
    ]),
});

pub struct VoxelMaterialPlugin;

impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default())
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(
                &VOXEL_MATERIAL_SHADER_HANDLE,
                Shader::from_wgsl(include_str!("voxel_material.wgsl"), "voxel_material.wgsl"),
            );
    }
}

#[derive(Clone, Debug, AsBindGroup, Asset, TypePath)]
pub struct VoxelMaterial {
    #[uniform(0)]
    pub colors: [Vec3; 256],
    #[uniform(1)]
    pub emissions: [Vec3; 256],
}

impl Material for VoxelMaterial {
    fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_MATERIAL_SHADER_HANDLE)
    }

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_MATERIAL_SHADER_HANDLE)
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_MATERIAL_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            ATTRIBUTE_COLOR_INDEX.at_shader_location(2),
        ])?;

        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

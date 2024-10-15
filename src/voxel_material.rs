use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef, UnpreparedBindGroup},
};
use uuid::Uuid;

const VOXEL_MATERIAL_SHADER_HANDLE: Handle<Shader> = Handle::Weak(AssetId::Uuid {
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

#[derive(Clone, Copy, Default, Asset, TypePath)]
pub struct VoxelMaterial;

impl AsBindGroup for VoxelMaterial {
    type Data = ();

    fn unprepared_bind_group(
        &self,
        _layout: &bevy::render::render_resource::BindGroupLayout,
        _render_device: &bevy::render::renderer::RenderDevice,
        _images: &bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<
        bevy::render::render_resource::UnpreparedBindGroup<Self::Data>,
        bevy::render::render_resource::AsBindGroupError,
    > {
        Ok(UnpreparedBindGroup {
            bindings: vec![],
            data: (),
        })
    }

    fn bind_group_layout_entries(
        _render_device: &bevy::render::renderer::RenderDevice,
    ) -> Vec<bevy::render::render_resource::BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        Vec::new()
    }
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_MATERIAL_SHADER_HANDLE)
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(VOXEL_MATERIAL_SHADER_HANDLE)
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

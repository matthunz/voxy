use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef, UnpreparedBindGroup},
    },
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::Shape;

pub trait Palette {
    type Voxel;

    fn color(
        &self,
        voxel: &Self::Voxel,
        indices: &[u32; 6],
        positions: &[[f32; 3]; 4],
        normals: &[[f32; 3]; 4],
    ) -> Color;
}

pub struct Chunk<'a, P, V, S> {
    pub palette: &'a P,
    pub voxels: &'a [V],
    pub shape: S,
    pub min: UVec3,
    pub max: UVec3,
}

impl<P, V, S> MeshBuilder for Chunk<'_, P, V, S>
where
    P: Palette<Voxel = V>,
    V: MergeVoxel,
    S: Shape<3, Coord = u32>,
{
    fn build(&self) -> Mesh {
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        let mut quad_buffer = GreedyQuadsBuffer::new(self.shape.size() as _);

        greedy_quads(
            self.voxels,
            &self.shape,
            self.min.into(),
            self.max.into(),
            &faces,
            &mut quad_buffer,
        );

        let num_indices = quad_buffer.quads.num_quads() * 6;
        let num_vertices = quad_buffer.quads.num_quads() * 4;

        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut colors = Vec::with_capacity(num_vertices);

        for (quads, face) in quad_buffer.quads.groups.into_iter().zip(faces) {
            for quad in quads {
                let quad_indices = face.quad_mesh_indices(positions.len() as u32);
                indices.extend_from_slice(&quad_indices);

                let quad_positions = face.quad_mesh_positions(&quad, 1.);
                positions.extend_from_slice(&quad_positions);

                let quad_normals = face.quad_mesh_normals();
                normals.extend_from_slice(&quad_normals);

                let idx = self.shape.linearize(quad.minimum);
                for _ in 0..4 {
                    let color = self.palette.color(
                        &self.voxels[idx as usize],
                        &quad_indices,
                        &quad_positions,
                        &quad_normals,
                    );
                    colors.push(color.to_srgba().to_u8_array().map(|x| x as f32));
                }
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![Vec2::ZERO; num_vertices])
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
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
        "shader.wgsl".into()
    }
}

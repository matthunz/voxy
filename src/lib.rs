use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, MeshVertexAttribute, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{PrimitiveTopology, VertexFormat},
    },
};
use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, RIGHT_HANDED_Y_UP_CONFIG};
use ndshape::Shape;
use std::marker::PhantomData;

pub mod prelude {
    pub use crate::scene::{VoxelScene, VoxelSceneModels};
    pub use crate::voxel_material::VoxelMaterial;
}

mod asset;
pub use self::asset::{
    AssetVoxel, VoxAssetLoader, VoxFileAsset, VoxFileAssetPlugin, 
};

pub mod scene;
pub use self::scene::{ScenePlugin, VoxelLight, VoxelScene, VoxelSceneModels};

mod voxel_material;
pub use self::voxel_material::{VoxelMaterial, VoxelMaterialPlugin};

pub struct DefaultPlugins;

impl Plugin for DefaultPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins((VoxelMaterialPlugin, VoxFileAssetPlugin, ScenePlugin));
    }
}

pub const ATTRIBUTE_COLOR_INDEX: MeshVertexAttribute =
    MeshVertexAttribute::new("ColorIndex", 988940917, VertexFormat::Uint32);

/// A chunk of voxels that can be built into a mesh.
/// 
/// This struct produces a [`Mesh`] with standard attributes so it can be rendered with a [`VoxelMaterial`] or extended with custom shaders.
/// 
/// [`ATTRIBUTE_COLOR_INDEX`] is inserted into the mesh for each quad, representing the voxel index.
pub struct Chunk<V, VS, S> {
    pub voxels: VS,
    pub shape: S,
    pub min: UVec3,
    pub max: UVec3,
    _marker: PhantomData<V>,
}

impl<V, VS, S> MeshBuilder for Chunk<V, VS, S>
where
    VS: AsRef<[V]>,
    V: MergeVoxel + AsRef<u8>,
    S: Shape<3, Coord = u32>,
{
    fn build(&self) -> Mesh {
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        let mut quad_buffer = GreedyQuadsBuffer::new(self.voxels.as_ref().len());

        greedy_quads(
            self.voxels.as_ref(),
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
        let mut color_indices = Vec::with_capacity(num_vertices);

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
                    color_indices.push(*self.voxels.as_ref()[idx as usize].as_ref() as u32 - 1);
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
        .with_inserted_attribute(
            ATTRIBUTE_COLOR_INDEX,
            VertexAttributeValues::Uint32(color_indices),
        )
        .with_inserted_indices(Indices::U32(indices))
    }
}

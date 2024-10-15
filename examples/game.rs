use bevy::prelude::*;
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use ndshape::{ConstShape, ConstShape3u32};
use voxy::{Chunk, Palette, VoxelMaterial};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

const CHUNK_SIZE: u32 = 16;
const PADDED_CHUNK_SIZE: u32 = CHUNK_SIZE + 2;

type PaddedChunkShape = ConstShape3u32<PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE, PADDED_CHUNK_SIZE>;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    let mut voxels = [Block::Air; PaddedChunkShape::SIZE as usize];
    for z in 1..10 {
        for y in 1..10 {
            for x in 1..10 {
                let i = PaddedChunkShape::linearize([x, y, z]);
                voxels[i as usize] = Block::Stone;
            }
        }
    }

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Chunk {
            voxels: &voxels,
            shape: PaddedChunkShape {},
            min: UVec3::ZERO,
            max: UVec3::splat(CHUNK_SIZE + 1),
            palette: &BlockPalette,
        }),
        material: materials.add(VoxelMaterial),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight::default());

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::splat(40.)).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Dirt,
    Stone,
}

impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == Block::Air {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Block {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

pub struct BlockPalette;

impl Palette for BlockPalette {
    type Voxel = Block;

    fn color(
        &self,
        voxel: &Self::Voxel,
        _indices: &[u32; 6],
        _positions: &[[f32; 3]; 4],
        _normals: &[[f32; 3]; 4],
    ) -> Color {
        match voxel {
            Block::Air => Color::NONE,
            Block::Dirt => Color::srgb(0.5, 0.25, 0.),
            Block::Stone => Color::srgb_u8(255, 0, 0),
        }
    }
}

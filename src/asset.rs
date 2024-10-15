use crate::{Chunk, Emission, Palette, PaletteSample};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::ConditionalSendFuture,
};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use dot_vox::DotVoxData;
use ndshape::{RuntimeShape, Shape};
use smol::io::AsyncReadExt;

pub struct VoxFileAssetPlugin;

impl Plugin for VoxFileAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VoxFileAsset>()
            .init_asset_loader::<VoxAssetLoader>();
    }
}

#[derive(Clone, Copy, Default)]
pub struct AssetVoxel {
    idx: u8,
}

impl Voxel for AssetVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.idx == 0 {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for AssetVoxel {
    type MergeValue = u8;

    fn merge_value(&self) -> Self::MergeValue {
        self.idx
    }
}

pub struct VoxFilePalette {
    palette: Vec<PaletteSample>,
}

impl Palette for VoxFilePalette {
    type Voxel = AssetVoxel;

    fn sample(
        &self,
        voxel: &Self::Voxel,
        _indices: &[u32; 6],
        _positions: &[[f32; 3]; 4],
        _normals: &[[f32; 3]; 4],
    ) -> PaletteSample {
        self.palette[voxel.idx as usize]
    }
}

#[derive(Debug, Asset, TypePath)]
pub struct VoxFileAsset {
    pub file: DotVoxData,
}

impl VoxFileAsset {
    pub fn palette(&self) -> VoxFilePalette {
        VoxFilePalette {
            palette: self
                .file
                .palette
                .iter()
                .enumerate()
                .map(|(idx, color)| PaletteSample {
                    color: Color::srgb_u8(color.r, color.g, color.b),
                    emission: Emission {
                        alpha: self.file.materials[idx]
                            .properties
                            .get("_emit")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or_default(),
                        intensity: 1.,
                    },
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn chunks<'a>(
        &'a self,
        palette: &'a VoxFilePalette,
    ) -> impl Iterator<Item = Chunk<&'a VoxFilePalette, Vec<AssetVoxel>, RuntimeShape<u32, 3>>> + 'a
    {
        self.file.models.iter().map(move |model| {
            let shape =
                RuntimeShape::<u32, 3>::new([model.size.x + 2, model.size.y + 2, model.size.z + 2]);

            let mut voxels = vec![AssetVoxel::default(); shape.size() as usize];
            for voxel in &model.voxels {
                voxels[shape.linearize([voxel.x as u32 + 1, voxel.z as u32 + 1, voxel.y as u32 + 1])
                    as usize] = AssetVoxel { idx: voxel.i };
            }

            Chunk {
                palette,
                voxels,
                shape,
                min: UVec3::ZERO,
                max: UVec3::new(model.size.x, model.size.y, model.size.z),
            }
        })
    }
}

#[derive(Default)]
pub struct VoxAssetLoader;

impl AssetLoader for VoxAssetLoader {
    type Asset = VoxFileAsset;

    type Settings = ();

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            let file = dot_vox::load_bytes(&buf).unwrap();
            Ok(VoxFileAsset { file })
        }
    }
}

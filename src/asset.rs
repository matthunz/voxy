use crate::{Chunk, Emission, Palette, PaletteSample};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    utils::{hashbrown::HashMap, ConditionalSendFuture},
};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use dot_vox::{DotVoxData, SceneNode};
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
    pub idx: u8,
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
    pub samples: Vec<PaletteSample>,
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
        if voxel.idx == 0 {
            PaletteSample {
                color: Color::NONE,
                emission: Emission::default(),
            }
        } else {
            self.samples[voxel.idx as usize - 1]
        }
    }
}

#[derive(Component)]
pub struct VoxFileModels {
    pub entities: HashMap<String, Entity>,
}

#[derive(Debug, Asset, TypePath)]
pub struct VoxFileAsset {
    pub file: DotVoxData,
}

impl VoxFileAsset {
    pub fn palette(&self) -> VoxFilePalette {
        VoxFilePalette {
            samples: self
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

    pub fn chunks<'a, P>(
        &'a self,
        palette: P,
    ) -> impl Iterator<
        Item = (
            Chunk<P, Vec<AssetVoxel>, RuntimeShape<u32, 3>>,
            Transform,
            Option<String>,
        ),
    > + 'a
    where
        P: Palette + Clone + 'a,
    {
        let mut models = Vec::new();
        visit_node(
            &self.file,
            &mut models,
            &self.file.scenes[0],
            Transform::default(),
            None,
        );

        models.into_iter().map(move |(model, transform, name)| {
            let shape =
                RuntimeShape::<u32, 3>::new([model.size.x + 2, model.size.z + 2, model.size.y + 2]);

            let mut voxels = vec![AssetVoxel::default(); shape.size() as usize];
            for voxel in &model.voxels {
                voxels[shape.linearize([voxel.x as u32 + 1, voxel.z as u32 + 1, voxel.y as u32 + 1])
                    as usize] = AssetVoxel { idx: voxel.i + 1 };
            }

            (
                Chunk {
                    palette: palette.clone(),
                    voxels,
                    shape,
                    min: UVec3::ZERO,
                    max: UVec3::new(model.size.x + 1, model.size.z + 1, model.size.y + 1),
                },
                transform,
                name,
            )
        })
    }
}

fn visit_node<'a>(
    file: &'a DotVoxData,
    models: &mut Vec<(&'a dot_vox::Model, Transform, Option<String>)>,
    node: &SceneNode,
    transform: Transform,
    name: Option<String>,
) {
    match node {
        SceneNode::Transform {
            attributes,
            frames,
            child,
            ..
        } => {
            let translation = frames[0]
                .position()
                .map(|t| Vec3::new(-t.x as _, t.z as _, t.y as _))
                .unwrap_or_default();
            let name = attributes.get("_name").cloned().or(name);

            visit_node(
                file,
                models,
                &file.scenes[*child as usize],
                transform.with_translation(transform.translation + translation),
                name,
            );
        }
        SceneNode::Group { children, .. } => {
            for child in children {
                visit_node(
                    file,
                    models,
                    &file.scenes[*child as usize],
                    transform,
                    name.clone(),
                );
            }
        }
        SceneNode::Shape {
            models: shape_models,
            ..
        } => {
            for model in shape_models {
                models.push((
                    &file.models[model.model_id as usize],
                    transform.with_translation(
                        transform.translation
                            - Vec3::new(
                                file.models[model.model_id as usize].size.x as _,
                                file.models[model.model_id as usize].size.z as _,
                                file.models[model.model_id as usize].size.y as _,
                            ) / 2.,
                    ),
                    name.clone(),
                ));
            }
        }
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

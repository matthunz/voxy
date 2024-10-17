use crate::{Chunk, Emission, Palette, PaletteSample, VoxelMaterial};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadState},
    ecs::system::EntityCommands,
    prelude::*,
    utils::ConditionalSendFuture,
};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use dot_vox::{DotVoxData, SceneNode};
use ndshape::{RuntimeShape, Shape};
use smol::io::AsyncReadExt;

pub struct VoxFileAssetPlugin;

impl Plugin for VoxFileAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VoxFileAsset>()
            .init_asset_loader::<VoxAssetLoader>()
            .add_systems(Update, load_assets);
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
        self.samples[voxel.idx as usize]
    }
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

    pub fn chunks<'a>(
        &'a self,
        palette: &'a VoxFilePalette,
    ) -> impl Iterator<
        Item = (
            Chunk<&'a VoxFilePalette, Vec<AssetVoxel>, RuntimeShape<u32, 3>>,
            Transform,
        ),
    > + 'a {
        let mut next = vec![&self.file.scenes[0]];
        let mut transform = Transform::default();
        let mut models = Vec::new();
        while let Some(scene) = next.pop() {
            match scene {
                SceneNode::Transform { frames, child, .. } => {
                    let t = frames[0]
                        .position()
                        .map(|t| Vec3::new(t.x as _, t.y as _, t.z as _))
                        .unwrap_or_default();
                    transform.translation += t;

                    next.push(&self.file.scenes[*child as usize]);
                }
                SceneNode::Shape {
                    models: shape_models,
                    ..
                } => {
                    for model in shape_models {
                        models.push((&self.file.models[model.model_id as usize], transform));
                    }
                }
                SceneNode::Group { children, .. } => {
                    for child in children {
                        next.push(&self.file.scenes[*child as usize]);
                    }
                }
            }
        }

        models.into_iter().map(move |(model, transform)| {
            let shape =
                RuntimeShape::<u32, 3>::new([model.size.x + 2, model.size.y + 2, model.size.z + 2]);

            let mut voxels = vec![AssetVoxel::default(); shape.size() as usize];
            for voxel in &model.voxels {
                voxels[shape.linearize([voxel.x as u32 + 1, voxel.z as u32 + 1, voxel.y as u32 + 1])
                    as usize] = AssetVoxel { idx: voxel.i };
            }

            (
                Chunk {
                    palette,
                    voxels,
                    shape,
                    min: UVec3::ZERO,
                    max: UVec3::new(model.size.x, model.size.y, model.size.z),
                },
                transform,
            )
        })
    }

    fn spawn(
        &self,
        mut entity_commands: EntityCommands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<VoxelMaterial>,
    ) {
        let palette = self.palette();

        entity_commands.with_children(|parent| {
            for (chunk, transform) in self.chunks(&palette) {
                parent
                    .spawn_empty()
                    .with_children(|parent| {
                        for (idx, voxel) in chunk.voxels.iter().enumerate() {
                            let sample = chunk.palette.samples[voxel.idx as usize];

                            let [x, y, z] = chunk.shape.delinearize(idx as _).map(|n| n as f32);

                            if sample.emission.alpha > 0. {
                                parent.spawn(PointLightBundle {
                                    point_light: PointLight {
                                        intensity: sample.emission.alpha * 100_000.,
                                        range: 10.,
                                        ..default()
                                    },
                                    transform: Transform::from_translation(
                                        Vec3::new(x, y, z) + transform.translation,
                                    ),
                                    ..default()
                                });
                            }
                        }
                    })
                    .insert(MaterialMeshBundle {
                        material: materials.add(VoxelMaterial),
                        mesh: meshes.add(chunk),
                        ..default()
                    });
            }
        });
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

#[derive(Component)]
struct Loaded;

fn load_assets(
    mut commands: Commands,
    query: Query<(Entity, &Handle<VoxFileAsset>), Without<Loaded>>,
    asset_server: Res<AssetServer>,
    vox_assets: Res<Assets<VoxFileAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
) {
    for (entity, handle) in &query {
        if asset_server.load_state(handle) == LoadState::Loaded {
            commands.entity(entity).insert(Loaded);

            let vox = vox_assets.get(handle).unwrap();
            vox.spawn(commands.entity(entity), &mut meshes, &mut materials);
        }
    }
}

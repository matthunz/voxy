use crate::{
    VoxAssetLoader, VoxFileModels,
    VoxelMaterial,
};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadState},
    ecs::system::EntityCommands,
    prelude::*,
    utils::{hashbrown::HashMap, ConditionalSendFuture},
};
use futures::future;
use ndshape::Shape;
use std::sync::Arc;

pub struct VoxFileMeshAssetPlugin;

impl Plugin for VoxFileMeshAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VoxFileMeshAsset>()
            .init_asset_loader::<VoxFileMeshAssetLoader>()
            .add_systems(Update, load_assets);
    }
}

#[derive(Debug)]
pub struct VoxelLight {
    pub origin: Vec3,
    pub intensity: f32,
}

#[derive(Debug)]
pub struct LitMesh {
    pub mesh: Mesh,
    pub lights: Vec<VoxelLight>,
    pub name: Option<String>,
    pub transform: Transform,
}

#[derive(Debug, Asset, TypePath)]
pub struct VoxFileMeshAsset {
    meshes: Vec<LitMesh>,
}

impl VoxFileMeshAsset {
    fn spawn(
        &self,
        mut entity_commands: EntityCommands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<VoxelMaterial>,
    ) {
        let mut entities = HashMap::new();

        entity_commands.with_children(|parent| {
            for lit_mesh in &self.meshes {
                let entity = parent
                    .spawn_empty()
                    .with_children(|parent| {
                        for light in &lit_mesh.lights {
                            parent.spawn(PointLightBundle {
                                point_light: PointLight {
                                    intensity: light.intensity * 100_000.,
                                    range: 10.,
                                    ..default()
                                },
                                transform: Transform::from_translation(light.origin),
                                ..default()
                            });
                        }
                    })
                    .insert(MaterialMeshBundle {
                        material: materials.add(VoxelMaterial),
                        mesh: meshes.add(lit_mesh.mesh.clone()),
                        transform: lit_mesh.transform,
                        ..default()
                    })
                    .id();

                if let Some(name) = &lit_mesh.name {
                    entities.insert(name.clone(), entity);
                }
            }
        });

        entity_commands.insert(VoxFileModels { entities });
    }
}

#[derive(Default)]
pub struct VoxFileMeshAssetLoader;

impl AssetLoader for VoxFileMeshAssetLoader {
    type Asset = VoxFileMeshAsset;

    type Settings = ();

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let x = VoxAssetLoader.load(reader, settings, load_context).await?;

            let palette = Arc::new(x.palette());
            let chunks: Vec<_> = x.chunks(palette.clone()).collect();

            let meshes = future::join_all(chunks.into_iter().map(|(chunk, transform, name)| {
                smol::unblock(move || {
                    let mesh = chunk.build();
                    let mut lights = Vec::new();

                    // TODO
                    for (idx, voxel) in chunk.voxels.iter().enumerate() {
                        let sample = chunk.palette.samples[voxel.idx as usize];

                        let [x, y, z] = chunk.shape.delinearize(idx as _).map(|n| n as f32);

                        if sample.emission.alpha > 0. {
                            lights.push(VoxelLight {
                                origin: Vec3::new(x, y, z),
                                intensity: sample.emission.intensity,
                            });
                        }
                    }

                    LitMesh {
                        mesh,
                        lights,
                        name,
                        transform,
                    }
                })
            }))
            .await;

            Ok(VoxFileMeshAsset { meshes })
        }
    }
}

#[derive(Component)]
struct Loaded;

fn load_assets(
    mut commands: Commands,
    query: Query<(Entity, &Handle<VoxFileMeshAsset>), Without<Loaded>>,
    asset_server: Res<AssetServer>,
    vox_assets: Res<Assets<VoxFileMeshAsset>>,
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

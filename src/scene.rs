use crate::{VoxAssetLoader, VoxelMaterial};
use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext, LoadState},
    ecs::system::EntityCommands,
    prelude::*,
    utils::{hashbrown::HashMap, ConditionalSendFuture},
};
use futures::future;
use ndshape::Shape;
use std::sync::Arc;

pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<VoxelScene>()
            .init_asset_loader::<SceneLoader>()
            .init_resource::<LoadedAssets>()
            .add_systems(Update, (load_scenes, handle_scene_events));
    }
}

#[derive(Component)]
pub struct VoxelSceneModels {
    pub entities: HashMap<String, Entity>,
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
pub struct VoxelScene {
    pub meshes: Vec<LitMesh>,
    pub material: VoxelMaterial,
}

impl VoxelScene {
    fn spawn(
        &self,
        mut entity_commands: EntityCommands,
        material: Handle<VoxelMaterial>,
        meshes: &[Handle<Mesh>],
    ) {
        let mut entities = HashMap::new();

        entity_commands.with_children(|parent| {
            for (idx, lit_mesh) in self.meshes.iter().enumerate() {
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
                        material: material.clone(),
                        mesh: meshes[idx].clone(),
                        transform: lit_mesh.transform,
                        ..default()
                    })
                    .id();

                if let Some(name) = &lit_mesh.name {
                    entities.insert(name.clone(), entity);
                }
            }
        });

        entity_commands.insert(VoxelSceneModels { entities });
    }
}

#[derive(Default)]
pub struct SceneLoader;

impl AssetLoader for SceneLoader {
    type Asset = VoxelScene;

    type Settings = ();

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let asset = VoxAssetLoader.load(reader, settings, load_context).await?;

            let material = asset.material();

            let emissions = Arc::new(material.emissions);
            let chunks: Vec<_> = asset.chunks().collect();

            let meshes = future::join_all(chunks.into_iter().map(|asset_chunk| {
                let emissions = emissions.clone();

                smol::unblock(move || {
                    let mesh = asset_chunk.chunk.build();

                    // TODO check positions
                    let mut lights = Vec::new();
                    for (idx, voxel) in asset_chunk.chunk.voxels.iter().enumerate() {
                        let emissive = emissions[voxel.idx as usize];

                        let [x, y, z] = asset_chunk.chunk.shape.delinearize(idx as _).map(|n| n as f32);

                        if emissive.x > 0. {
                            lights.push(VoxelLight {
                                origin: Vec3::new(x, y, z),
                                intensity: emissive.x,
                            });
                        }
                    }

                    LitMesh {
                        mesh,
                        lights,
                        name: asset_chunk.name,
                        transform: asset_chunk.transform,
                    }
                })
            }))
            .await;

            Ok(VoxelScene { meshes, material })
        }
    }
}

struct MaterialMeshes {
    material: Handle<VoxelMaterial>,
    meshes: Vec<Handle<Mesh>>,
}

#[derive(Default, Resource)]
pub struct LoadedAssets {
    assets: HashMap<AssetId<VoxelScene>, MaterialMeshes>,
}

#[derive(Component)]
pub struct Loaded;

pub fn load_scenes(
    mut commands: Commands,
    query: Query<(Entity, &Handle<VoxelScene>), Without<Loaded>>,
    asset_server: Res<AssetServer>,
    vox_assets: Res<Assets<VoxelScene>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut loaded_assets: ResMut<LoadedAssets>,
) {
    for (entity, handle) in &query {
        if asset_server.load_state(handle) == LoadState::Loaded {
            let scene = vox_assets.get(handle).unwrap();

            if !loaded_assets.assets.contains_key(&handle.id()) {
                loaded_assets.assets.insert(
                    handle.id(),
                    MaterialMeshes {
                        material: materials.add(scene.material.clone()),
                        meshes: scene
                            .meshes
                            .iter()
                            .map(|lit_mesh| meshes.add(lit_mesh.mesh.clone()))
                            .collect(),
                    },
                );
            }

            commands.entity(entity).insert(Loaded);

            let material_meshes = &loaded_assets.assets.get(&handle.id()).unwrap();
            scene.spawn(
                commands.entity(entity),
                material_meshes.material.clone(),
                &material_meshes.meshes,
            );
        }
    }
}

pub fn handle_scene_events(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<VoxelScene>>,
    scenes: Res<Assets<VoxelScene>>,
    query: Query<(Entity, &Handle<VoxelScene>), With<Loaded>>,
    mut loaded_assets: ResMut<LoadedAssets>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for event in events.read() {
        if let AssetEvent::Modified { id } = event {
            for (entity, handle) in &query {
                if handle.id() == *id {
                    let scene = scenes.get(handle).unwrap();

                    commands
                        .entity(entity)
                        .despawn_descendants()
                        .remove::<VoxelSceneModels>();

                    loaded_assets.assets.insert(
                        handle.id(),
                        MaterialMeshes {
                            material: materials.add(scene.material.clone()),
                            meshes: scene
                                .meshes
                                .iter()
                                .map(|lit_mesh| meshes.add(lit_mesh.mesh.clone()))
                                .collect(),
                        },
                    );

                    let material_meshes = &loaded_assets.assets.get(&handle.id()).unwrap();
                    scene.spawn(
                        commands.entity(entity),
                        material_meshes.material.clone(),
                        &material_meshes.meshes,
                    );
                }
            }
        }
    }
}

use bevy::{asset::LoadState, core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxFileAsset, VoxFileAssetPlugin, VoxelMaterial, VoxelMaterialPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            VoxelMaterialPlugin,
            VoxFileAssetPlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, load_asset)
        .run();
}

#[derive(Default, Resource)]
struct LoadingAsset(Option<Handle<VoxFileAsset>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let vox_file: Handle<VoxFileAsset> = asset_server.load("example.vox");
    commands.insert_resource(LoadingAsset(Some(vox_file)));

    commands.insert_resource(AmbientLight {
        brightness: 0.,
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_translation(Vec3::splat(100.))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        BloomSettings::NATURAL,
    ));
}

fn load_asset(
    mut commands: Commands,
    mut loading_asset: ResMut<LoadingAsset>,
    asset_server: Res<AssetServer>,
    vox_assets: Res<Assets<VoxFileAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut voxel_materials: ResMut<Assets<VoxelMaterial>>,
) {
    if let Some(handle) = loading_asset.0.clone() {
        if asset_server.load_state(&handle) == LoadState::Loaded {
            loading_asset.0 = None;

            let vox = vox_assets.get(&handle).unwrap();
            let palette = vox.palette();

            for chunk in vox.chunks(&palette) {
                commands.spawn(MaterialMeshBundle {
                    material: voxel_materials.add(VoxelMaterial),
                    mesh: meshes.add(chunk),
                    ..default()
                });
            }
        }
    }
}

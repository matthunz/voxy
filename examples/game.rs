use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxFileAsset, VoxFileAssetPlugin, VoxelMaterialPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VoxelMaterialPlugin, VoxFileAssetPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let vox_file: Handle<VoxFileAsset> = asset_server.load("example.vox");
    commands.spawn(vox_file);

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

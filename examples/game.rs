use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxFileAsset, VoxFileAssetPlugin, VoxelMaterialPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VoxelMaterialPlugin, VoxFileAssetPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        asset_server.load::<VoxFileAsset>("character.vox"),
        TransformBundle::default(),
    ));

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

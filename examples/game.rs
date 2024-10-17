use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxFileAsset, VoxFileAssetPlugin, VoxFileModels, VoxelMaterialPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VoxelMaterialPlugin, VoxFileAssetPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        asset_server.load::<VoxFileAsset>("character.vox"),
        SpatialBundle::default(),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 500.,
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-60., 60., -60.))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        BloomSettings::NATURAL,
    ));
}

fn rotate(models_query: Query<&VoxFileModels>, mut transform_query: Query<&mut Transform>) {
    for models in &models_query {
        /*if let Some(entity) = models.entities.get("left_arm") {
            transform_query
                .get_mut(*entity)
                .unwrap()
                .rotate_around(Vec3::ZERO, Quat::from_rotation_x(0.01));
        }*/
    }
}

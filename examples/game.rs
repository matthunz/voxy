use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxelScene, VoxelSceneModels};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, voxy::DefaultPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        asset_server.load::<VoxelScene>("character.vox"),
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

fn rotate(models_query: Query<&VoxelSceneModels>, mut transform_query: Query<&mut Transform>) {
    for models in &models_query {
        if let Some(entity) = models.entities.get("right_arm") {
            let mut transform = transform_query.get_mut(*entity).unwrap();

            transform.rotate_around(Vec3::new(0., 24., 4.), Quat::from_rotation_x(0.01));
        }
    }
}

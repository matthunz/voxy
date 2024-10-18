# Voxy

[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/matthunz/voxy)
[![Crates.io](https://img.shields.io/crates/v/voxy.svg)](https://crates.io/crates/voxy)
[![Downloads](https://img.shields.io/crates/d/voxy.svg)](https://crates.io/crates/voxy)
[![Docs](https://docs.rs/voxy/badge.svg)](https://docs.rs/voxy/latest/voxy/)
[![CI](https://github.com/matthunz/voxy/workflows/CI/badge.svg)](https://github.com/matthunz/voxy/actions)

A voxel engine for [Bevy](https://github.com/bevyengine/bevy).

Features:
 - Uses the [block_mesh](https://docs.rs/block-mesh/latest/block_mesh/) crate for high-performance chunk meshing
 - Uses the [dot_vox](https://github.com/dust-engine/dot_vox) crate to load [MagicaVoxel](https://ephtracy.github.io/) `.vox` files
   - Load multiple models into a `Scene`
   - Hot-reload of scene files
   - Emissive textures and lighting

```rs
use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, voxy::DefaultPlugins))
        .add_systems(Startup, setup)
        .add_systems(Update, rotate)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn a scene with multiple models from a `.vox` file.
    commands.spawn((
        asset_server.load::<VoxelScene>("character.vox"),
        SpatialBundle::default(),
    ));

    // Setup default lighting.
    commands.insert_resource(AmbientLight {
        brightness: 500.,
        ..default()
    });

    // Setup the camera.
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

// Rotate the right arm of our character.
fn rotate(models_query: Query<&VoxelSceneModels>, mut transform_query: Query<&mut Transform>) {
    for models in &models_query {
        if let Some(entity) = models.entities.get("right_arm") {
            let mut transform = transform_query.get_mut(*entity).unwrap();

            transform.rotate_around(Vec3::new(0., 24., 4.), Quat::from_rotation_x(0.01));
        }
    }
}
```

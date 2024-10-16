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

```rs
use bevy::{core_pipeline::bloom::BloomSettings, prelude::*};
use voxy::{VoxFileAsset, VoxFileAssetPlugin, VoxelMaterialPlugin};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, VoxelMaterialPlugin, VoxFileAssetPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load and spawn `example.vox`.
    commands.spawn(asset_server.load::<VoxFileAsset>("example.vox"));

    // Setup camera.
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(Vec3::splat(80.))
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings::NATURAL,
    ));
}
```

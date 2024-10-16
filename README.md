# Voxy

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
    let vox_file: Handle<VoxFileAsset> = asset_server.load("example.vox");
    commands.spawn(vox_file);

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

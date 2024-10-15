use bevy::{asset::LoadState, core_pipeline::bloom::BloomSettings, prelude::*};
use block_mesh::{MergeVoxel, Voxel, VoxelVisibility};
use voxy::{Emission, Palette, PaletteSample, VoxFileAsset, VoxAssetLoader, VoxelMaterial};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()),
            MaterialPlugin::<VoxelMaterial>::default(),
        ))
        .init_asset::<VoxFileAsset>()
        .init_asset_loader::<VoxAssetLoader>()
        .add_systems(Startup, setup)
        .add_systems(Update, load_asset)
        .run();
}

#[derive(Default, Resource)]
struct LoadingAsset(Option<Handle<VoxFileAsset>>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let x: Handle<VoxFileAsset> = asset_server.load("example.vox");
    commands.insert_resource(LoadingAsset(Some(x)));

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Air,
    Solid,
    Light,
}

impl Voxel for Block {
    fn get_visibility(&self) -> VoxelVisibility {
        if *self == Block::Air {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for Block {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        *self
    }
}

pub struct BlockPalette;

impl Palette for BlockPalette {
    type Voxel = Block;

    fn sample(
        &self,
        voxel: &Self::Voxel,
        _indices: &[u32; 6],
        _positions: &[[f32; 3]; 4],
        _normals: &[[f32; 3]; 4],
    ) -> PaletteSample {
        match voxel {
            Block::Air => PaletteSample::default(),
            Block::Solid => PaletteSample {
                color: Color::srgb_u8(255, 255, 0),
                emission: Emission {
                    alpha: 1.,
                    intensity: 1.,
                },
            },
            Block::Light => PaletteSample {
                color: Color::srgb_u8(255, 0, 0),
                emission: Emission {
                    alpha: 1.,
                    intensity: 1.,
                },
            },
        }
    }
}

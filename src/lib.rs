use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_asset::RenderAssetUsages,
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef, UnpreparedBindGroup},
    },
    utils::ConditionalSendFuture,
};
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use dot_vox::DotVoxData;
use ndshape::{RuntimeShape, Shape};
use smol::io::AsyncReadExt;

#[derive(Clone, Copy, Default)]
pub struct Emission {
    pub alpha: f32,
    pub intensity: f32,
}

#[derive(Clone, Copy, Default)]
pub struct PaletteSample {
    pub color: Color,
    pub emission: Emission,
}

pub trait Palette {
    type Voxel;

    fn sample(
        &self,
        voxel: &Self::Voxel,
        indices: &[u32; 6],
        positions: &[[f32; 3]; 4],
        normals: &[[f32; 3]; 4],
    ) -> PaletteSample;
}

impl<P: Palette> Palette for &P {
    type Voxel = P::Voxel;

    fn sample(
        &self,
        voxel: &Self::Voxel,
        indices: &[u32; 6],
        positions: &[[f32; 3]; 4],
        normals: &[[f32; 3]; 4],
    ) -> PaletteSample {
        (**self).sample(voxel, indices, positions, normals)
    }
}

pub struct Chunk<P, V, S> {
    pub palette: P,
    pub voxels: V,
    pub shape: S,
    pub min: UVec3,
    pub max: UVec3,
}

impl<P, V, VS, S> MeshBuilder for Chunk<P, VS, S>
where
    P: Palette<Voxel = V>,
    VS: AsRef<[V]>,
    V: MergeVoxel,
    S: Shape<3, Coord = u32>,
{
    fn build(&self) -> Mesh {
        let faces = RIGHT_HANDED_Y_UP_CONFIG.faces;
        let mut quad_buffer = GreedyQuadsBuffer::new(self.shape.size() as _);

        greedy_quads(
            self.voxels.as_ref(),
            &self.shape,
            self.min.into(),
            self.max.into(),
            &faces,
            &mut quad_buffer,
        );

        let num_indices = quad_buffer.quads.num_quads() * 6;
        let num_vertices = quad_buffer.quads.num_quads() * 4;

        let mut indices = Vec::with_capacity(num_indices);
        let mut positions = Vec::with_capacity(num_vertices);
        let mut normals = Vec::with_capacity(num_vertices);
        let mut colors = Vec::with_capacity(num_vertices);
        let mut emissions = Vec::with_capacity(num_vertices);

        for (quads, face) in quad_buffer.quads.groups.into_iter().zip(faces) {
            for quad in quads {
                let quad_indices = face.quad_mesh_indices(positions.len() as u32);
                indices.extend_from_slice(&quad_indices);

                let quad_positions = face.quad_mesh_positions(&quad, 1.);
                positions.extend_from_slice(&quad_positions);

                let quad_normals = face.quad_mesh_normals();
                normals.extend_from_slice(&quad_normals);

                let idx = self.shape.linearize(quad.minimum);
                for _ in 0..4 {
                    let sample = self.palette.sample(
                        &self.voxels.as_ref()[idx as usize],
                        &quad_indices,
                        &quad_positions,
                        &quad_normals,
                    );
                    colors.push(sample.color.to_srgba().to_u8_array().map(|x| x as f32));
                    emissions.push([sample.emission.alpha, sample.emission.intensity]);
                }
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float32x3(positions),
        )
        .with_inserted_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            VertexAttributeValues::Float32x3(normals),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, emissions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(indices))
    }
}

#[derive(Clone, Copy, Default, Asset, TypePath)]
pub struct VoxelMaterial;

impl AsBindGroup for VoxelMaterial {
    type Data = ();

    fn unprepared_bind_group(
        &self,
        _layout: &bevy::render::render_resource::BindGroupLayout,
        _render_device: &bevy::render::renderer::RenderDevice,
        _images: &bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>,
        _fallback_image: &bevy::render::texture::FallbackImage,
    ) -> Result<
        bevy::render::render_resource::UnpreparedBindGroup<Self::Data>,
        bevy::render::render_resource::AsBindGroupError,
    > {
        Ok(UnpreparedBindGroup {
            bindings: vec![],
            data: (),
        })
    }

    fn bind_group_layout_entries(
        _render_device: &bevy::render::renderer::RenderDevice,
    ) -> Vec<bevy::render::render_resource::BindGroupLayoutEntry>
    where
        Self: Sized,
    {
        Vec::new()
    }
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shader.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }
}

#[derive(Clone, Copy, Default)]
pub struct AssetVoxel {
    idx: u8,
}

impl Voxel for AssetVoxel {
    fn get_visibility(&self) -> VoxelVisibility {
        if self.idx == 0 {
            VoxelVisibility::Empty
        } else {
            VoxelVisibility::Opaque
        }
    }
}

impl MergeVoxel for AssetVoxel {
    type MergeValue = u8;

    fn merge_value(&self) -> Self::MergeValue {
        self.idx
    }
}

pub struct VoxFilePalette {
    palette: Vec<PaletteSample>,
}

impl Palette for VoxFilePalette {
    type Voxel = AssetVoxel;

    fn sample(
        &self,
        voxel: &Self::Voxel,
        _indices: &[u32; 6],
        _positions: &[[f32; 3]; 4],
        _normals: &[[f32; 3]; 4],
    ) -> PaletteSample {
        self.palette[voxel.idx as usize]
    }
}

#[derive(Debug, Asset, TypePath)]
pub struct VoxFileAsset {
    pub file: DotVoxData,
}

impl VoxFileAsset {
    pub fn palette(&self) -> VoxFilePalette {
        VoxFilePalette {
            palette: self
                .file
                .palette
                .iter()
                .enumerate()
                .map(|(idx, color)| PaletteSample {
                    color: Color::srgb_u8(color.r, color.g, color.b),
                    emission: Emission {
                        alpha: self.file.materials[idx]
                            .properties
                            .get("_emit")
                            .and_then(|s| s.parse().ok())
                            .unwrap_or_default(),
                        intensity: 1.,
                    },
                })
                .collect::<Vec<_>>(),
        }
    }

    pub fn chunks<'a>(
        &'a self,
        palette: &'a VoxFilePalette,
    ) -> impl Iterator<Item = Chunk<&'a VoxFilePalette, Vec<AssetVoxel>, RuntimeShape<u32, 3>>> + 'a
    {
        self.file.models.iter().map(move |model| {
            let shape =
                RuntimeShape::<u32, 3>::new([model.size.x + 2, model.size.y + 2, model.size.z + 2]);

            let mut voxels = vec![AssetVoxel::default(); shape.size() as usize];
            for voxel in &model.voxels {
                voxels[shape.linearize([voxel.x as u32 + 1, voxel.z as u32 + 1, voxel.y as u32 + 1])
                    as usize] = AssetVoxel { idx: voxel.i };
            }

            Chunk {
                palette,
                voxels,
                shape,
                min: UVec3::ZERO,
                max: UVec3::new(model.size.x, model.size.y, model.size.z),
            }
        })
    }
}

#[derive(Default)]
pub struct VoxAssetLoader;

impl AssetLoader for VoxAssetLoader {
    type Asset = VoxFileAsset;

    type Settings = ();

    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        _load_context: &'a mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        async move {
            let mut buf = Vec::new();
            reader.read_to_end(&mut buf).await?;

            let file = dot_vox::load_bytes(&buf).unwrap();
            Ok(VoxFileAsset { file })
        }
    }
}

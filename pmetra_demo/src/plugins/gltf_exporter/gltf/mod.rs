use std::io::Cursor;
use bevy_pmetra::re_exports::anyhow::{anyhow, Result};

use base64::{engine::general_purpose::STANDARD, Engine};
use bevy::{prelude::*, render::mesh::VertexAttributeValues};
use gltf::json;
use image::RgbaImage;
use itertools::Itertools;
use json::validation::Checked::Valid;

use converters::ToGltfMaterial;

use self::converters::StandardMaterialWithImages;

pub mod converters;

/// Gltf exporter.
#[derive(Debug, Clone)]
pub struct GltfExporter {
    pub vertices: Vec<GltfExporterVertex>,
    pub material: StandardMaterialWithImages,
}

impl GltfExporter {
    /// Export GLTF to file.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn export(&self, path: String, output: GltfExporterOutput) -> Result<()> {
        use std::borrow::Cow;
        use std::io::Write;
        use std::path::PathBuf;
        use std::str::FromStr;
        use std::{fs, mem};

        use gltf::json::buffer::Stride;
        use gltf::json::material::{NormalTexture, OcclusionTexture, StrengthFactor};
        use gltf::json::texture::{Info, Sampler};
        use gltf::json::validation::USize64;
        use gltf::json::{Image, Texture};

        let mut path = PathBuf::from_str(&path)?;
        // set proper extension...
        match output {
            GltfExporterOutput::Standard => {
                path.set_extension("gltf");
            }
            GltfExporterOutput::Binary => {
                path.set_extension("glb");
            }
        }
        let dir_path = path
            .parent()
            .ok_or_else(|| anyhow!("Could not get parent of given path!"))?;
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow!("Could not get file_name from path!"))?
            .to_str()
            .ok_or_else(|| anyhow!("to_str() returned None!"))?
            .to_string();

        let vertices = &self.vertices;
        let bounds = GltfExporterVertexBounds::from(vertices.as_ref());

        let buffer_length = (vertices.len() * mem::size_of::<GltfExporterVertex>()) as u64;
        let buffer_uri = file_name + "_buffer0.bin";
        let buffer = json::Buffer {
            byte_length: buffer_length.into(),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            uri: if output == GltfExporterOutput::Standard {
                Some(buffer_uri.clone())
            } else {
                None
            },
        };
        let buffer_view = json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: buffer.byte_length,
            byte_offset: None,
            byte_stride: Some(Stride(mem::size_of::<GltfExporterVertex>())),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        };
        let positions = json::Accessor {
            buffer_view: Some(json::Index::new(0)),
            byte_offset: Some(USize64(0)),
            count: USize64(vertices.len() as u64),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::from(bounds.min.position.to_vec())),
            max: Some(json::Value::from(bounds.max.position.to_vec())),
            name: None,
            normalized: false,
            sparse: None,
        };
        let tex_coords = json::Accessor {
            buffer_view: Some(json::Index::new(0)),
            byte_offset: Some(USize64(3 * mem::size_of::<f32>() as u64)),
            count: USize64(vertices.len() as u64),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: Some(json::Value::from(bounds.min.uv.to_vec())),
            max: Some(json::Value::from(bounds.max.uv.to_vec())),
            name: None,
            normalized: false,
            sparse: None,
        };

        // Material stuff...

        let mut images = vec![];
        let mut samplers = vec![];
        let mut textures = vec![];

        let sampler = Sampler::default();
        samplers.push(sampler);

        let mut material = self.material.to_gltf_material_no_textures()?;
        if let Some(image) = &self.material.emissive_texture {
            let uri = image_to_uri(image)?;
            let image = Image {
                buffer_view: None,
                mime_type: None,
                name: None,
                uri: Some(uri),
                extensions: None,
                extras: None,
            };
            images.push(image);

            let texture = Texture {
                name: None,
                sampler: Some(json::Index::new(samplers.len() as u32 - 1)),
                source: json::Index::new(images.len() as u32 - 1),
                extensions: None,
                extras: None,
            };
            textures.push(texture);

            material.emissive_texture = Some(Info {
                index: json::Index::new(textures.len() as u32 - 1),
                tex_coord: 0,
                extensions: None,
                extras: None,
            });
        }
        if let Some(image) = &self.material.occlusion_texture {
            let uri = image_to_uri(image)?;
            let image = Image {
                buffer_view: None,
                mime_type: None,
                name: None,
                uri: Some(uri),
                extensions: None,
                extras: None,
            };
            images.push(image);

            let texture = Texture {
                name: None,
                sampler: Some(json::Index::new(samplers.len() as u32 - 1)),
                source: json::Index::new(images.len() as u32 - 1),
                extensions: None,
                extras: None,
            };
            textures.push(texture);

            material.occlusion_texture = Some(OcclusionTexture {
                index: json::Index::new(textures.len() as u32 - 1),
                strength: StrengthFactor(1.0),
                tex_coord: 0,
                extensions: None,
                extras: None,
            });
        }
        if let Some(image) = &self.material.base_color_texture {
            let uri = image_to_uri(image)?;
            let image = Image {
                buffer_view: None,
                mime_type: None,
                name: None,
                uri: Some(uri),
                extensions: None,
                extras: None,
            };
            images.push(image);

            let texture = Texture {
                name: None,
                sampler: Some(json::Index::new(samplers.len() as u32 - 1)),
                source: json::Index::new(images.len() as u32 - 1),
                extensions: None,
                extras: None,
            };
            textures.push(texture);

            material.pbr_metallic_roughness.base_color_texture = Some(Info {
                index: json::Index::new(textures.len() as u32 - 1),
                tex_coord: 0,
                extensions: None,
                extras: None,
            });
        }
        if let Some(image) = &self.material.normal_map_texture {
            let uri = image_to_uri(image)?;
            let image = Image {
                buffer_view: None,
                mime_type: None,
                name: None,
                uri: Some(uri),
                extensions: None,
                extras: None,
            };
            images.push(image);

            let texture = Texture {
                name: None,
                sampler: Some(json::Index::new(samplers.len() as u32 - 1)),
                source: json::Index::new(images.len() as u32 - 1),
                extensions: None,
                extras: None,
            };
            textures.push(texture);

            material.normal_texture = Some(NormalTexture {
                index: json::Index::new(textures.len() as u32 - 1),
                scale: 1.0,
                tex_coord: 0,
                extensions: None,
                extras: None,
            });
        }
        if let Some(image) = &self.material.metallic_roughness_texture {
            let uri = image_to_uri(image)?;
            let image = Image {
                buffer_view: None,
                mime_type: None,
                name: None,
                uri: Some(uri),
                extensions: None,
                extras: None,
            };
            images.push(image);

            let texture = Texture {
                name: None,
                sampler: Some(json::Index::new(samplers.len() as u32 - 1)),
                source: json::Index::new(images.len() as u32 - 1),
                extensions: None,
                extras: None,
            };
            textures.push(texture);

            material.pbr_metallic_roughness.metallic_roughness_texture = Some(Info {
                index: json::Index::new(textures.len() as u32 - 1),
                tex_coord: 0,
                extensions: None,
                extras: None,
            });
        }

        let primitive = json::mesh::Primitive {
            attributes: {
                let mut map = std::collections::BTreeMap::new();
                map.insert(Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                map.insert(
                    Valid(json::mesh::Semantic::TexCoords(0)),
                    json::Index::new(1),
                );
                map
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: None,
            material: Some(json::Index::new(0)),
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        let mesh = json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            primitives: vec![primitive],
            weights: None,
        };

        let node = json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(0)),
            name: None,
            rotation: None,
            scale: None,
            translation: None,
            skin: None,
            weights: None,
        };

        // Root...

        let root = json::Root {
            accessors: vec![positions, tex_coords],
            buffers: vec![buffer],
            buffer_views: vec![buffer_view],
            meshes: vec![mesh],
            nodes: vec![node],
            scenes: vec![json::Scene {
                extensions: Default::default(),
                extras: Default::default(),
                name: None,
                nodes: vec![json::Index::new(0)],
            }],
            materials: vec![material],
            textures,
            images,
            samplers,
            ..Default::default()
        };

        // Output(write to file)...

        match output {
            GltfExporterOutput::Standard => {
                let writer = fs::File::create(path.clone()).expect("I/O error");
                json::serialize::to_writer_pretty(writer, &root).expect("Serialization error");

                let bin = to_padded_byte_vector(vertices.to_vec());
                let mut writer =
                    fs::File::create(dir_path.join(buffer_uri.clone())).expect("I/O error");
                writer.write_all(&bin).expect("I/O error");
            }
            GltfExporterOutput::Binary => {
                let json_string = json::serialize::to_string(&root).expect("Serialization error");
                let mut json_offset = json_string.len() as u32;
                align_to_multiple_of_four(&mut json_offset);
                let glb = gltf::binary::Glb {
                    header: gltf::binary::Header {
                        magic: *b"glTF",
                        version: 2,
                        length: json_offset + buffer_length as u32,
                    },
                    bin: Some(Cow::Owned(to_padded_byte_vector(vertices.to_vec()))),
                    json: Cow::Owned(json_string.into_bytes()),
                };
                let writer = std::fs::File::create(path).expect("I/O error");
                glb.to_writer(writer).expect("glTF binary output error");
            }
        }

        Ok(())
    }
}

impl GltfExporter {
    pub fn new(mesh: &Mesh, material: StandardMaterialWithImages) -> Result<Self> {
        let indices = mesh
            .indices()
            .ok_or_else(|| anyhow!("Could not get indices!"))?;

        let Some(VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            return Err(anyhow!("Could not get Float32x3 for ATTRIBUTE_POSITION!"));
        };
        let Some(VertexAttributeValues::Float32x2(uv_coords)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            return Err(anyhow!("Could not get Float32x2 for ATTRIBUTE_UV_0!"));
        };

        let mut exporter_vertices = vec![];
        for (i1, i2, i3) in indices.iter().tuples() {
            let tri_face_indices = [i1, i2, i3];
            for index in tri_face_indices.iter() {
                let Some(position) = positions.get(*index) else {
                    continue;
                };
                let Some(uv) = uv_coords.get(*index) else {
                    continue;
                };
                exporter_vertices.push(GltfExporterVertex {
                    position: *position,
                    uv: *uv,
                });
            }
        }

        Ok(GltfExporter {
            vertices: exporter_vertices,
            material,
        })
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, Hash, PartialEq)]
pub enum GltfExporterOutput {
    /// Output standard glTF.
    #[default]
    Standard,
    /// Output binary glTF.
    Binary,
}

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct GltfExporterVertex {
    position: [f32; 3],
    uv: [f32; 2],
}

/// Min/Max Bounds for array of [`GltfExporterVertex`]
#[derive(Copy, Clone, Debug)]
pub struct GltfExporterVertexBounds {
    /// Min values for pos, color, uv etc.
    pub min: GltfExporterVertex,
    /// Max values for pos, color, uv etc.
    pub max: GltfExporterVertex,
}

impl Default for GltfExporterVertexBounds {
    fn default() -> Self {
        Self {
            min: GltfExporterVertex {
                position: Vec3::MAX.to_array(),
                uv: Vec2::MAX.to_array(),
            },
            max: GltfExporterVertex {
                position: Vec3::MIN.to_array(),
                uv: Vec2::MIN.to_array(),
            },
        }
    }
}

impl From<&[GltfExporterVertex]> for GltfExporterVertexBounds {
    fn from(vertices: &[GltfExporterVertex]) -> Self {
        let mut bounds = GltfExporterVertexBounds::default();

        for GltfExporterVertex { position, uv } in vertices {
            for i in 0..3 {
                // pos
                bounds.min.position[i] = f32::min(bounds.min.position[i], position[i]);
                bounds.max.position[i] = f32::max(bounds.max.position[i], position[i]);
            }
            for i in 0..2 {
                // uvs
                bounds.min.uv[i] = f32::min(bounds.min.uv[i], uv[i]);
                bounds.max.uv[i] = f32::max(bounds.max.uv[i], uv[i]);
            }
        }

        bounds
    }
}

pub fn align_to_multiple_of_four(n: &mut u32) {
    *n = (*n + 3) & !3;
}

#[cfg(not(target_arch = "wasm32"))]
pub fn to_padded_byte_vector<T>(vec: Vec<T>) -> Vec<u8> {
    use std::mem;

    let byte_length = vec.len() * mem::size_of::<T>();
    let byte_capacity = vec.capacity() * mem::size_of::<T>();
    let alloc = vec.into_boxed_slice();
    let ptr = Box::<[T]>::into_raw(alloc) as *mut u8;
    let mut new_vec = unsafe { Vec::from_raw_parts(ptr, byte_length, byte_capacity) };
    while new_vec.len() % 4 != 0 {
        new_vec.push(0); // pad to multiple of four bytes
    }
    new_vec
}

pub fn image_to_uri(image: &Image) -> Result<String> {
    let rgba = RgbaImage::from_vec(image.width(), image.height(), image.data.clone())
        .ok_or_else(|| anyhow!("Could not create rgba from bevy Image!"))?;
    let img = image::DynamicImage::ImageRgba8(rgba);

    let mut buf = vec![];

    img.write_to(&mut Cursor::new(&mut buf), image::ImageOutputFormat::Png)?;
    let engine = STANDARD;
    let data_str = engine.encode(buf);
    let uri = format!("data:image/png;base64,{}", data_str);

    Ok(uri)
}

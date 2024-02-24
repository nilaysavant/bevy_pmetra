use anyhow::{anyhow, Result};
use bevy::{
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    utils::{HashMap, HashSet},
};
use image::imageops::FilterType;
use tiny_skia::{FillRule, IntSize, Paint, Path, PathBuilder, Pixmap, PixmapPaint, Stroke};
use truck_meshalgo::{
    rexport_polymesh::PolygonMesh,
    tessellation::{triangulation::MeshedShell, MeshedShape},
};
use truck_modeling::{FaceID, Surface, Vector2};

use crate::cad_core::{
    dimensions::AsBevyDVec2, meshing::BuildCadMeshedShell, tessellation::CadMeshedShell,
};

use super::CadShell;

/// Builder for building [`CadMaterialTextureSet`] that will
/// make up the [`StandardMaterial`] of the [`CadMesh`](super::CadMesh)
#[derive(Debug, Clone, Default)]
pub struct CadMaterialWithPolygonBuilder<P: Default + Clone> {
    // Inputs...
    pub params: P,
    pub shell: CadShell,
    pub textures: CadMaterialTextures<Option<Image>>,
    /// Config the output texture atlas image for each texture.
    pub atlas_image_config: CadTextureAtlasImageConfig,
    // State...
    cad_meshed_shell: Option<CadMeshedShell<Surface>>,
    tile_id_by_material_id: HashMap<CadMaterialId, UVec2>,
    material_texture_set: CadMaterialTextureSet<Option<Image>>,
}

impl<P: Default + Clone> CadMaterialWithPolygonBuilder<P> {
    pub fn new(
        params: P,
        shell: CadShell,
        textures: CadMaterialTextures<Option<Image>>,
    ) -> Result<Self> {
        let builder = Self {
            params,
            shell,
            textures,
            ..default()
        };
        Ok(builder)
    }

    pub fn set_atlas_image_config(
        &mut self,
        atlas_image_config: CadTextureAtlasImageConfig,
    ) -> Result<Self> {
        self.atlas_image_config = atlas_image_config;

        Ok(self.clone())
    }

    pub fn build_cad_meshed_shell(&mut self) -> Result<Self> {
        let cad_meshed_shell = self.shell.build_cad_meshed_shell()?;
        self.cad_meshed_shell = Some(cad_meshed_shell);

        Ok(self.clone())
    }

    /// Builds the textures and sets them on material set.
    ///
    /// Combines each set of textures (for `base_color`, `normal_map` etc.) into texture atlas images.
    /// These texture atlas images are then set to the material set.
    ///
    /// Also sets the map of material texture ids to generated tile ids.
    pub fn build_material_textures(&mut self) -> Result<Self> {
        let Self {
            textures,
            atlas_image_config,
            ..
        } = self;

        let texture_count = textures.len();
        let tile_dim_wo_margins = atlas_image_config.tile_dimensions_wo_margins(texture_count);
        let tile_ids = atlas_image_config.tile_ids(texture_count);
        // Map material_id to generated tile_ids...
        let tile_id_by_material_id = textures
            .keys()
            .enumerate()
            .map(|(idx, k)| {
                let tile_id = tile_ids.get(idx).unwrap();
                (k.clone(), *tile_id)
            })
            .collect::<HashMap<_, _>>();

        // For Base Color Texture...
        // Get pixmaps for all base color textures...
        let base_color_img_pixmaps = textures
            .iter()
            .filter_map(|(id, texture_set)| {
                let Some(base_color_img) = texture_set.base_color_texture.clone() else {
                    return None;
                };
                let Ok(mut dyn_img) = base_color_img.try_into_dynamic() else {
                    return None;
                };
                // resize image to tile dimensions...
                dyn_img = dyn_img.resize(
                    tile_dim_wo_margins.x,
                    tile_dim_wo_margins.y,
                    FilterType::Nearest,
                );
                // create pixmap from resized image...
                let Some(pixmap) = Pixmap::from_vec(
                    dyn_img.as_bytes().to_vec(),
                    IntSize::from_wh(dyn_img.width(), dyn_img.height()).unwrap(),
                ) else {
                    warn!(
                        "Could not get pixmap for `base_color_img` for CadMaterialId: {:?}",
                        id
                    );
                    return None;
                };
                Some((id.clone(), pixmap))
            })
            .collect::<HashMap<_, _>>();
        // Generate base color texture atlas image...
        let base_color_texture = Self::atlas_texture_from_pixmaps(
            &base_color_img_pixmaps,
            atlas_image_config,
            texture_count,
            &tile_id_by_material_id,
        )?;
        // set base color texture...
        self.material_texture_set.base_color_texture = Some(base_color_texture);

        // For Normal Texture...
        // Get pixmaps for all base color textures...
        let normal_img_pixmaps = textures
            .iter()
            .filter_map(|(id, texture_set)| {
                let Some(normal_map_img) = texture_set.normal_map_texture.clone() else {
                    return None;
                };
                let Ok(mut dyn_img) = normal_map_img.try_into_dynamic() else {
                    return None;
                };
                // resize image to tile dimensions...
                dyn_img = dyn_img.resize(
                    tile_dim_wo_margins.x,
                    tile_dim_wo_margins.y,
                    FilterType::Nearest,
                );
                // create pixmap from resized image...
                let Some(pixmap) = Pixmap::from_vec(
                    dyn_img.as_bytes().to_vec(),
                    IntSize::from_wh(dyn_img.width(), dyn_img.height()).unwrap(),
                ) else {
                    warn!(
                        "Could not get pixmap for `normal_map_img` for CadMaterialId: {:?}",
                        id
                    );
                    return None;
                };
                Some((id.clone(), pixmap))
            })
            .collect::<HashMap<_, _>>();
        // Generate base color texture atlas image...
        let normal_map_texture = Self::atlas_texture_from_pixmaps(
            &normal_img_pixmaps,
            atlas_image_config,
            texture_count,
            &tile_id_by_material_id,
        )?;
        // set base color texture...
        self.material_texture_set.normal_map_texture = Some(normal_map_texture);

        // For Metallic/Roughness Texture...
        // Get pixmaps for all base color textures...
        let metallic_roughness_img_pixmaps = textures
            .iter()
            .filter_map(|(id, texture_set)| {
                let Some(img) = texture_set.metallic_roughness_texture.clone() else {
                    return None;
                };
                let Ok(mut dyn_img) = img.try_into_dynamic() else {
                    return None;
                };
                // resize image to tile dimensions...
                dyn_img = dyn_img.resize(
                    tile_dim_wo_margins.x,
                    tile_dim_wo_margins.y,
                    FilterType::Nearest,
                );
                // create pixmap from resized image...
                let Some(pixmap) = Pixmap::from_vec(
                    dyn_img.as_bytes().to_vec(),
                    IntSize::from_wh(dyn_img.width(), dyn_img.height()).unwrap(),
                ) else {
                    warn!(
                        "Could not get pixmap for `metallic_roughness_texture` for CadMaterialId: {:?}",
                        id
                    );
                    return None;
                };
                Some((id.clone(), pixmap))
            })
            .collect::<HashMap<_, _>>();
        // Generate base color texture atlas image...
        let metallic_roughness_texture = Self::atlas_texture_from_pixmaps(
            &metallic_roughness_img_pixmaps,
            atlas_image_config,
            texture_count,
            &tile_id_by_material_id,
        )?;
        // set base color texture...
        self.material_texture_set.metallic_roughness_texture = Some(metallic_roughness_texture);

        // Set the mapping of material_id to tile_id...
        self.tile_id_by_material_id = tile_id_by_material_id;

        Ok(self.clone())
    }

    fn atlas_texture_from_pixmaps(
        pixmaps_by_material_id: &HashMap<CadMaterialId, Pixmap>,
        atlas_image_config: &CadTextureAtlasImageConfig,
        texture_count: usize,
        tile_id_by_material_id: &HashMap<CadMaterialId, UVec2>,
    ) -> Result<Image> {
        let mut texture_pixmap = Pixmap::new(
            atlas_image_config.image_dim.x,
            atlas_image_config.image_dim.y,
        )
        .ok_or_else(|| anyhow!("Could not create pixmap for given texture image sizes"))?;
        let paint = PixmapPaint::default();
        let tile_dim = atlas_image_config.tile_dimensions(texture_count);
        for (id, pixmap) in pixmaps_by_material_id.iter() {
            let tile_id = tile_id_by_material_id.get(id).unwrap();
            let offset = atlas_image_config.tile_offset_from_id_dim(*tile_id, tile_dim);

            texture_pixmap.draw_pixmap(
                offset.x as i32,
                offset.y as i32,
                pixmap.as_ref(),
                &paint,
                tiny_skia::Transform::default(),
                None,
            );
        }
        // // Save image for debug purposes...
        // let start = SystemTime::now();
        // let since_the_epoch = start
        //     .duration_since(UNIX_EPOCH)
        //     .expect("Time went backwards");
        // let in_ms =
        //     since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;
        // texture_pixmap
        //     .save_png(format!("exports/temp/cad_diffuse_{}.png", in_ms))
        //     .unwrap();
        let base_color_texture = Image::new(
            Extent3d {
                width: atlas_image_config.image_dim.x,
                height: atlas_image_config.image_dim.y,
                ..default()
            },
            TextureDimension::D2,
            texture_pixmap.data().to_vec(),
            TextureFormat::Rgba8Unorm,
        );
        Ok(base_color_texture)
    }

    pub fn apply_material_to_faces(
        &mut self,
        material_id: CadMaterialId,
        faces: &HashSet<FaceID>,
        face_uv_confs: &FaceUvConfs,
    ) -> Result<Self> {
        let Self {
            textures,
            atlas_image_config,
            tile_id_by_material_id,
            ..
        } = self;
        let CadMeshedShell {
            mut meshed_shell,
            meshed_faces_by_brep_face,
        } = match self.cad_meshed_shell.clone() {
            Some(cad_meshed_shell) => cad_meshed_shell,
            None => self.shell.build_cad_meshed_shell()?,
        };

        // Move uv coords/uv faces in meshed shell to desired pos...
        for (face_id, meshed_face) in meshed_faces_by_brep_face.iter() {
            if !faces.contains(face_id) {
                continue;
            }
            let face_uv_conf = face_uv_confs.get(face_id);
            let Some(meshed_shell_face) = meshed_shell
                .face_iter_mut()
                .find(|f| f.id() == meshed_face.id())
            else {
                continue;
            };
            let Some(mut polygon_mesh) = meshed_shell_face.surface() else {
                continue;
            };

            let texture_count = textures.len();
            let image_tile_dim = atlas_image_config.tile_dimensions(texture_count);
            let image_tile_dim_wo_margins =
                atlas_image_config.tile_dimensions_wo_margins(texture_count);
            let tile_id = tile_id_by_material_id.get(&material_id).unwrap();

            // move uv coords in polygon_mesh...
            polygon_mesh.uv_coords_mut().iter_mut().for_each(|u| {
                let scale = image_tile_dim_wo_margins.as_dvec2();
                let offset = atlas_image_config
                    .tile_offset_from_id_dim(*tile_id, image_tile_dim)
                    .as_dvec2();
                let mut new_u = u.as_bevy_dvec2();
                // Apply any transform if configured...
                if let Some(FaceUvConf { transform }) = face_uv_conf {
                    let new_u_vec3 = transform.transform_point(new_u.extend(0.).as_vec3());
                    new_u = new_u_vec3.truncate().as_dvec2();
                }
                // Scale and offset the uv coord as per tile...
                new_u = (new_u * scale + offset) / atlas_image_config.image_dim.as_dvec2();

                *u = new_u.to_array().into();
            });
            // Update the meshed shell face with updated polygon mesh as surface
            meshed_shell_face.set_surface(Some(polygon_mesh));
        }

        // Update meshed shell in builder...
        if let Some(cad_meshed_shell) = self.cad_meshed_shell.as_mut() {
            cad_meshed_shell.meshed_shell = meshed_shell;
        } else {
            self.cad_meshed_shell = Some(CadMeshedShell {
                meshed_shell,
                meshed_faces_by_brep_face: meshed_faces_by_brep_face.clone(),
            })
        }

        Ok(self.clone())
    }

    pub fn debug_save_uv_image(
        &mut self,
        save_path: String,
        exclude_faces: &HashSet<FaceID>,
    ) -> Result<Self> {
        let CadMeshedShell {
            meshed_shell,
            meshed_faces_by_brep_face,
            ..
        } = match self.cad_meshed_shell.clone() {
            Some(cad_meshed_shell) => cad_meshed_shell,
            None => self.shell.build_cad_meshed_shell()?,
        };

        let excluded_meshed_faces = exclude_faces
            .iter()
            .flat_map(|id| meshed_faces_by_brep_face.get(id))
            .collect::<HashSet<_>>();

        let meshed_shell = meshed_shell
            .face_iter()
            .flat_map(|f| {
                if excluded_meshed_faces.contains(f) {
                    return None;
                }
                Some(f.clone())
            })
            .collect::<MeshedShell>();

        // Setup image dimensions and margins...
        let atlas_image_config = CadTextureAtlasImageConfig::default();
        let CadTextureAtlasImageConfig { image_dim, .. } = &atlas_image_config;

        // For Debug UV image...
        let meshed_shell_polygon = meshed_shell.to_polygon();
        let meshed_shell_polygon_uv_coords = meshed_shell_polygon.uv_coords();
        let cad_texture_path_vec = meshed_shell_polygon
            .face_iter()
            .filter_map(|face_verts| {
                let uv_coords_w_idx_vec = face_verts
                    .iter()
                    .filter_map(|v| {
                        let Some(idx) = v.uv else {
                            return None;
                        };
                        let Some(uv_coord) = meshed_shell_polygon_uv_coords.get(idx) else {
                            return None;
                        };
                        Some(UvCoordWithIdx {
                            idx,
                            uv_coord: *uv_coord,
                        })
                    })
                    .collect::<Vec<_>>();

                let Some(path) = UvCoordsWithIndices(uv_coords_w_idx_vec)
                    .try_build_skia_path(image_dim.as_ivec2(), UVec2::ZERO.as_ivec2())
                    .ok()
                else {
                    return None;
                };
                Some(CadTexturePath {
                    path,
                    kind: CadTexturePathKind::Stroke(Color::WHITE),
                })
            })
            .collect::<Vec<_>>();
        let cad_texture_paths = CadTexturePaths(cad_texture_path_vec);
        // Create pixmap from texture paths...
        let pixmap = cad_texture_paths.create_pixmap(*image_dim);

        // [For Debugging] Save as diffuse image...
        pixmap.save_png(save_path).unwrap();

        Ok(self.clone())
    }

    #[deprecated]
    /// Builds the debug texture and sets it to `base_color_texture`.
    pub fn build_debug_texture_base_color(&mut self) -> Result<Self> {
        let CadMeshedShell {
            mut meshed_shell,
            meshed_faces_by_brep_face,
        } = match self.cad_meshed_shell.clone() {
            Some(cad_meshed_shell) => cad_meshed_shell,
            None => self.shell.build_cad_meshed_shell()?,
        };

        // Setup image dimensions and margins...
        let texture_count = meshed_faces_by_brep_face.keys().count();
        let atlas_image_config = CadTextureAtlasImageConfig::default();
        let CadTextureAtlasImageConfig {
            image_dim,
            tile_margins,
            ..
        } = &atlas_image_config;
        let image_tile_dim = atlas_image_config.tile_dimensions(texture_count);
        let image_tile_dim_wo_margins =
            atlas_image_config.tile_dimensions_wo_margins(texture_count);
        let image_tile_ids = atlas_image_config.tile_ids(texture_count);

        let face_id_count = meshed_faces_by_brep_face.keys().count();
        let colors_w_tile_id_by_face_id = meshed_faces_by_brep_face
            .keys()
            .enumerate()
            .map(|(idx, face_id)| {
                (
                    face_id,
                    (
                        Color::RED.with_h((360. / face_id_count as f32) * idx as f32),
                        *image_tile_ids.get(idx).unwrap(),
                    ),
                )
            })
            .collect::<HashMap<_, _>>();

        // Move uv coords/uv faces in meshed shell to desired pos...
        for (face_id, meshed_face) in meshed_faces_by_brep_face.iter() {
            let (stroke_color, tile_id) = *colors_w_tile_id_by_face_id.get(face_id).unwrap();
            let Some(meshed_shell_face) = meshed_shell
                .face_iter_mut()
                .find(|f| f.id() == meshed_face.id())
            else {
                continue;
            };
            let Some(mut polygon_mesh) = meshed_shell_face.surface() else {
                continue;
            };
            // move uv coords in polygon_mesh...
            polygon_mesh.uv_coords_mut().iter_mut().for_each(|u| {
                let scale = image_tile_dim_wo_margins.as_dvec2();
                let offset = atlas_image_config
                    .tile_offset_from_id_dim(tile_id, image_tile_dim)
                    .as_dvec2();
                let new_u = ((u.as_bevy_dvec2() * scale + offset) / image_dim.as_dvec2())
                    .to_array()
                    .into();
                *u = new_u;
            });
            // Update the meshed shell face with updated polygon mesh as surface
            meshed_shell_face.set_surface(Some(polygon_mesh));
        }

        // For Debug UV image...
        let meshed_shell_polygon = meshed_shell.to_polygon();
        let meshed_shell_polygon_uv_coords = meshed_shell_polygon.uv_coords();
        let cad_texture_path_vec = meshed_shell_polygon
            .face_iter()
            .filter_map(|face_verts| {
                let uv_coords_w_idx_vec = face_verts
                    .iter()
                    .filter_map(|v| {
                        let Some(idx) = v.uv else {
                            return None;
                        };
                        let Some(uv_coord) = meshed_shell_polygon_uv_coords.get(idx) else {
                            return None;
                        };
                        Some(UvCoordWithIdx {
                            idx,
                            uv_coord: *uv_coord,
                        })
                    })
                    .collect::<Vec<_>>();

                let Some(path) = UvCoordsWithIndices(uv_coords_w_idx_vec)
                    .try_build_skia_path(image_dim.as_ivec2(), UVec2::ZERO.as_ivec2())
                    .ok()
                else {
                    return None;
                };
                Some(CadTexturePath {
                    path,
                    kind: CadTexturePathKind::Stroke(Color::WHITE),
                })
            })
            .collect::<Vec<_>>();
        let cad_texture_paths = CadTexturePaths(cad_texture_path_vec);
        // Create pixmap from texture paths...
        let pixmap = cad_texture_paths.create_pixmap(*image_dim);

        // // [For Debugging] Save as diffuse image...
        // pixmap.save_png("exports/temp/cad_uv_debug.png").unwrap();

        // For Diffuse UV image...
        // Calculate all paths to be drawn on canvas...
        let cad_texture_path_vec = meshed_faces_by_brep_face
            .iter()
            .filter_map(|(face_id, meshed_face)| {
                let (stroke_color, tile_id) = *colors_w_tile_id_by_face_id.get(face_id).unwrap();
                let Some(face_polygon_mesh) = meshed_face.surface() else {
                    return None;
                };
                let face_polygon_uv_coords = face_polygon_mesh.uv_coords();
                Some(
                    face_polygon_mesh
                        .face_iter()
                        .filter_map(|face_verts| {
                            let uv_coords_w_idx_vec = face_verts
                                .iter()
                                .filter_map(|v| {
                                    let Some(idx) = v.uv else {
                                        return None;
                                    };
                                    let Some(uv_coord) = face_polygon_uv_coords.get(idx) else {
                                        return None;
                                    };
                                    Some(UvCoordWithIdx {
                                        idx,
                                        uv_coord: *uv_coord,
                                    })
                                })
                                .collect::<Vec<_>>();

                            let Some(path) = UvCoordsWithIndices(uv_coords_w_idx_vec)
                                .try_build_skia_path(image_dim.as_ivec2(), IVec2::ZERO)
                                .ok()
                            else {
                                return None;
                            };
                            Some(CadTexturePath {
                                path,
                                kind: CadTexturePathKind::Fill(stroke_color),
                            })
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .flatten()
            .collect::<Vec<_>>();
        let cad_texture_paths = CadTexturePaths(cad_texture_path_vec);
        // Create pixmap from texture paths...
        let pixmap = cad_texture_paths.create_pixmap(*image_dim);

        // // [For Debugging] Save as diffuse image...
        // pixmap.save_png("exports/temp/cad_diffuse.png").unwrap();

        // Create bevy image from pixmap png data...
        let base_color_texture = Image::new(
            Extent3d {
                width: image_dim.x,
                height: image_dim.y,
                ..default()
            },
            TextureDimension::D2,
            pixmap.data().to_vec(),
            TextureFormat::Rgba8Unorm,
        );
        // set diffuse texture in builder...
        self.material_texture_set.base_color_texture = Some(base_color_texture);

        // Update meshed shell in builder...
        if let Some(cad_meshed_shell) = self.cad_meshed_shell.as_mut() {
            cad_meshed_shell.meshed_shell = meshed_shell;
        } else {
            self.cad_meshed_shell = Some(CadMeshedShell {
                meshed_shell,
                meshed_faces_by_brep_face: meshed_faces_by_brep_face.clone(),
            })
        }

        Ok(self.clone())
    }

    pub fn build(&self) -> Result<(CadMaterialTextureSet<Option<Image>>, PolygonMesh)> {
        let CadMeshedShell { meshed_shell, .. } = match self.cad_meshed_shell.clone() {
            Some(cad_meshed_shell) => cad_meshed_shell,
            None => self.shell.build_cad_meshed_shell()?,
        };
        let polygon_mesh = meshed_shell.to_polygon();

        Ok((self.material_texture_set.clone(), polygon_mesh))
    }
}

#[derive(Debug, Clone, Deref, DerefMut)]
pub struct UvCoordsWithIndices(Vec<UvCoordWithIdx>);

impl UvCoordsWithIndices {
    /// Try building a [`tiny_skia::Path`] from [`UvCoordsWithIndices`].
    ///
    /// Provide `scale` and `offset` to be applied to the path.
    pub fn try_build_skia_path(&self, scale: IVec2, offset: IVec2) -> Result<Path> {
        let mut pb = PathBuilder::new();
        let len = self.len();
        for (idx, UvCoordWithIdx { uv_coord, .. }) in self.iter().enumerate() {
            if idx == 0 {
                pb.move_to(
                    uv_coord.x as f32 * scale.x as f32 + offset.x as f32,
                    uv_coord.y as f32 * scale.y as f32 + offset.y as f32,
                );
            } else if idx == len - 1 {
                pb.line_to(
                    uv_coord.x as f32 * scale.x as f32 + offset.x as f32,
                    uv_coord.y as f32 * scale.y as f32 + offset.y as f32,
                );
                pb.close();
            } else {
                pb.line_to(
                    uv_coord.x as f32 * scale.x as f32 + offset.x as f32,
                    uv_coord.y as f32 * scale.y as f32 + offset.y as f32,
                );
            }
        }
        pb.finish()
            .ok_or_else(|| anyhow!("Could not construct path from UvCoordsWithIndices"))
    }
}

#[derive(Debug, Clone)]
pub struct UvCoordWithIdx {
    pub idx: usize,
    pub uv_coord: Vector2,
}

/// Base set of material textures.
/// Used for procedurally generating materials for CAD models.
#[derive(Debug, Clone, Default, Deref, DerefMut, Component, Reflect)]
pub struct CadMaterialTextures<T: Default>(pub HashMap<CadMaterialId, CadMaterialTextureSet<T>>);

impl CadMaterialTextures<Option<Handle<Image>>> {
    /// Resolves textures from `Handle<Image>` to [`Image`].
    pub fn resolve_image_handles(
        &self,
        images: &Assets<Image>,
    ) -> CadMaterialTextures<Option<Image>> {
        let resolved_texture_sets = self
            .iter()
            .map(|(id, texture_set)| {
                let resolved_texture_set = texture_set.resolve_image_handles(images);
                (id.clone(), resolved_texture_set)
            })
            .collect::<HashMap<_, _>>();

        CadMaterialTextures::<Option<Image>>(resolved_texture_sets)
    }
}

/// ID for the [`CadMaterialTextureSet`] in [`CadMaterialTextures`]' [`HashMap`].
#[derive(Debug, Clone, Deref, DerefMut, Hash, PartialEq, Eq, Component, Reflect)]
pub struct CadMaterialId(pub String);

impl From<String> for CadMaterialId {
    fn from(value: String) -> Self {
        CadMaterialId(value)
    }
}

/// Generic struct for set of `textures` that make up a [`StandardMaterial`].
///
/// `T` is that type of texture, can usually be:
/// - `Option<Image>`
/// - `Option<Handle<Image>>`
///
/// This allows us to convert between unresolved [`Handle`]s to resolved [`Image`]s.
///  
#[derive(Debug, Clone, Default, Reflect)]
pub struct CadMaterialTextureSet<T: Default> {
    pub base_color_texture: T,
    pub emissive_texture: T,
    pub metallic_roughness_texture: T,
    pub normal_map_texture: T,
    pub occlusion_texture: T,
}

impl CadMaterialTextureSet<Option<Handle<Image>>> {
    /// Resolves textures from `Handle<Image>` to [`Image`].
    pub fn resolve_image_handles(
        &self,
        images: &Assets<Image>,
    ) -> CadMaterialTextureSet<Option<Image>> {
        let Self {
            base_color_texture,
            emissive_texture,
            metallic_roughness_texture,
            normal_map_texture,
            occlusion_texture,
            ..
        } = self;

        CadMaterialTextureSet::<Option<Image>> {
            base_color_texture: Self::get_image_from_opt_hdl(base_color_texture, images),
            emissive_texture: Self::get_image_from_opt_hdl(emissive_texture, images),
            metallic_roughness_texture: Self::get_image_from_opt_hdl(
                metallic_roughness_texture,
                images,
            ),
            normal_map_texture: Self::get_image_from_opt_hdl(normal_map_texture, images),
            occlusion_texture: Self::get_image_from_opt_hdl(occlusion_texture, images),
        }
    }

    fn get_image_from_opt_hdl(
        optional_image_hdl: &Option<Handle<Image>>,
        images: &Assets<Image>,
    ) -> Option<Image> {
        let image = match optional_image_hdl.clone() {
            Some(image_hdl) => images.get(image_hdl).cloned(),
            None => None,
        };
        image
    }
}

impl CadMaterialTextureSet<Option<Image>> {
    /// Creates [`Handle`]s for stored [`Image`] textures.
    pub fn create_image_handles(
        &self,
        images: &mut Assets<Image>,
    ) -> CadMaterialTextureSet<Option<Handle<Image>>> {
        let Self {
            base_color_texture,
            emissive_texture,
            metallic_roughness_texture,
            normal_map_texture,
            occlusion_texture,
            ..
        } = self;

        CadMaterialTextureSet::<Option<Handle<Image>>> {
            base_color_texture: Self::create_hdl_for_image(base_color_texture, images),
            emissive_texture: Self::create_hdl_for_image(emissive_texture, images),
            metallic_roughness_texture: Self::create_hdl_for_image(
                metallic_roughness_texture,
                images,
            ),
            normal_map_texture: Self::create_hdl_for_image(normal_map_texture, images),
            occlusion_texture: Self::create_hdl_for_image(occlusion_texture, images),
        }
    }

    fn create_hdl_for_image(
        optional_image: &Option<Image>,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        let image_hdl = optional_image.clone().map(|image| images.add(image));
        image_hdl
    }
}

/// Used for configuring the Texture Atlas image.
#[derive(Debug, Clone, Reflect)]
pub struct CadTextureAtlasImageConfig {
    /// Dimensions of the full texture atlas image.
    pub image_dim: UVec2,
    /// Margins of each tile in the image.
    pub tile_margins: UVec2,
}

impl Default for CadTextureAtlasImageConfig {
    fn default() -> Self {
        Self {
            image_dim: UVec2::splat(1000),
            tile_margins: UVec2::splat(10),
        }
    }
}

impl CadTextureAtlasImageConfig {
    pub fn tile_dimensions(&self, texture_count: usize) -> UVec2 {
        Self::get_image_tile_dimensions(texture_count, self.image_dim)
    }

    pub fn tile_dimensions_wo_margins(&self, texture_count: usize) -> UVec2 {
        self.tile_dimensions(texture_count) - self.tile_margins * 2
    }

    pub fn tile_ids(&self, texture_count: usize) -> Vec<UVec2> {
        Self::generate_tile_ids(texture_count)
    }

    pub fn tile_offset_from_id_dim(&self, tile_id: UVec2, tile_dimensions: UVec2) -> UVec2 {
        tile_dimensions * tile_id + self.tile_margins
    }

    /// Gets the tile dimensions for the num of textures that need to fit in the image with `image_dim`.
    ///
    /// Divides the `image_dim` into tiles such that each texture can be assigned to a single tile.
    ///
    fn get_image_tile_dimensions(texture_count: usize, image_dim: UVec2) -> UVec2 {
        // Calc the sqrt of the total tiles on the final UV texture.
        // This is thus the side length for dividing the texture size into tiles...
        let tile_count_sqrt = (texture_count as f32).sqrt().ceil() as usize;
        let image_tile_dim = image_dim / tile_count_sqrt as u32;

        image_tile_dim
    }

    /// Generate tile ids for given `tile_count`.
    ///
    /// PS: `tile_count` has to be a **perfect square**.
    ///
    /// # Example
    ///
    /// For `tile_count = 2`, this will generate:
    /// - `(0,0)`: bottom-left.
    /// - `(0,1)`: bottom-right.
    /// - `(1,0)`: top-left.
    /// - `(1,1)`: top-right.
    ///
    fn generate_tile_ids(tile_count: usize) -> Vec<UVec2> {
        let mut tile_ids = vec![];
        let side_length = (tile_count as f32).sqrt().ceil() as usize;

        for col in 0..side_length {
            for row in 0..side_length {
                tile_ids.push(UVec2::new(col as u32, row as u32));
            }
        }

        tile_ids
    }
}

/// Configs for UV coords of faces.
#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct FaceUvConfs(pub HashMap<FaceID, FaceUvConf>);

/// UV coord config for [`truck_modeling::Face`].
#[derive(Debug, Clone, Default)]
pub struct FaceUvConf {
    /// Transform the UV face relative to atlas tile.
    ///
    /// Uses only `x` and `y` components. (`z` is ignored).
    pub transform: Transform,
}

/// Used for generating textures for CAD stuff. List of [`CadTexturePath`]s.
#[derive(Debug, Clone, Deref, DerefMut, Default)]
pub struct CadTexturePaths(pub Vec<CadTexturePath>);

impl CadTexturePaths {
    /// Creates [`Pixmap`] (a kind of image pixels buffer) from [`CadTexturePaths`].
    pub fn create_pixmap(&self, image_dimensions: UVec2) -> Pixmap {
        // Setup stroke/fill paints...
        let mut path_stroke_paint = Paint::default();
        let mut path_fill_paint = Paint::default();

        // Create pixmap canvas for image...
        let mut pixmap = Pixmap::new(image_dimensions.x, image_dimensions.y).unwrap();
        // Draw paths on canvas...
        for CadTexturePath { path, kind } in self.iter() {
            match kind {
                CadTexturePathKind::Fill(color) => {
                    let fill_color = color.as_rgba_u8();
                    path_fill_paint.set_color_rgba8(
                        fill_color[0],
                        fill_color[1],
                        fill_color[2],
                        fill_color[3],
                    );
                    pixmap.fill_path(
                        path,
                        &path_fill_paint,
                        FillRule::Winding,
                        tiny_skia::Transform::default(),
                        None,
                    );
                }
                CadTexturePathKind::Stroke(color) => {
                    let stroke_color = color.as_rgba_u8();
                    path_stroke_paint.set_color_rgba8(
                        stroke_color[0],
                        stroke_color[1],
                        stroke_color[2],
                        stroke_color[3],
                    );
                    pixmap.stroke_path(
                        path,
                        &path_stroke_paint,
                        &Stroke::default(),
                        // FillRule::Winding,
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
            }
        }

        pixmap
    }
}

/// Wraps [`Path`]. Used to draw paths with stroke/fill for Cad generated Texture.
#[derive(Debug, Clone)]
pub struct CadTexturePath {
    pub path: Path,
    pub kind: CadTexturePathKind,
}

#[derive(Debug, Clone)]
pub enum CadTexturePathKind {
    Fill(Color),
    Stroke(Color),
}

mod tests {
    use super::*;

    #[test]
    fn test_image_tile_dim_calc() {
        let texture_count = 3;
        let atlas_config = CadTextureAtlasImageConfig::default();

        let image_tile_dimensions = atlas_config.tile_dimensions(texture_count);

        assert_eq!(image_tile_dimensions, UVec2::splat(500));

        let image_tile_dimensions_wo_margins =
            atlas_config.tile_dimensions_wo_margins(texture_count);

        assert_eq!(image_tile_dimensions_wo_margins, UVec2::splat(480));
    }

    #[test]
    fn test_generate_tile_ids() {
        let texture_count = 3;
        let atlas_config = CadTextureAtlasImageConfig::default();

        let tile_ids = atlas_config.tile_ids(texture_count);

        assert_eq!(
            tile_ids,
            vec![
                UVec2::new(0, 0),
                UVec2::new(0, 1),
                UVec2::new(1, 0),
                UVec2::new(1, 1),
            ]
        );
    }
}

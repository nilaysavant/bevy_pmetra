use anyhow::Result;
use bevy::{
    asset::{Assets, Handle},
    prelude::StandardMaterial,
    render::{color::Color, render_resource::Face, texture::Image},
};
use gltf::{
    json::{
        self,
        material::{EmissiveFactor, PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor},
    },
    material::AlphaMode,
};
use json::{validation::Checked::Valid, Material};

pub trait ToGltfMaterial {
    /// Convert to [`gltf::json::Material`] but with no textures.
    fn to_gltf_material_no_textures(&self) -> Result<Material>;
}

impl ToGltfMaterial for StandardMaterialWithImages {
    /// [`StandardMaterial`] to [`Material`].
    fn to_gltf_material_no_textures(&self) -> Result<Material> {
        let mut material = Material::default();

        material.alpha_mode = match self.alpha_mode {
            bevy::pbr::AlphaMode::Opaque => Valid(AlphaMode::Opaque),
            bevy::pbr::AlphaMode::Mask(_) => Valid(AlphaMode::Mask),
            bevy::pbr::AlphaMode::Blend => Valid(AlphaMode::Blend),
            _ => Valid(AlphaMode::default()),
        };
        material.double_sided = self.double_sided;
        let emissive = self.emissive.as_rgba_f32();
        material.emissive_factor = EmissiveFactor([emissive[0], emissive[1], emissive[2]]);
        material.pbr_metallic_roughness = PbrMetallicRoughness {
            base_color_factor: PbrBaseColorFactor(self.base_color.as_rgba_f32()),
            metallic_factor: StrengthFactor(self.metallic),
            roughness_factor: StrengthFactor(self.perceptual_roughness),
            ..Default::default()
        };

        Ok(material)
    }
}

#[derive(Debug, Clone)]
pub struct StandardMaterialWithImages {
    pub base_color: Color,
    pub base_color_texture: Option<Image>,
    pub emissive: Color,
    pub emissive_texture: Option<Image>,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub metallic_roughness_texture: Option<Image>,
    pub reflectance: f32,
    pub diffuse_transmission: f32,
    pub specular_transmission: f32,
    pub normal_map_texture: Option<Image>,
    pub occlusion_texture: Option<Image>,
    pub double_sided: bool,
    pub unlit: bool,
    pub alpha_mode: bevy::pbr::AlphaMode,
}

impl StandardMaterialWithImages {
    pub fn from_standard_material(
        standard_material: StandardMaterial,
        images: &Assets<Image>,
    ) -> Self {
        let StandardMaterial {
            base_color,
            base_color_texture,
            emissive,
            emissive_texture,
            perceptual_roughness,
            metallic,
            metallic_roughness_texture,
            reflectance,
            diffuse_transmission,
            specular_transmission,
            normal_map_texture,
            occlusion_texture,
            double_sided,
            unlit,
            alpha_mode,
            ..
        } = standard_material;

        Self {
            base_color,
            base_color_texture: get_image_from_opt_hdl(&base_color_texture, images),
            emissive,
            emissive_texture: get_image_from_opt_hdl(&emissive_texture, images),
            perceptual_roughness,
            metallic,
            metallic_roughness_texture: get_image_from_opt_hdl(&metallic_roughness_texture, images),
            reflectance,
            diffuse_transmission,
            specular_transmission,
            normal_map_texture: get_image_from_opt_hdl(&normal_map_texture, images),
            occlusion_texture: get_image_from_opt_hdl(&occlusion_texture, images),
            double_sided,
            unlit,
            alpha_mode,
        }
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

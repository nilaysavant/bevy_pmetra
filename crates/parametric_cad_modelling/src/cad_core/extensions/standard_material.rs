use bevy::prelude::*;

use crate::cad_core::builders::CadMaterialTextureSet;

pub trait StandardMaterialExtensions {
    /// Update texture images from [`CadMaterialTextureSet`].
    fn update_textures_from_set(&mut self, textures: &CadMaterialTextureSet<Option<Handle<Image>>>);
}

impl StandardMaterialExtensions for StandardMaterial {
    fn update_textures_from_set(
        &mut self,
        textures_set: &CadMaterialTextureSet<Option<Handle<Image>>>,
    ) {
        let CadMaterialTextureSet {
            base_color_texture,
            emissive_texture,
            metallic_roughness_texture,
            normal_map_texture,
            occlusion_texture,
        } = textures_set.clone();
        self.base_color_texture = base_color_texture;
        self.emissive_texture = emissive_texture;
        self.metallic_roughness_texture = metallic_roughness_texture;
        self.normal_map_texture = normal_map_texture;
        self.occlusion_texture = occlusion_texture;
    }
}

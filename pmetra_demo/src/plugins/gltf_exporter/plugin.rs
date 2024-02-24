use bevy::{prelude::*, tasks::IoTaskPool};
use bevy_pmetra::{
    bevy_plugin::components::cad::CadGeneratedMesh, cad_core::builders::CadMeshName,
};

use crate::plugins::gltf_exporter::gltf::converters::StandardMaterialWithImages;

use super::gltf::{GltfExporter, GltfExporterOutput};

pub struct GltfExporterPlugin;

impl Plugin for GltfExporterPlugin {
    fn build(&self, app: &mut App) {
        // Run only in native...
        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(Update, save_mesh);

        // rest...
        app // app
            .add_systems(Startup, || info!("GltfExporterPlugin Started..."));
    }
}

pub fn save_mesh(
    key_input: Res<Input<KeyCode>>,
    selected_meshes: Query<
        (&CadMeshName, &Handle<Mesh>, &Handle<StandardMaterial>),
        With<CadGeneratedMesh>,
    >,
    meshes: Res<Assets<Mesh>>,
    materials: Res<Assets<StandardMaterial>>,
    images: Res<Assets<Image>>,
) {
    if !(key_input.pressed(KeyCode::ControlLeft) && key_input.just_pressed(KeyCode::S)) {
        return;
    }

    for (CadMeshName(name), mesh_hdl, material_hdl) in selected_meshes.iter() {
        info!("Saving mesh... {}", name);

        let Some(mesh) = meshes.get(mesh_hdl) else {
            continue;
        };
        let Some(material) = materials.get(material_hdl) else {
            continue;
        };
        let material_w_images =
            StandardMaterialWithImages::from_standard_material(material.clone(), &images);

        let Ok(gltf_exporter) = GltfExporter::new(mesh, material_w_images.clone()) else {
            error!("Could not create new GltfExporter!");
            continue;
        };
        let file_name = name.to_string() + ".glb";

        // Writing the scene to a new file. Using a task to avoid calling the filesystem APIs in a system
        // as they are blocking
        // This can't work in WASM as there is no filesystem access
        #[cfg(not(target_arch = "wasm32"))]
        IoTaskPool::get()
            .spawn(async move {
                gltf_exporter.export(
                    format!("exports/temp/{}", file_name),
                    GltfExporterOutput::Binary,
                )
            })
            .detach();
    }
}

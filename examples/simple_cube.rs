use bevy::{color::palettes::css, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::shell::ShellCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Result},
        truck_modeling::{builder, ParametricSurface3D, Point3, Shell, Vector3, Vertex},
    },
};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

fn main() {
    App::new() // app
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::BLACK))
        // orbit camera...
        .add_plugins((LookTransformPlugin, OrbitCameraPlugin::default()))
        // pmetra...
        .add_plugins((
            PmetraBasePlugin::default(),
            PmetraModellingPlugin::<SimpleCube>::default(),
            PmetraInteractionsPlugin::<SimpleCube>::default(),
        ))
        // scene...
        .add_systems(Startup, scene_setup)
        // debug...
        .add_systems(Update, render_origin_gizmo)
        // rest...
        .add_systems(Startup, || info!("SimpleCube example started!"))
        .run();
}

fn scene_setup(
    mut commands: Commands,
    mut spawn_simple_cube: EventWriter<GenerateCadModel<SimpleCube>>,
) {
    commands
        .spawn((
            Camera3d::default(),
            Projection::Perspective(PerspectiveProjection {
                near: 0.001,
                ..default()
            }),
        ))
        .insert((
            OrbitCameraBundle::new(
                OrbitCameraController {
                    mouse_rotate_sensitivity: Vec2::splat(0.25),
                    mouse_translate_sensitivity: Vec2::splat(0.5),
                    mouse_wheel_zoom_sensitivity: 0.06,
                    smoothing_weight: 0.,
                    ..default()
                },
                Vec3::new(-2.0, 5.0, 5.0),
                Vec3::new(0., 0., 0.),
                Vec3::Y,
            ),
            CadCamera, // Mark the camera to be used for CAD.
        ));
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Spawn the simple cube...
    spawn_simple_cube.write(GenerateCadModel::default());
}

fn render_origin_gizmo(mut gizmos: Gizmos) {
    gizmos.arrow(Vec3::ZERO, Vec3::X, css::RED);
    gizmos.arrow(Vec3::ZERO, Vec3::Y, css::GREEN);
    gizmos.arrow(Vec3::ZERO, Vec3::Z, css::BLUE);
}

/// Create struct for the simple parametric cube.
#[derive(Debug, Reflect, Component, Clone)]
struct SimpleCube {
    /// Length of the side of the cube.
    side_length: f64,
    /// Number of cubes to spawn.
    array_count: u32,
}

impl Default for SimpleCube {
    fn default() -> Self {
        Self {
            side_length: 0.5,
            array_count: 1,
        }
    }
}

impl PmetraCad for SimpleCube {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        CadShellsBuilders::new(self.clone())?
            .add_shell_builder(CadShellName("SimpleCube".into()), cube_shell_builder)
    }
}

fn cube_shell_builder(params: &SimpleCube) -> Result<CadShell> {
    let SimpleCube { side_length, .. } = &params;
    let mut tagged_elements = CadTaggedElements::default();
    let vertex = Vertex::new(Point3::new(-side_length / 2., 0., side_length / 2.));
    let edge = builder::tsweep(&vertex, Vector3::unit_x() * *side_length);
    let face = builder::tsweep(&edge, -Vector3::unit_z() * *side_length);
    tagged_elements.insert(
        CadElementTag("ProfileFace".into()),
        CadElement::Face(face.clone()),
    );
    let solid = builder::tsweep(&face, Vector3::unit_y() * *side_length);
    let shell = Shell::try_from_solid(&solid)?;
    Ok(CadShell {
        shell,
        tagged_elements,
    })
}

impl PmetraModelling for SimpleCube {
    fn meshes_builders_by_shell(
        &self,
        shells_by_name: &CadShellsByName,
    ) -> Result<CadMeshesBuildersByCadShell<Self>> {
        let mut meshes_builders_by_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?;
        let shell_name = CadShellName("SimpleCube".into());
        for i in 0..self.array_count {
            meshes_builders_by_shell.add_mesh_builder_with_outlines(
                shell_name.clone(),
                "SimpleCube".to_string() + &i.to_string(),
                CadMeshBuilder::new(self.clone(), shell_name.clone())? // builder
                    .set_transform(Transform::from_translation(
                        Vec3::X * (i as f32 * (self.side_length as f32 * 1.5)),
                    ))?
                    .set_base_material(Color::from(css::RED).into())?,
            )?;
        }
        Ok(meshes_builders_by_shell)
    }
}

impl PmetraInteractions for SimpleCube {
    fn sliders(&self, shells_by_name: &CadShellsByName) -> Result<CadSliders> {
        let sliders = CadSliders::default() // sliders
            .add_slider(
                CadSliderName("SideLengthSlider".into()),
                build_side_length_slider(self, shells_by_name)?,
            )?
            .add_slider(
                CadSliderName("ArrayCountSlider".into()),
                build_array_count_slider(self, shells_by_name)?,
            )?;
        Ok(sliders)
    }

    fn on_slider_transform(
        &mut self,
        name: CadSliderName,
        prev_transform: Transform,
        new_transform: Transform,
    ) {
        if name.0 == "SideLengthSlider" {
            let delta = new_transform.translation - prev_transform.translation;
            if delta.length() > 0. {
                self.side_length += delta.z as f64;
            }
        } else if name.0 == "ArrayCountSlider" {
            let delta = new_transform.translation - prev_transform.translation;
            if delta.length() > 0. {
                self.array_count = (new_transform.translation.x / (self.side_length as f32 * 1.5))
                    .floor() as u32
                    + 1;
            }
        }
    }

    fn on_slider_tooltip(&self, name: CadSliderName) -> Result<Option<String>> {
        if name.0 == "SideLengthSlider" {
            Ok(Some(format!("side_length: {:.2}", self.side_length)))
        } else if name.0 == "ArrayCountSlider" {
            Ok(Some(format!("array_count: {}", self.array_count)))
        } else {
            Ok(None)
        }
    }
}

fn build_side_length_slider(
    params: &SimpleCube,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let SimpleCube { side_length, .. } = &params;
    let cad_shell = shells_by_name
        .get(&CadShellName("SimpleCube".to_string()))
        .ok_or_else(|| anyhow!("Could not get cube shell!"))?;
    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let slider_pos = face_centroid.as_vec3() + Vec3::Z * (*side_length as f32 / 2. + 0.1);
    let slider_transform = Transform::from_translation(slider_pos)
        .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));
    Ok(CadSlider {
        drag_plane_normal: face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: Vec3::Z,
            limit_min: Some(Vec3::Z * 0.2),
            limit_max: Some(Vec3::INFINITY),
        },
        ..default()
    })
}

fn build_array_count_slider(
    params: &SimpleCube,
    shells_by_name: &CadShellsByName,
) -> Result<CadSlider> {
    let SimpleCube {
        side_length,
        array_count,
    } = &params;
    let cad_shell = shells_by_name
        .get(&CadShellName("SimpleCube".to_string()))
        .ok_or_else(|| anyhow!("Could not get cube shell!"))?;
    let Some(CadElement::Face(face)) =
        cad_shell.get_element_by_tag(CadElementTag::new("ProfileFace"))
    else {
        return Err(anyhow!("Could not find face!"));
    };
    let face_normal = face.oriented_surface().normal(0.5, 0.5).as_bevy_vec3();
    let face_boundaries = face.boundaries();
    let face_wire = face_boundaries.last().expect("No wire found!");
    let face_centroid = face_wire.get_centroid();
    let slider_pos = face_centroid.as_vec3()
        + Vec3::X * (*array_count as f32 * (*side_length as f32 * 1.5))
        + Vec3::Y * *side_length as f32;
    let slider_transform = Transform::from_translation(slider_pos)
        .with_rotation(get_rotation_from_normals(Vec3::Z, face_normal));
    Ok(CadSlider {
        drag_plane_normal: face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: Vec3::X,
            limit_min: Some(Vec3::X * 0.35),
            limit_max: Some(Vec3::INFINITY),
        },
        ..default()
    })
}

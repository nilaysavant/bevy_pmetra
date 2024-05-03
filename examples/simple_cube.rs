use bevy::{math::DVec3, prelude::*};
use bevy_pmetra::{
    math::get_rotation_from_normals,
    pmetra_core::extensions::shell::ShellCadExtension,
    prelude::*,
    re_exports::{
        anyhow::{anyhow, Result},
        truck_modeling::{builder, EuclideanSpace, ParametricSurface3D, Shell, Vector3, Vertex},
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
        .spawn(Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.001,
                ..Default::default()
            }),
            ..Default::default()
        })
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

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
    // Spawn the simple cube...
    spawn_simple_cube.send(GenerateCadModel::default());
}

fn render_origin_gizmo(mut gizmos: Gizmos) {
    // x
    gizmos.arrow(Vec3::ZERO, Vec3::X, Color::RED);
    // y
    gizmos.arrow(Vec3::ZERO, Vec3::Y, Color::GREEN);
    // z
    gizmos.arrow(Vec3::ZERO, Vec3::Z, Color::BLUE);
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
    let v0 = Vertex::new(
        (DVec3::new(-side_length / 2., 0., side_length / 2.))
            .to_array()
            .into(),
    );
    let v1 = Vertex::new(
        (DVec3::new(side_length / 2., 0., side_length / 2.))
            .to_array()
            .into(),
    );
    tagged_elements.insert(
        CadElementTag("VertexV0".into()),
        CadElement::Vertex(v0.clone()),
    );
    tagged_elements.insert(
        CadElementTag("VertexV1".into()),
        CadElement::Vertex(v1.clone()),
    );
    let edge = builder::tsweep(&v0, v1.point().to_vec() - v0.point().to_vec());
    let face = builder::tsweep(&edge, -Vector3::unit_z() * *side_length);
    tagged_elements.insert(
        CadElementTag("ProfileFace".into()),
        CadElement::Face(face.clone()),
    );
    let solid = builder::tsweep(&face, (DVec3::Y * *side_length).to_array().into());
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
        let mut cad_meshes_lazy_builders_by_cad_shell =
            CadMeshesBuildersByCadShell::new(self.clone(), shells_by_name.clone())?;

        for i in 0..self.array_count {
            cad_meshes_lazy_builders_by_cad_shell.add_mesh_builder(
                CadShellName("SimpleCube".into()),
                "SimpleCube".to_string() + &i.to_string(),
                cube_mesh_builder(
                    self,
                    CadShellName("SimpleCube".into()),
                    Transform::from_translation(
                        Vec3::X * (i as f32 * (self.side_length as f32 * 1.5)),
                    ),
                )?,
            )?;
        }

        Ok(cad_meshes_lazy_builders_by_cad_shell)
    }
}

fn cube_mesh_builder(
    params: &SimpleCube,
    shell_name: CadShellName,
    transform: Transform,
) -> Result<CadMeshBuilder<SimpleCube>> {
    let mesh_builder = CadMeshBuilder::new(params.clone(), shell_name.clone())? // builder
        .set_transform(transform)?
        .set_base_material(Color::RED.into())?;

    Ok(mesh_builder)
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
                self.array_count = (new_transform.translation.x / (self.side_length as f32 * 1.5)).floor() as u32 + 1;
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
        normal: face_normal,
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
        normal: face_normal,
        transform: slider_transform,
        slider_type: CadSliderType::Linear {
            direction: Vec3::X,
            limit_min: Some(Vec3::X * 0.35),
            limit_max: Some(Vec3::INFINITY),
        },
        ..default()
    })
}

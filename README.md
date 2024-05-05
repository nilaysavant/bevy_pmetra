<div align="center">

# Bevy Pmetra

Parametric Modelling for [Bevy Engine][bevy-website] using [Truck][truck-github] CAD kernel.

[Web Demo][pmetra-demo-web]

</div>

> [!WARNING]
> Work in Progress! Feel free to try it out, fork and mod it for your use case. You are welcome to contribute your changes upstream. I will be glad to review and merge them if they fit the vision/scope of the project. Would love any feedback and help!

## Add Plugin

Add the dependency in your project's `Cargo.toml`. Make sure you're using the right version _tag_. Refer to the [Bevy Compatibility](#bevy-compatibility) table.

```toml
[dependencies]
bevy_pmetra = { git = "https://github.com/nilaysavant/bevy_pmetra", tag = "v0.1.0" }
```

## Create Simple Parametric Cube

Refer to [`examples/simple_cube.rs`](https://github.com/nilaysavant/bevy_pmetra/blob/master/examples/simple_cube.rs) for full example.

### Create struct

Create _struct_ for the `SimpleCube` with **fields** being the **parameters**:

```rs
#[derive(Debug, Reflect, Component, Clone)]
struct SimpleCube {
    /// Length of the side of the cube.
    side_length: f64,
    /// Number of cubes to spawn.
    array_count: u32,
}
```

- Make sure to derive `Component`. As it will be added to the _root_/_parent_ `SpatialBundle` entity of the parametric model. Also derive `Clone`.
- Optionally derive `Reflect` and `Debug`.

Implement `Default` for default values of parameters:

```rs
impl Default for SimpleCube {
  fn default() -> Self {
      Self {
          side_length: 0.5,
          array_count: 1,
      }
  }
}
```

### Implement Traits

Implement a few `traits` on our `SimpleCube` _struct_ for _parametric_ behavior:

- [`PmetraCad`](#pmetracad): For generating multiple `CadShell`(s) using this struct via Truck's modelling APIs. `CadShell` is a wrapper around Truck's [`Shell`](https://docs.rs/truck-topology/0.1.1/truck_topology/struct.Shell.html).
- `PmetraModelling`: For parametrically generating `Mesh`(s) from `CadShell`(s).
- `PmetraInteractions`: (Optional) Setup interactions for live manipulations on models using `CadSlider`(s).

#### PmetraCad

```rs
impl PmetraCad for SimpleCube {
    fn shells_builders(&self) -> Result<CadShellsBuilders<Self>> {
        CadShellsBuilders::new(self.clone())?
            .add_shell_builder(CadShellName("SimpleCube".into()), cube_shell_builder)
    }
}
```

- `CadShellsBuilders` lets us add multiple `CadShell` builders per parametric model. Each builder is added as a callback function.
- Since we only have a single kind of geometry/mesh we just need to add one shell builder for our cube: `cube_shell_builder`. This will need to return a `CadShell` for our cube. We give it a name `"SimpleCube"` which we can reference later.
- If we need another kind of geometry/mesh (eg. cylinder, rectangle etc) we can add more such builders which will generate their equivalent shells. NB: We can only use the parameters of our `SimpleCube` _struct_ to generate all the `CadShell`(s).

Here is the code for `cube_shell_builder`:

```rs
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
```

- We model the cube using Truck's APIs. Refer [Truck Cube Modelling Tutorial](https://ricos.gitlab.io/truck-tutorial/v0.1/modeling.html#cube).
- We additionally _tag_ the `"ProfileFace"`. This helps with positioning/orienting things like `CadSliders` with respect to the tagged element, as we will see later.
- We can tag Truck's primitives like `Vertex`, `Edge`, `Wire`, `Face` etc.

#### PmetraModelling

```rs
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
                    .set_base_material(Color::RED.into())?,
            )?;
        }
        Ok(meshes_builders_by_shell)
    }
}
```

- `CadShellsByName` holds the `CadShell`(s) by the given name. In our case we will have a shell for `"SimpleCube"` name.
- `CadMeshesBuildersByCadShell` is used generate multiple `Mesh`(s) for each defined `CadShell` via a `CadMeshBuilder`.
- We add a new mesh builder for our cube using `add_mesh_builder_with_outlines()`, which includes adding outlines for the generated meshes. We can use `add_mesh_builder()` for no outlines (**more performance**!).
- To the above we pass the `shell_name`, a name for the mesh we will be generating, along with the builder for the same.
- The `CadMeshBuilder` takes the parameter struct and the `shell_name`. We can set the `Transform` and the `Material` of our mesh here.
- Since we want to _array_ the cubes (using `array_count`), we run this inside a for loop passing down the index (for naming) and also set the **transform** for each cube.

#### PmetraInteractions

Optionally you can implement this `trait` for _interactive_ _sliders_ that can manipulate parameters of the `SimpleCube`:

```rs
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
```

- We configure the sliders using the `CadSliders` builder in `sliders()` using the received `shells_by_name`.
- Each slider can be added using `add_slider()` which accepts the name of the slider (for future reference/ID) and the `CadSlider` struct itself.
- Here we use utility functions to create the `CadSlider` struct like `build_side_length_slider` for `"SideLengthSlider"`.
- `on_slider_transform` is called by the plugin whenever a slider's _transform_ is changed. We receive the `prev_transform` and the `new_transform` using which can change the parameters of our `SimpleCube` struct. The name of the slider is useful to distinguish and apply changes from the correct slider.
- `on_slider_tooltip` is used to (optionally) set the tooltip text for the _active_ slider.

Here is the code for `build_side_length_slider`:

```rs
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
```

- We used the `"ProfileFace"` tag (we added earlier) to calculate the slider's `Transform` and also set the normal of the _drag plane_.
- 2 types of sliders supported: `Linear` and `Planer`. `Linear` also allows setting the drag _limits_ of the slider along the given _direction_.

## Bevy Compatibility

| bevy | bevy_pmetra       |
| ---- | ----------------- |
| 0.13 | `master` `v0.1.0` |

[bevy-website]: https://bevyengine.org/
[truck-github]: https://github.com/ricosjp/truck
[pmetra-demo-web]: pmetra.vercel.app

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
- `PmetraModelling`: For parametrically generating `Mesh`(s) from struct.
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

## Bevy Compatibility

| bevy | bevy_pmetra       |
| ---- | ----------------- |
| 0.13 | `master` `v0.1.0` |

[bevy-website]: https://bevyengine.org/
[truck-github]: https://github.com/ricosjp/truck
[pmetra-demo-web]: pmetra.vercel.app

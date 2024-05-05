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

- `PmetraCad`: For generating multiple `CadShell`(s) using this struct via Truck's modelling APIs. `CadShell` is a wrapper around Truck's [`Shell`](https://docs.rs/truck-topology/0.1.1/truck_topology/struct.Shell.html).
- `PmetraModelling`: For parametrically generating `Mesh`(s) from struct.
- `PmetraInteractions`: (Optional) For interactive _sliders_.

## Bevy Compatibility

| bevy | bevy_pmetra       |
| ---- | ----------------- |
| 0.13 | `master` `v0.1.0` |

[bevy-website]: https://bevyengine.org/
[truck-github]: https://github.com/ricosjp/truck
[pmetra-demo-web]: pmetra.vercel.app

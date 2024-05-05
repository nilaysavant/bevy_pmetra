<div align="center">

# Bevy Pmetra

Parametric Modelling for [Bevy Engine][bevy-website] using [Truck][truck-github] CAD kernel.

</div>

> [!WARNING]
> Work in Progress! Feel free to try it out, fork and mod it for your use case. You are welcome to contribute your changes upstream. I will be glad to review and merge them if they fit the vision/scope of the project. Would love any feedback and help!

## Add Plugin

Add the dependency in your project's `Cargo.toml`. Make sure you're using the right version _tag_. Refer to the [Bevy Compatibility](#bevy-compatibility) table.

```toml
[dependencies]
bevy_pmetra = { git = "https://github.com/nilaysavant/bevy_pmetra", tag = "v0.1.0" }
```

## Bevy Compatibility

| bevy | bevy_pmetra       |
| ---- | ----------------- |
| 0.13 | `master` `v0.1.0` |

[bevy-website]: https://bevyengine.org/
[truck-github]: https://github.com/ricosjp/truck

# Default recipe.
default: dev-demo

# Normal dev run. The most used command.
dev-demo:
  cargo run --package=pmetra_demo --features=dev

# Run dev with tracy (`bevy/trace_tracy`). This is useful for profiling.
dev-demo-tracy:
  cargo run --package=pmetra_demo --features=dev,bevy/trace_tracy

# Build the release version of the pmetra demo.
build-release-demo:
  cargo build --package=pmetra_demo --release

# Build all.
build:
  cargo build

# Build and serve the pmetra demo web release.
build-serve-pmetra-demo: build-pmetra-demo-web serve-pmetra-demo-web-release

# Build the web release version of the pmetra demo.
build-pmetra-demo-web:
  #!/bin/bash

  echo "trunk build in release mode..."
  RUSTFLAGS="--cfg=web_sys_unstable_apis" trunk build --release --no-default-features
  echo "trunk build in release mode... done!"

trunk-serve-web:
  #!/bin/bash

  trunk serve --release --no-default-features

# Serve demo web release build.
serve-pmetra-demo-web-release:
  #!/bin/bash

  # Serve dist
  serve -s dist 

# Run all tests.
test:
  cargo test

# Build for Vercel deployment.
vercel-build: build-pmetra-demo-web
  vercel build

# Build and deploy demo WASM via Vercel.
vercel-deploy: vercel-build
  vercel deploy --prebuilt

# List all available recipes.
list:
  just --list

# This is a comment.
example-recipe:
  @echo 'This is example recipe.'

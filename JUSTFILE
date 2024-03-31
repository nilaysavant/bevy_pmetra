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
  trunk build --release --no-default-features
  echo "trunk build in release mode... done!"

  echo "cd into dist..."
  cd dist

  echo "fix paths (from absolute to relative) in index.html..."
  sed -i -e 's/href="/href="./g' index.html
  sed -i -e "s/'\//'.\//g" index.html
  echo "fix paths (from absolute to relative) in index.html... done!"

# Serve demo web release build.
serve-pmetra-demo-web-release:
  #!/bin/bash

  # Serve dist
  serve -s dist 

# Run all tests.
test:
  cargo test

# List all available recipes.
list:
  just --list

# This is a comment.
example-recipe:
  @echo 'This is example recipe.'

# Default recipe.
default: dev-demo

# Normal dev run. The most used command.
dev-demo:
  cargo run --package=pmetra_demo --features=dev

# Run dev with tracy (`bevy/trace_tracy`). This is useful for profiling.
dev-demo-tracy:
  cargo run --package=pmetra_demo --features=dev,bevy/trace_tracy

# Build all.
build:
  cargo build

# Run all tests.
test:
  cargo test

# List all available recipes.
list:
  just --list

# This is a comment.
example-recipe:
  @echo 'This is example recipe.'

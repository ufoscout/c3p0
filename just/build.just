
# Builds all artifacts
[group('build')]
build:
  cargo build
  cargo build --all-features

# Publishes all artifacts on crates.io
[group('build')]
publish:
  cargo publish

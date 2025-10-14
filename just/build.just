
# Builds all artifacts
[group('build')]
build:
  cargo build
  cargo check --features mysql
  cargo check --features postgres
  cargo check --features sqlite
  cargo check --all-features
  cargo check --examples --features postgres

# Publishes all artifacts on crates.io
[group('build')]
publish:
  cargo publish

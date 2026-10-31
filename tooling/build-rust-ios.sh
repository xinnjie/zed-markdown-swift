#!/usr/bin/env bash
# Builds the Rust static library for the iOS simulator and exposes it to Package.swift at
# target/ios/<configuration>. Extra arguments are passed to cargo (e.g. --release).
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

rust_target=aarch64-apple-ios-sim
profile=debug
for argument in "$@"; do
  if [[ "$argument" == "--release" ]]; then profile=release; fi
done

# The Metal Toolchain is optional: shaders compile at runtime instead of at build time.
IPHONEOS_DEPLOYMENT_TARGET=17.0 cargo build --target "$rust_target" -p zed-gpui-c-api \
  --features runtime_shaders "$@"

mkdir -p "target/ios"
ln -sfn "$root/target/$rust_target/$profile" "target/ios/$profile"

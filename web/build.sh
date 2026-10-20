#!/bin/sh
# Builds the gallery for the browser into web/dist: the page, gallery.js and gallery_bg.wasm.
# Needs the wasm32-unknown-unknown target and a wasm-bindgen CLI of the version in Cargo.lock.
set -eu
cd "$(dirname "$0")/.."
# Smaller over faster: the page downloads all of it before it draws anything.
CARGO_PROFILE_RELEASE_OPT_LEVEL=z CARGO_PROFILE_RELEASE_LTO=true CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 \
    cargo build --release --example gallery --target wasm32-unknown-unknown
rm -rf web/dist
wasm-bindgen --target web --no-typescript --out-dir web/dist --out-name gallery \
    target/wasm32-unknown-unknown/release/examples/gallery.wasm
if command -v wasm-opt >/dev/null; then
    wasm-opt -Oz --all-features -o web/dist/gallery_bg.wasm web/dist/gallery_bg.wasm
fi
cp web/index.html web/dist/

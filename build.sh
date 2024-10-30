#!/usr/bin/env bash

canister=$1
canister_root="src/$canister"
cargo build --manifest-path="$canister_root/Cargo.toml" \
  --target wasm32-unknown-unknown \
  --release --package "$canister"
candid-extractor "target/wasm32-unknown-unknown/release/$canister.wasm" > "$canister_root/$canister.did"



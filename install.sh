#!/usr/bin/env bash

main() {
    cargo build --release --target wasm32-wasip1 &&
        install -Dm644 ${CARGO_TARGET_DIR}/wasm32-wasip1/release/reltab.wasm ~/.config/zellij/plugins/reltab.wasm
}

main "${@}"

# Documentation targets
mod doc

default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

alias f := fmt

# CI=true for https://github.com/tauri-apps/tauri/issues/3055#issuecomment-1624389208)
bundle $CI="true":
    # HACK (change PWD): Until https://github.com/DioxusLabs/dioxus/issues/1283
    cd ./crates/omnix-gui/assets && dx bundle --release
    nix run nixpkgs#lsd -- --tree ./dist/bundle/macos/omnix-gui.app

# Run omnix-gui locally
watch-gui $RUST_BACKTRACE="1":
    # XXX: hot reload doesn't work with tailwind
    # dx serve --hot-reload
    cd ./crates/omnix-gui && dx serve --bin omnix-gui

alias wg := watch-gui

# Run omnix-cli locally
watch *ARGS:
    bacon --job run -- -- {{ ARGS }}

alias w := watch

# Run CI locally
ci:
    nix run . ci

# Run CI locally in devShell (using cargo)
ci-cargo:
    cargo run -p omnix-cli -- ci run

clippy:
    cargo clippy --release --locked --all-targets --all-features -- --deny warnings

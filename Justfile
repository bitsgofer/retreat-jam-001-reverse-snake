[private]
default: help-list

# List all recipes
[private]
help-list:
    just --list --unsorted

# ##############################################################################
# CONVENTION ###################################################################
# ##############################################################################

set lazy
# ------------------------------------------------------------------------------

repoRoot := invocation_directory() # OR: env("PWD", shell("pwd"))

# ##############################################################################
# dev ##########################################################################
# ##############################################################################

# Enter Nix development environment
[group("dev")]
[private]
dev:
    nix develop

# Build: game
[group("dev")]
build:
    cargo build


# Run: dev build (uses dynamic linking + asset hot-reload)
[group("dev")]
run:
    cargo run -p reverse-snake --features dev

# ##############################################################################
# test #########################################################################
# ##############################################################################

# ##############################################################################
# fmt ##########################################################################
# ##############################################################################

# Format: *.rs
[group("fmt")]
fmt:
    cargo fmt

# Format: everything
[group("fmt")]
fmt-all: fmt fmt-just fmt-nix

# Format: Justfile
[group("fmt")]
[private]
fmt-just:
    just --fmt

# Format: *.nix
[group("fmt")]
[private]
fmt-nix:
    find . -type f -name '*.nix' -o -name '*.nix' | xargs nixfmt

# ##############################################################################
# lint #########################################################################
# ##############################################################################

# Lint: *.rs
[group('lint')]
lint:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

# Lint: everything
[group('lint')]
lint-all: lint lint-just lint-nix

[group('lint')]
[private]
lint-just:
    just --unstable --fmt --check

[group('lint')]
[private]
lint-nix:
    find . -type f -name '*.nix' -o -name '*.nix' | xargs nixfmt --check

# ##############################################################################
# release ######################################################################
# ##############################################################################

# Build: release binary (optimized, no dev features)
[group("release")]
build-release:
    cargo build --release -p reverse-snake

# Run: release build
[group("release")]
release-run: build-release
    ./target/release/reverse-snake

# Package: Linux build ready to upload to itch.io
[group("release")]
package-linux: build-release
    rm -rf dist/reverse-snake-linux
    mkdir -p dist/reverse-snake-linux
    cp target/release/reverse-snake dist/reverse-snake-linux/
    # `cargo build` on Nix links against $NIX_STORE paths for both the ELF
    # interpreter and its libs (wayland/alsa/udev/libgcc_s/libc). Neither
    # exists on a player's machine, so the binary won't even start without
    # this: point it at the standard system loader and drop the baked-in
    # Nix rpath, so it falls back to the target system's own copies of
    # those (near-universal) libs instead. Vulkan/Mesa are already dlopen'd
    # at runtime, not linked, so they're unaffected by this.
    patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 --remove-rpath dist/reverse-snake-linux/reverse-snake
    cp -r assets dist/reverse-snake-linux/
    rm -f dist/reverse-snake-linux/assets/*.aseprite
    cd dist/reverse-snake-linux && zip -rq ../reverse-snake-linux.zip .
    @echo "Upload: dist/reverse-snake-linux.zip"

# Build: wasm32 release + JS bindings for the browser
[group("release")]
[private]
build-web:
    cargo build --release --target wasm32-unknown-unknown -p reverse-snake
    rm -rf dist/reverse-snake-web
    mkdir -p dist/reverse-snake-web
    wasm-bindgen --no-typescript --target web --out-dir dist/reverse-snake-web --out-name game target/wasm32-unknown-unknown/release/reverse-snake.wasm
    # wasm-bindgen's raw output is ~60MB; wasm-opt -Oz cuts it by ~40% for
    # itch.io's embedded player. --all-features matches whatever feature
    # flags (bulk-memory, etc.) rustc enabled for wasm32-unknown-unknown,
    # since wasm-opt otherwise rejects the module as invalid.
    wasm-opt --all-features -Oz -o dist/reverse-snake-web/game_bg.wasm dist/reverse-snake-web/game_bg.wasm

# Package: browser build ready to upload to itch.io
[group("release")]
package-web: build-web
    cp web/index.html dist/reverse-snake-web/
    cp -r assets dist/reverse-snake-web/
    rm -f dist/reverse-snake-web/assets/*.aseprite
    cd dist/reverse-snake-web && zip -rq ../reverse-snake-web.zip .
    @echo "Upload: dist/reverse-snake-web.zip (itch.io: check 'This file will be played in the browser', entry point index.html)"

# Package: Linux + browser builds, ready to upload to itch.io
[group("release")]
package: package-linux package-web

# ##############################################################################
# utils ########################################################################
# ##############################################################################

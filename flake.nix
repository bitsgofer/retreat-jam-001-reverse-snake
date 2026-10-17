{
  description = "BitsGofer Studio Game Jam 001 - Reverse Snake";
  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixos-unstable";
    };
  };
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.just
            pkgs.git-lfs
            pkgs.shfmt
            pkgs.shellcheck
            pkgs.rustc
			pkgs.rust-analyzer
			pkgs.clippy
            pkgs.cargo
            pkgs.nixfmt
            # Release packaging (itch.io): linking the wasm32-unknown-unknown
            # target (lld), generating its JS glue (wasm-bindgen-cli, pinned
            # in game/Cargo.toml to match this exact version), and zipping
            # up the Linux/web builds.
            pkgs.lld
            pkgs.wasm-bindgen-cli
            pkgs.binaryen # wasm-opt: shrinks the unprocessed ~60MB wasm32 build
            pkgs.patchelf # de-nixifies the Linux release binary, see `just package-linux`
            pkgs.zip
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            pkgs.pkg-config
            pkgs.wayland
            pkgs.alsa-lib
            pkgs.alsa-plugins
            pkgs.pipewire
            pkgs.udev
            pkgs.libxkbcommon
            pkgs.vulkan-loader
            pkgs.mesa
            pkgs.libdrm
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [
          ];

          shellHook = ''
            export JUST_EXPLAIN=true

            repotRoot="$(pwd)"

            # Assets path are relative to BEVY_ASSET_ROOT
            export BEVY_ASSET_ROOT="$repotRoot"

          ''
          + pkgs.lib.optionalString pkgs.stdenv.hostPlatform.isLinux ''
            # Bevy/winit dlopen() these at runtime (e.g. libxkbcommon for
            # Wayland keyboard input) instead of linking them at build time,
            # so having them in `packages` above isn't enough on non-NixOS
            # hosts without a system-wide copy. Point the dynamic linker at
            # the Nix store paths explicitly.
            export LD_LIBRARY_PATH="${
              pkgs.lib.makeLibraryPath [
                pkgs.wayland
                pkgs.alsa-lib
                pkgs.alsa-plugins
                pkgs.pipewire
                pkgs.udev
                pkgs.libxkbcommon
                pkgs.vulkan-loader
                pkgs.mesa
                pkgs.libdrm
              ]
            }:$LD_LIBRARY_PATH"
          ''
          + ''
            just
            echo ""
            echo "Run: just --usage [recipe] => individual recipe's usage."
            echo "You are now using the development environment!"
            echo ""
          '';
        };
      });
    };
}

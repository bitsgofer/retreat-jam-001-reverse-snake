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
      supportedOSAndArch = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems =
        f: nixpkgs.lib.genAttrs supportedOSAndArch (system: f nixpkgs.legacyPackages.${system});
    in
    {
      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            # Rust
            # TODO: consider using https://github.com/oxalica/rust-overlay
            pkgs.rustc
            pkgs.rust-analyzer
            pkgs.clippy
            pkgs.cargo
          ]
          ++ [
            # Utilities
            pkgs.just
            pkgs.nixfmt
            pkgs.git-lfs
            pkgs.shfmt
            pkgs.shellcheck
          ]
          ++ [
            # Release
            pkgs.lld
            pkgs.wasm-bindgen-cli
            pkgs.binaryen
            pkgs.patchelf
            pkgs.zip
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [
            # On Linux: vendor graphics, sound, input libraries
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
            # On MacOS:
          ];

          shellHook = ''
            # Repo's absolute path
            repotRoot="$(pwd)"

            # Just always print the docs to explain a recipe.
            export JUST_EXPLAIN=true

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
            }"
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

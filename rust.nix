# Nix module for the Rust part of the project
#
# This uses https://github.com/srid/dioxus-desktop-template/blob/master/nix/flake-module.nix
{
  perSystem = { config, self', pkgs, lib, system, ... }: {
    dioxus-desktop = {
      overrideCraneArgs = oa: {
        nativeBuildInputs = (oa.nativeBuildInputs or [ ]) ++ [
          pkgs.nix # cargo tests need nix
        ];
        meta.description = "WIP: nix-browser";
      };
      rustBuildInputs = lib.optionals pkgs.stdenv.isLinux
        (with pkgs; [
          webkitgtk_4_1
          pkg-config
        ]) ++ lib.optionals pkgs.stdenv.isDarwin (
        with pkgs.darwin.apple_sdk.frameworks; [
          IOKit
          Carbon
          WebKit
          Security
          Cocoa
          # Use newer SDK because some crates require it
          # cf. https://github.com/NixOS/nixpkgs/pull/261683#issuecomment-1772935802
          pkgs.darwin.apple_sdk_11_0.frameworks.CoreFoundation
        ]
      );
    };

    packages.default = self'.packages.nix-browser;

    devShells.rust = pkgs.mkShell {
      inputsFrom = [
        self'.devShells.nix-browser
      ];
      packages = with pkgs; [
        cargo-watch
        cargo-expand
        cargo-nextest
        config.process-compose.cargo-doc-live.outputs.package
      ];
      shellHook = ''
        echo
        echo "🍎🍎 Run 'just <recipe>' to get started"
        just
      '';
    };
  };
}

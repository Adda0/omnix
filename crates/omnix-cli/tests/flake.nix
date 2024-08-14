# A dummy flake to cache test dependencies in Nix store.
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    # NOTE: These inputs should kept in sync with those used in the Rust source (cli.rs)
    haskell-multi-nix.url = "github:srid/haskell-multi-nix/c85563721c388629fa9e538a1d97274861bc8321";
    services-flake.url = "github:juspay/services-flake/3d764f19d0a121915447641fe49a9b8d02777ff8";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = import inputs.systems;
      perSystem = { pkgs, system, ... }: {
        packages = {
          haskell-multi-nix = inputs.haskell-multi-nix.packages.${system}.default;
        };
      };
    };
}
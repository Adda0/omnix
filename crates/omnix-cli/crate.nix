{ flake
, rust-project
, pkgs
, lib
, ...
}:

let
  inherit (flake) inputs;
  inherit (pkgs) stdenv pkgsStatic;
in
{
  autoWire = false;
  crane = {
    args = {
      nativeBuildInputs = with pkgs.apple_sdk_frameworks; lib.optionals stdenv.isDarwin [
        Security
        SystemConfiguration
      ] ++ [
        # Packages from `pkgsStatic` require cross-compilation support for the target platform,
        # which is not yet available for `x86_64-apple-darwin` in nixpkgs. Upon trying to evaluate
        # a static package for `x86_64-apple-darwin`, you may see an error like:
        #
        # > error: don't yet have a `targetPackages.darwin.LibsystemCross for x86_64-apple-darwin`
        (if (stdenv.isDarwin && stdenv.isAarch64) then pkgsStatic.libiconv else pkgs.libiconv)
        pkgs.pkg-config
      ];
      buildInputs = lib.optionals pkgs.stdenv.isDarwin
        (
          with pkgs.apple_sdk_frameworks; [
            IOKit
            CoreFoundation
          ]
        ) ++ lib.optionals pkgs.stdenv.isLinux [
        pkgsStatic.openssl
      ];

      inherit (rust-project.crates."nix_rs".crane.args)
        DEVOUR_FLAKE
        DEFAULT_FLAKE_SCHEMAS
        NIX_FLAKE_SCHEMAS_BIN
        ;
      inherit (rust-project.crates."nixci".crane.args)
        OMNIX_SOURCE
        ;
      inherit (rust-project.crates."flakreate".crane.args)
        OM_INIT_REGISTRY
        ;

      # Disable tests due to sandboxing issues; we run them on CI
      # instead.
      doCheck = false;
      meta = {
        description = "Command-line interface for Omnix";
        mainProgram = "om";
      };
      CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";
    } //
    lib.optionalAttrs pkgs.stdenv.isLinux {
      CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    };
  };
}

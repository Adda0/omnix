{
  # omnix configuration
  flake = {
    om = {
      ci.default = {
        omnix = {
          dir = ".";
          steps = {
            # build.enable = false;
            flake-check.enable = false; # Not necessary
            custom = {
              om-show = {
                type = "app";
                # name = "default";
                args = [ "show" "." ];
              };
              binary-size-is-small = {
                type = "app";
                name = "check-closure-size";
                systems = [ "x86_64-linux" ]; # We have static binary for Linux only.
              };
              omnix-source-is-buildable = {
                type = "app";
                name = "omnix-source-is-buildable";
              };
              cargo-tests = {
                type = "devshell";
                command = [ "just" "cargo-test" ];
                systems = [ "x86_64-linux" "aarch64-darwin" ]; # Avoid emulated systems
              };
              cargo-clippy = {
                type = "devshell";
                command = [ "just" "clippy" ];
                systems = [ "x86_64-linux" "aarch64-darwin" ]; # Avoid emulated systems
              };
              cargo-doc = {
                type = "devshell";
                command = [ "just" "cargo-doc" ];
                systems = [ "x86_64-linux" "aarch64-darwin" ]; # Avoid emulated systems
              };
            };
          };
        };
        doc.dir = "doc";

        registry = {
          dir = "crates/omnix-init/registry";
          steps = {
            build.enable = false;

            # FIXME: Why does omnix require this?
            custom = { };
          };
        };

        # Because the cargo tests invoking Nix doesn't pass github access tokens..
        # To avoid GitHub rate limits during the integration test (which
        # doesn't use the token)
        cli-test-dep-cache = {
          dir = "crates/omnix-cli/tests";
          steps = {
            lockfile.enable = false;
            flake_check.enable = false;
            # FIXME: Why does omnix require this?
            custom = { };
          };
        };
      };
      health.default = {
        nix-version.min-required = "2.16.0";
        caches.required = [ "https://om.cachix.org" ];
        direnv.required = true;
      };
      develop.default = {
        readme = ''
          🍾 Welcome to the **omnix** project

          To run omnix,

          ```sh-session
          just watch <args>
          ```

          (Now, as you edit the Rust sources, the above will reload!)

          🍎🍎 Run 'just' to see more commands. See <https://nixos.asia/en/vscode> for IDE setup.
        '';
      };
    };
  };
}

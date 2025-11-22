{
  # This is our input set, it contains the channels we will construct the flake from.
  inputs = {
    # Flake-utils allows us to easily support multiple architectures.
    flake-utils.url = "github:numtide/flake-utils";
    # Naersk is a zero-configuration zero-codegen solution to packaging Rust.
    naersk.url = "github:nix-community/naersk";
    # Nixpkgs is the main package repo for Nix, we will use it to bring in all of our
    # libraries and tools.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  # This is the output set, Nix and other Flakes will be able to consume the attributes
  # in the output set.
  outputs = { self, flake-utils, naersk, nixpkgs }:
    # We wrap the entire output set in this flake-utils function, which builds the flake
    # for each architecture type supported by nix.
    flake-utils.lib.eachDefaultSystem (system:
      let
        # This sets up nixpkgs, where we will pull our dependencies from
        pkgs = (import nixpkgs) {
          # You can insert overlays here by calling `inherit system overlays;` 
          inherit system;
        };

        # This sets up naersk, which we will use later.
        naersk' = pkgs.callPackage naersk { };

        # Here we can add non-rust dependencies that our program requires *at run time*.
        buildInputs = with pkgs; [

        ];

        # here we can add non-rust dependencies that our program requires *at build time*.
        nativeBuildInputs = with pkgs; [

        ];
      in
      rec {
        # Build this with `nix build`, run it with `nix run`
        defaultPackage = packages.app;
        packages =
          {
            app = naersk'.buildPackage {
              # Naersk will look for a `Cargo.toml` in this directory
              src = ./.;
              # Our buildinputs from above are specified here
              nativeBuildInputs = nativeBuildInputs;
              buildInputs = buildInputs;
            };
            contianer = pkgs.dockerTools.buildImage
              {
                name = "app";
                config = {
                  entrypoint = [ "${packages.app}/bin/app" ];
                };
              };
          };

        # This will be entered by direnv, or by manually running `nix shell`. This ensures
        # that our development environment will have all the correct tools at the correct
        # version for this project.
        devShell = pkgs.mkShell {
          # Here we add any tools that we want in our dev-shell but aren't required to build
          # our application.
          nativeBuildInputs = with pkgs;
            [
              nixpkgs-fmt
              cmake
              rustc
              cargo
              clippy
            ] ++ buildInputs ++ nativeBuildInputs;
          # The above line merges our buildInputs into the devshell, so we have them when
          # using cargo tools from inside our devshell.
        };
      }
    );
}


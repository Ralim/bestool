{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };
  outputs =
    { self
    , nixpkgs
    , flake-utils
    , ...
    }:
    flake-utils.lib.eachDefaultSystem
      (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
      in
      with pkgs; {
        packages.bestool = callPackage ./.bestool.nix { };
        packages.default = self.packages.${system}.bestool;

        devShells.default = mkShell {
          inputsFrom = lib.singleton self.packages.${system}.default;
          buildInputs = [
            cargo-edit
            clippy
            rustfmt
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
        };
        })
    // {
      overlays.default = final: prev: {
        inherit (self.packages.${final.system}) bestool;
      };
    };
}

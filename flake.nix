
{
  description = "LaDFX";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShell = pkgs.mkShell {
          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.wasm-pack
            pkgs.nodejs
          ];
        };

        packages.default = pkgs.stdenv.mkDerivation {
          name = "build-dist";
          src = ./.;

          buildInputs = [
            pkgs.rustc
            pkgs.cargo
            pkgs.wasm-pack
          ];

          buildPhase = ''
            wasm-pack build --target web
            mkdir -p dist
            cp index.html dist/
            cp logotex.png dist/
            cp -r pkg dist/
          '';

          installPhase = ''
            mkdir -p $out
            cp -r dist/* $out/
          '';
        };
      });
}

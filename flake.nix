{
  description = "Oneil -  Design specification language for rapid, comprehensive system modeling.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          config.allowUnfree = true;
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust tools
            rustc
            cargo
            clippy
            rustfmt
            rust-analyzer

            # VSCode extension tools
            nodejs_20
            pnpm
            vsce # "Visual Studio Code Extension Manager"
          ];
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "oneil";
          version = "0.15.0";
          src = ./.;
          cargoLock = {
            lockFile = ./Cargo.lock;
          };
        };
      });
}


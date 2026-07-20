{
  description = "Oneil - Design specification language for rapid, comprehensive system modeling.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ "x86_64-linux" "aarch64-linux" ];

    perSystem = { pkgs, system, ... }: {
      devShells.default = pkgs.mkShell {
        buildInputs = [
          # Rust tools
          pkgs.rustc
          pkgs.cargo
          pkgs.clippy
          pkgs.rustfmt
          pkgs.rust-analyzer
          pkgs.python3 # used by PyO3

          # VSCode extension tools
          pkgs.nodejs
          pkgs.pnpm
          pkgs.vsce # "Visual Studio Code Extension Manager"
        ];
      };

      packages.default = pkgs.rustPlatform.buildRustPackage {
        pname = "oneil";
        version = "0.16.1";
        src = ./.;
        nativeBuildInputs = [ pkgs.python3 ]; # used by PyO3
        cargoLock.lockFile = ./Cargo.lock;
      };
    };
  };
}

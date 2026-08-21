{
  description = "Oneil - Design specification language for rapid, comprehensive system modeling.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [
      "x86_64-linux"
      "aarch64-linux"
      "x86_64-darwin"
      "aarch64-darwin"
    ];

    flake.overlays.default = final: _prev: {
      oneil = inputs.self.packages.${final.stdenv.hostPlatform.system}.oneil;
    };

    perSystem = { pkgs, self', ... }:
      let
        python = pkgs.python312;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            # Rust tools
            pkgs.rustc
            pkgs.cargo
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
            python # used by PyO3

            # VSCode extension tools
            pkgs.nodejs
            pkgs.pnpm
            pkgs.vsce # "Visual Studio Code Extension Manager"
          ];
          env = {
            PYO3_PYTHON = python.interpreter;
            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };
        };

        packages.oneil = pkgs.rustPlatform.buildRustPackage {
          pname = "oneil";
          version = "1.0.0";
          src = ./.;
          nativeBuildInputs = [ python ];
          buildInputs = [ python ];
          cargoLock.lockFile = ./Cargo.lock;
          env.PYO3_PYTHON = python.interpreter;

          meta = {
            description = "Design specification language for rapid, comprehensive system modeling";
            longDescription = ''
              Oneil is a design specification language for modeling systems as
              collections of parameters. Models can evaluate corresponding
              designs (value assignments), with built-in unit handling, tests,
              and a command-line interface for tracing calculations.
            '';
            homepage = "https://github.com/careweather/oneil";
            changelog = "https://github.com/careweather/oneil/releases";
            license = pkgs.lib.licenses.mpl20;
            platforms = pkgs.lib.platforms.unix;
            mainProgram = "oneil";
          };
        };
        packages.default = self'.packages.oneil;
      };
  };
}

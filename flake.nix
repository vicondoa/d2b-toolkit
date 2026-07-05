{
  description = "Shared Rust/Nix toolkit crates for d2b Wayland desktop integrations";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAllSystems (pkgs: {
        default = pkgs.stdenvNoCC.mkDerivation {
          pname = "d2b-toolkit-source";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          installPhase = ''
            runHook preInstall
            mkdir -p $out/share/d2b-toolkit
            cp -R Cargo.toml Cargo.lock crates README.md docs $out/share/d2b-toolkit/
            runHook postInstall
          '';
        };
      });

      checks = forAllSystems (pkgs: {
        rust-workspace = pkgs.rustPlatform.buildRustPackage {
          pname = "d2b-toolkit-workspace-check";
          version = "0.1.0";
          src = pkgs.lib.cleanSource ./.;
          cargoLock.lockFile = ./Cargo.lock;
          cargoBuildFlags = [ "--workspace" ];
          cargoTestFlags = [ "--workspace" ];
        };
        cargo-fmt = pkgs.runCommand "d2b-toolkit-cargo-fmt" {
          nativeBuildInputs = [ pkgs.cargo pkgs.rustc pkgs.rustfmt ];
        } ''
          cp -R ${pkgs.lib.cleanSource ./.} source
          chmod -R u+w source
          cd source
          cargo fmt --all -- --check
          touch $out
        '';
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [ pkgs.cargo pkgs.rustc pkgs.rustfmt pkgs.clippy pkgs.nixpkgs-fmt ];
        };
      });
    };
}

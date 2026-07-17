{
  description = "Canonical d2b client toolkit source distribution";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    d2b-src = {
      url = "github:vicondoa/d2b/4018d9c9652bd826c2e6a9abccdcdcafb832d944";
      flake = false;
    };
  };

  outputs = { self, nixpkgs, d2b-src }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = function:
        nixpkgs.lib.genAttrs systems
          (system: function nixpkgs.legacyPackages.${system});
      version = "2.0.0";
    in
    {
      packages = forAllSystems (pkgs:
        let
          distributionSource = pkgs.lib.cleanSource self;
          sourcePackage = pkgs.stdenvNoCC.mkDerivation {
            pname = "d2b-client-toolkit-source";
            inherit version;
            src = distributionSource;
            nativeBuildInputs = [ pkgs.python3 ];

            dontBuild = true;
            doCheck = true;

            checkPhase = ''
              runHook preCheck
              D2B_CANONICAL_SOURCE=${d2b-src} \
                python3 scripts/check-source-fingerprint.py
              runHook postCheck
            '';

            installPhase = ''
              runHook preInstall
              root=$out/share/d2b-client-toolkit
              mkdir -p "$root/distribution" "$root/d2b"
              cp -R ./. "$root/distribution/"
              cp -R ${d2b-src}/. "$root/d2b/"
              runHook postInstall
            '';

            passthru = {
              canonicalRevision = "4018d9c9652bd826c2e6a9abccdcdcafb832d944";
              sourceFingerprint = "c2c99bdd77ba66948fce81161dcc3efde608eefefb96f28fa934c9f58d96d838";
            };
          };
        in
        {
          d2b-client-toolkit = sourcePackage;
          default = sourcePackage;
        });

      checks = forAllSystems (pkgs: {
        rust-workspace = pkgs.rustPlatform.buildRustPackage {
          pname = "d2b-client-toolkit-workspace-check";
          inherit version;
          src = pkgs.lib.cleanSource self;
          cargoHash = "sha256-sKtH2ABCpeC1jIAW9yMyXSLEwqSoUW7Lqgp5LIxSWh8=";
          cargoBuildFlags = [ "--workspace" "--all-features" ];
          cargoTestFlags = [ "--workspace" "--all-features" ];
          nativeBuildInputs = [ pkgs.python3 ];
          D2B_CANONICAL_SOURCE = d2b-src;
          preCheck = ''
            python3 scripts/check-source-fingerprint.py
          '';
        };

        cargo-fmt = pkgs.runCommand "d2b-client-toolkit-cargo-fmt"
          {
            nativeBuildInputs = [ pkgs.cargo pkgs.rustc pkgs.rustfmt ];
          } ''
          cp -R ${pkgs.lib.cleanSource self} source
          chmod -R u+w source
          cd source
          cargo fmt --all -- --check
          touch $out
        '';

        source-fingerprint = pkgs.runCommand "d2b-client-toolkit-source-fingerprint"
          {
            nativeBuildInputs = [ pkgs.python3 ];
          } ''
          cp -R ${pkgs.lib.cleanSource self} source
          chmod -R u+w source
          cd source
          D2B_CANONICAL_SOURCE=${d2b-src} \
            python3 scripts/check-source-fingerprint.py
          touch $out
        '';
      });

      devShells = forAllSystems (pkgs: {
        default = pkgs.mkShell {
          packages = [
            pkgs.cargo
            pkgs.clippy
            pkgs.python3
            pkgs.rustc
            pkgs.rustfmt
            pkgs.nixpkgs-fmt
          ];
        };
      });
    };
}

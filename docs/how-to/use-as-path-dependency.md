# Use the source distribution as path dependencies

The flake output carries the facade and its exact canonical d2b source in one
store path:

```nix
let
  source =
    inputs.d2b-client-toolkit.packages.${system}.d2b-client-toolkit;
in
{
  toolkitRoot = "${source}/share/d2b-client-toolkit";
}
```

Use paths below `distribution/crates/` for toolkit-owned crates and paths below
`d2b/packages/` for canonical crates:

```toml
[dependencies]
d2b-client-toolkit = { path = "/nix/store/…/share/d2b-client-toolkit/distribution/crates/d2b-client-toolkit" }
d2b-client = { path = "/nix/store/…/share/d2b-client-toolkit/d2b/packages/d2b-client", default-features = false }
d2b-contracts = { path = "/nix/store/…/share/d2b-client-toolkit/d2b/packages/d2b-contracts", default-features = false, features = ["v2-services"] }
d2b-session = { path = "/nix/store/…/share/d2b-client-toolkit/d2b/packages/d2b-session", default-features = false }
```

Build tooling should generate those concrete paths from the flake output rather
than committing Nix store paths. When patching the facade's exact Git
dependencies to the bundled paths, patch all four canonical packages to the
same `d2b/` tree.

## Flake input

Keep one `nixpkgs` input:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    d2b-client-toolkit = {
      url = "github:vicondoa/d2b-client-toolkit";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

For local work, replace the GitHub URL with this checkout's `path:` URL and
retain the `follows` declaration.

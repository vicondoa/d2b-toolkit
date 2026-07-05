# Use as a path dependency

During local development, point downstream workspaces at this checkout:

```toml
[dependencies]
d2b-client = { path = "../d2b-toolkit/crates/d2b-client" }
d2b-toolkit-core = { path = "../d2b-toolkit/crates/d2b-toolkit-core" }
```

Run `cargo test --workspace` in this repository before updating downstream pins.

## Flake input boilerplate

When the downstream is a Nix flake, make the toolkit follow the same `nixpkgs`
as d2b and the desktop client:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    d2b = {
      url = "github:vicondoa/d2b";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    d2b-toolkit = {
      url = "github:vicondoa/d2b-toolkit";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    d2b-wlterm = {
      url = "github:vicondoa/d2b-wlterm";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.d2b-toolkit.follows = "d2b-toolkit";
    };
  };
}
```

For local iteration, replace `github:vicondoa/d2b-toolkit` with a `path:` URL
but keep the same `follows` lines. That keeps `nix flake check` evaluating the
client module and toolkit source against one nixpkgs revision.

## Packaged source output

The default package is intentionally a source package:

```nix
toolkitSource = inputs.d2b-toolkit.packages.${system}.default;
```

Clients can point Cargo path dependencies at
`${toolkitSource}/share/d2b-toolkit/crates/<crate>` during Nix builds. This is
the packaging seam used by d2b desktop companions; it avoids hard-coded
developer checkout paths while preserving ordinary Cargo path dependencies for
local worktrees.

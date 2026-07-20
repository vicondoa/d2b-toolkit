# Presentation helpers

## Colors

`d2b-client-toolkit-colors` parses nested `palette` fields and legacy flat
color fields. Values may be CSS hex (`#rgb`, `#rrggbb`, or `#rrggbbaa`) or
`r`/`g`/`b`/`a` components. Invalid JSON is an error; missing or invalid roles
use the default palette and produce a typed fallback record.

CSS output uses the stable presentation names `d2b-host`, `d2b-env`, `d2b-vm`,
`d2b-active`, and `d2b-warning`.

## Waybar

`d2b-client-toolkit-waybar` serializes the standard Waybar custom-module
fields and provides a palette summary. It does not accept shell, workload,
target, session, or Wayland-service wire types.

## Wayland control

This distribution does not own `d2b-wayland-proxy` and does not expose an
authenticated Wayland control helper. The canonical generated service client is
available through `d2b-client-toolkit`; live endpoint acquisition and integrated
proxy behavior remain outside this presentation crate. Consumers must not
substitute direct compositor access or an unauthenticated compatibility path.

# Wayland toolkit helpers

The Wayland crates are split by trust boundary:

- `d2b-wayland-core` provides safe DTOs for UI colors and standard Wayland client metadata. Metadata values are bounded, reject NUL bytes, serialize for user-interface use, and avoid high-cardinality debug labels.
- `d2b-wayland-colors` parses d2b UI color JSON artifacts. Missing or invalid color roles fall back to the default palette and are reported in the returned fallback list. CSS output uses the public `d2b-host`, `d2b-env`, `d2b-vm`, `d2b-active`, and `d2b-warning` names.
- `d2b-wayland-waybar` serializes Waybar custom-module JSON and keeps user-controlled shell names out of labels.
- `d2b-wayland-proxy` owns the Unix ancillary FD transport trait boundary for proxy integrations. Shared client crates must not parse raw proxy wire messages or model SCM_RIGHTS.

Color artifacts may use nested `palette` fields or flat top-level fields. Color values can be CSS hex strings (`#rgb`, `#rrggbb`, `#rrggbbaa`) or `{ "r": 0, "g": 0, "b": 0, "a": 255 }` component objects. Invalid JSON is an error; missing or invalid roles use defaults so status bars and launchers remain usable.

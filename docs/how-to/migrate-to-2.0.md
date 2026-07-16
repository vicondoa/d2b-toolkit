# Migrate from d2b-toolkit 0.2 to d2b-client-toolkit 2.0

Version 2.0 is intentionally incompatible. Do not retain aliases or a
dual-protocol fallback.

1. Rename the repository/flake input to `d2b-client-toolkit`.
2. Replace `d2b-toolkit-core` and the repository-local `d2b-client` with
   `d2b-client-toolkit` plus its canonical `client`, `contracts`, and `session`
   re-exports.
3. Replace `d2b-wayland-colors` with `d2b-client-toolkit-colors`.
4. Replace `d2b-wayland-waybar` with `d2b-client-toolkit-waybar`.
5. Remove use of public JSON framing, hello negotiation, error envelopes,
   workload/shell DTOs, protocol-v3 fixtures, socket classifiers, and generic
   serialized redaction wrappers.
6. Run canonical client futures on an explicit Tokio runtime boundary.
7. Point Nix path dependencies at the single packaged source tree described in
   [`use-as-path-dependency.md`](./use-as-path-dependency.md).

There is no replacement authenticated Wayland helper or live endpoint/route
example in this release. Wait for those canonical service APIs rather than
carrying old behavior forward.

## Repository transition

The GitHub repository rename from `vicondoa/d2b-toolkit` to
`vicondoa/d2b-client-toolkit` is a post-merge integrator action. After the
rename, verify the GitHub redirect, update protected-branch/release settings,
and only then create the 2.0 tag. Do not rename the repository before this
migration lands on its default branch.

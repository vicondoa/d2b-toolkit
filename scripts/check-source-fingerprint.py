#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib


SOURCE_REVISION = "9dc902243cdd7aba7ef269988b96f0aae6e037da"
SOURCE_REPOSITORY = "https://github.com/vicondoa/d2b"
INVENTORY_REVISION = "9dc902243cdd7aba7ef269988b96f0aae6e037da"
INVENTORY_SHA256 = "35c33c2e23e1b9f03b5abc3bbca2d3320e38c42dfc7aceb7e3476d28210cde8c"
DISTRIBUTION_ID = "d2b-client-toolkit"
DISTRIBUTION_FINGERPRINT = (
    "5a20cef3a64281df819eeb76bdfe385999755479b467b559653011582fb9c043"
)
SOURCE_GROUPS = {
    "workspace-manifest": "7cff164deff5b221f775ecc8e7e75a73c1e2e8e2ff863bfe9dde3250dbc35b5a",
    "contracts-package": "f9fe0276b02f27e172684d266b7d5e0c182f0d3fa960d98d9a7a4083f42fe471",
    "session-runtime": "f732a64c9944e040b69b95ed3c92e28847febd9ab34bfe13fec1da3aba523a39",
    "unix-session": "58afc178aa2fb742a78cac702c02e9178e523c1d645ed9c7e65b0944a2891b8c",
    "client": "8dfdfdda74a920d6d26c01fd5c014ff4f6cf5e3b4aa7520185efaccfc087f57e",
    "public-contract-artifacts": (
        "5a5a7818e9133d6097cb2d0488e212bbbab9a0bc74b40bf519ae6c4e4d1dcafb"
    ),
}
CANONICAL_PACKAGES = (
    "d2b-client",
    "d2b-contracts",
    "d2b-session",
    "d2b-session-unix",
)
SOURCE_SUPPLEMENTS = {
    "docs/reference/toolkit-source-contract.md",
    "docs/reference/v2-foundation-crates.md",
}
REMOVED_PROTOCOL_ROOTS = (
    "crates/d2b-client",
    "crates/d2b-toolkit-core",
    "crates/d2b-wayland-proxy",
)
EXPECTED_DISTRIBUTION_CRATES = {
    "crates/d2b-client-toolkit/Cargo.toml": "d2b-client-toolkit",
    "crates/d2b-client-toolkit-colors/Cargo.toml": "d2b-client-toolkit-colors",
    "crates/d2b-client-toolkit-waybar/Cargo.toml": "d2b-client-toolkit-waybar",
}


def fail(message: str) -> None:
    raise RuntimeError(message)


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_bytes(repository: Path, source: Path, relative: str) -> bytes:
    if relative in SOURCE_SUPPLEMENTS:
        path = repository / "canonical-source-artifacts" / relative
    else:
        path = source / relative
    if not path.is_file():
        fail(f"canonical source file is missing: {relative}")
    return path.read_bytes()


def fingerprint(
    domain: str,
    identifier: str,
    repository: Path,
    source: Path,
    paths: list[str],
) -> str:
    digest = hashlib.sha256()
    digest.update(domain.encode())
    digest.update(b"\0")
    digest.update(identifier.encode())
    digest.update(b"\0")
    for relative in paths:
        encoded = relative.encode()
        content = source_bytes(repository, source, relative)
        digest.update(len(encoded).to_bytes(8, "big"))
        digest.update(encoded)
        digest.update(len(content).to_bytes(8, "big"))
        digest.update(content)
    return digest.hexdigest()


def canonical_source_from_cargo_cache() -> Path:
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    candidates = []
    for manifest in cargo_home.glob(
        "git/checkouts/*/*/packages/d2b-client/Cargo.toml"
    ):
        root = manifest.resolve().parents[2]
        revision = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "HEAD"],
            check=False,
            capture_output=True,
            text=True,
        )
        if revision.returncode == 0 and revision.stdout.strip() == SOURCE_REVISION:
            candidates.append(root)
    if not candidates:
        fail("the exact canonical d2b checkout is absent from Cargo's cache")
    return sorted(candidates)[0]


def verify_pins(repository: Path) -> None:
    workspace = tomllib.loads((repository / "Cargo.toml").read_text())
    source_metadata = workspace["workspace"]["metadata"]["d2b-source"]
    if source_metadata != {
        "repository": SOURCE_REPOSITORY,
        "revision": SOURCE_REVISION,
        "distribution-fingerprint": DISTRIBUTION_FINGERPRINT,
    }:
        fail("Cargo workspace source metadata drifted")

    dependencies = workspace["workspace"]["dependencies"]
    for package in CANONICAL_PACKAGES:
        dependency = dependencies.get(package)
        if not isinstance(dependency, dict):
            fail(f"{package} is not an explicit canonical dependency")
        if dependency.get("git") != SOURCE_REPOSITORY:
            fail(f"{package} repository drifted")
        if dependency.get("rev") != SOURCE_REVISION:
            fail(f"{package} revision drifted")
        if dependency.get("default-features") is not False:
            fail(f"{package} must disable default features")

    lock = tomllib.loads((repository / "Cargo.lock").read_text())
    expected_source = f"git+{SOURCE_REPOSITORY}?rev={SOURCE_REVISION}#{SOURCE_REVISION}"
    locked_sources = {
        package["name"]: package.get("source")
        for package in lock["package"]
        if package["name"] in CANONICAL_PACKAGES
    }
    if locked_sources != {package: expected_source for package in CANONICAL_PACKAGES}:
        fail("Cargo.lock does not bind every canonical crate to the exact source revision")

    flake_lock = json.loads((repository / "flake.lock").read_text())
    d2b_node_name = flake_lock["nodes"]["root"]["inputs"].get("d2b-src")
    if not isinstance(d2b_node_name, str):
        fail("flake.lock is missing the d2b-src input")
    d2b_lock = flake_lock["nodes"][d2b_node_name]["locked"]
    if (
        d2b_lock.get("owner") != "vicondoa"
        or d2b_lock.get("repo") != "d2b"
        or d2b_lock.get("rev") != SOURCE_REVISION
    ):
        fail("flake.lock d2b-src revision drifted")


def verify_distribution_ownership(repository: Path) -> None:
    for relative in REMOVED_PROTOCOL_ROOTS:
        if (repository / relative).exists():
            fail(f"removed protocol owner reappeared: {relative}")

    actual_manifests = {
        str(path.relative_to(repository))
        for path in (repository / "crates").glob("*/Cargo.toml")
    }
    if actual_manifests != set(EXPECTED_DISTRIBUTION_CRATES):
        fail("distribution crate inventory drifted")
    for relative, expected_name in EXPECTED_DISTRIBUTION_CRATES.items():
        manifest = tomllib.loads((repository / relative).read_text())
        package = manifest["package"]
        if package.get("name") != expected_name:
            fail(f"distribution package name drifted: {relative}")
        if package.get("publish") is not False:
            fail(f"distribution package became publishable: {expected_name}")

    facade_root = repository / "crates/d2b-client-toolkit/src"
    facade = (facade_root / "lib.rs").read_text()
    if facade.count(SOURCE_REVISION) != 1:
        fail("facade source revision constant drifted")
    if facade.count(DISTRIBUTION_FINGERPRINT) != 1:
        fail("facade source fingerprint constant drifted")
    forbidden_implementations = (
        "serde::",
        "Serialize",
        "Deserialize",
        "prost::",
        "protobuf::",
        "include_proto!",
        "read_frame",
        "write_frame",
        "HelloFrame",
        "PublicRequest",
        "PublicResponse",
        "ShellOp",
        "WorkloadOp",
    )
    for path in facade_root.glob("*.rs"):
        source = path.read_text()
        for forbidden in forbidden_implementations:
            if forbidden in source:
                fail(f"facade defines forbidden protocol surface {forbidden}: {path.name}")


def verify_inventory(repository: Path, source: Path) -> None:
    inventory_path = repository / "docs/reference/toolkit-source-contract.json"
    if sha256_file(inventory_path) != INVENTORY_SHA256:
        fail("the canonical toolkit source inventory changed")
    inventory = json.loads(inventory_path.read_text())
    policy = inventory["fingerprintPolicy"]
    if policy != {
        "algorithm": "sha256",
        "digestEncoding": "lowercase-hex",
        "distributionDomain": "d2b-toolkit-distribution-v1",
        "lengthEncoding": "u64-big-endian",
        "pathEncoding": "utf-8",
        "sourceGroupDomain": "d2b-toolkit-source-group-v1",
    }:
        fail("source fingerprint policy drifted")

    groups = {group["id"]: group for group in inventory["sourceGroups"]}
    selected_paths: set[str] = set()
    for identifier, expected_fingerprint in SOURCE_GROUPS.items():
        group = groups.get(identifier)
        if group is None or group["fingerprint"] != expected_fingerprint:
            fail(f"{identifier} source-group fingerprint declaration drifted")
        paths = [entry["path"] for entry in group["files"]]
        if paths != sorted(set(paths)):
            fail(f"{identifier} paths are not sorted and unique")
        for entry in group["files"]:
            content = source_bytes(repository, source, entry["path"])
            if hashlib.sha256(content).hexdigest() != entry["sha256"]:
                fail(f"canonical source digest drifted: {entry['path']}")
        actual = fingerprint(
            policy["sourceGroupDomain"], identifier, repository, source, paths
        )
        if actual != expected_fingerprint:
            fail(f"{identifier} source-group content drifted")
        selected_paths.update(paths)

    distribution = next(
        (
            entry
            for entry in inventory["distributions"]
            if entry["id"] == DISTRIBUTION_ID
        ),
        None,
    )
    if distribution is None:
        fail("client toolkit distribution is absent from the source inventory")
    if distribution["sourceGroups"] != list(SOURCE_GROUPS):
        fail("client toolkit source-group selection drifted")
    if distribution["fingerprint"] != DISTRIBUTION_FINGERPRINT:
        fail("client toolkit distribution fingerprint declaration drifted")
    actual = fingerprint(
        policy["distributionDomain"],
        DISTRIBUTION_ID,
        repository,
        source,
        sorted(selected_paths),
    )
    if actual != DISTRIBUTION_FINGERPRINT:
        fail("client toolkit distribution content drifted")

    pin = json.loads((repository / "docs/reference/source-pin.json").read_text())
    if pin != {
        "schemaVersion": 1,
        "canonicalRepository": SOURCE_REPOSITORY,
        "sourceRevision": SOURCE_REVISION,
        "inventoryRevision": INVENTORY_REVISION,
        "distributionId": DISTRIBUTION_ID,
        "distributionFingerprint": DISTRIBUTION_FINGERPRINT,
        "inventorySha256": INVENTORY_SHA256,
    }:
        fail("source-pin.json drifted")


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Verify the exact canonical d2b client source distribution"
    )
    parser.add_argument(
        "--source",
        type=Path,
        help="canonical d2b source root (otherwise use D2B_CANONICAL_SOURCE or Cargo's cache)",
    )
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    source = args.source
    if source is None and os.environ.get("D2B_CANONICAL_SOURCE"):
        source = Path(os.environ["D2B_CANONICAL_SOURCE"])
    if source is None:
        source = canonical_source_from_cargo_cache()
    source = source.resolve()

    verify_pins(repository)
    verify_distribution_ownership(repository)
    verify_inventory(repository, source)
    print(
        f"{DISTRIBUTION_ID}: {SOURCE_REVISION} "
        f"{DISTRIBUTION_FINGERPRINT}"
    )
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (KeyError, OSError, RuntimeError) as error:
        print(f"source fingerprint check failed: {error}", file=sys.stderr)
        sys.exit(1)

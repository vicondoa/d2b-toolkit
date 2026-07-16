#!/usr/bin/env python3

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib


SOURCE_REVISION = "7e94327951d30913a1a6e0e7a47d4a24b462deff"
SOURCE_REPOSITORY = "https://github.com/vicondoa/d2b"
INVENTORY_REVISION = "d5a913922eb019ed83a16e4e64f562303b31ecf7"
INVENTORY_SHA256 = "71e783b9c8f98b3ac62a067bbe68da944b5758193b0d7cf0d7886fce29ef00cb"
DISTRIBUTION_ID = "d2b-client-toolkit"
DISTRIBUTION_FINGERPRINT = (
    "0401d1f463d9dad49efd663d1493184e42954f624a029fbfd41c49f0323e5708"
)
SOURCE_GROUPS = {
    "workspace-manifest": "5bdcccc3f279bb763208c0afdd4e6cd9de04417744343ba261c0d5888994b6af",
    "contracts-package": "480f29794edabf36c4c6fbc6feeee7fd17e0a716bbf9022b4a4ef4ca8eea4af4",
    "session-runtime": "375fbf89c82ee939cc12f15cad9847fcfac5876219bbd6555ea7f2f7f990aa70",
    "unix-session": "80cf8908ae8182b8597e36cbd7a9a57f61c51a2f6ba65495ed5ca44007ce98d4",
    "client": "fce5dcafee09c40bb963f34cc00c1fb96c446ba68750f21c778975acc371f89a",
    "public-contract-artifacts": (
        "c3a37248d7376827409d897eac7a1b30dcd39f3187261cf4098b6dbcc577459b"
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
        fail("the exact W9 toolkit source inventory changed")
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

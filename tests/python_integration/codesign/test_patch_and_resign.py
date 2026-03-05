"""
Test patching conda-repackaged binaries and re-signing with arwen.

For each binary in tests/data/macho/codesign/conda-repackaged/:
1. Copy to temp
2. Find a long ASCII string and replace it with random bytes (same length)
3. Verify the original ad-hoc signature is now broken
4. Re-sign with arwen's adhoc-sign
5. Verify the new signature is valid using macOS codesign -v
"""

import os
import random
import shutil
import string
import subprocess
import sys
from pathlib import Path

import pytest

CONDA_REPACKAGED_DIR = (
    Path(__file__).parent.parent.parent / "data" / "macho" / "codesign" / "conda-repackaged"
)

BINARIES = sorted(CONDA_REPACKAGED_DIR.glob("*")) if CONDA_REPACKAGED_DIR.exists() else []


def find_patchable_string(data: bytes, min_length: int = 32) -> tuple[int, int] | None:
    """Find a long ASCII string in binary data suitable for patching.

    Returns (offset, length) of the first string >= min_length, or None.
    """
    i = 0
    while i < len(data):
        if 0x20 <= data[i] <= 0x7E:
            start = i
            while i < len(data) and 0x20 <= data[i] <= 0x7E:
                i += 1
            length = i - start
            if length >= min_length:
                return start, length
        else:
            i += 1
    return None


def patch_binary(path: Path) -> bool:
    """Patch a binary by replacing a long ASCII string with random chars.

    Returns True if patching succeeded.
    """
    data = bytearray(path.read_bytes())
    result = find_patchable_string(data)
    if result is None:
        return False

    offset, length = result
    rng = random.Random(42)
    replacement = bytes(ord(rng.choice(string.ascii_letters)) for _ in range(length))
    data[offset : offset + length] = replacement
    path.write_bytes(bytes(data))
    return True


def codesign_verify(path: Path) -> bool:
    """Return True if macOS codesign -v --strict accepts the binary."""
    result = subprocess.run(
        ["codesign", "-v", "--strict", str(path)],
        capture_output=True,
        text=True,
        timeout=10,
    )
    return result.returncode == 0


def arwen_adhoc_sign(arwen_bin: Path, target: Path, identifier: str) -> subprocess.CompletedProcess:
    """Run arwen macho adhoc-sign on a binary."""
    return subprocess.run(
        [str(arwen_bin), "macho", "adhoc-sign", "--identifier", identifier, str(target)],
        capture_output=True,
        text=True,
        timeout=30,
    )


@pytest.fixture(scope="session")
def arwen_bin() -> Path:
    """Build arwen and return the path to the release binary."""
    project_root = Path(__file__).parent.parent.parent.parent
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=project_root,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        pytest.fail(f"Failed to build arwen: {result.stderr}")
    path = project_root / "target" / "release" / "arwen"
    if not path.exists():
        pytest.fail(f"arwen binary not found at {path}")
    return path


@pytest.mark.skipif(sys.platform != "darwin", reason="macOS only")
@pytest.mark.skipif(not BINARIES, reason="No conda-repackaged binaries found")
@pytest.mark.parametrize("binary_path", BINARIES, ids=lambda p: p.name)
def test_patch_and_resign(arwen_bin: Path, binary_path: Path, tmp_path: Path):
    """Patch a binary to break its signature, re-sign with arwen, verify with codesign."""
    # Copy binary to temp
    work = tmp_path / binary_path.name
    shutil.copy(binary_path, work)
    os.chmod(work, 0o755)

    # Verify the original is validly signed
    assert codesign_verify(work), f"Original binary {binary_path.name} has invalid signature"

    # Patch the binary to break the signature
    patched = patch_binary(work)
    assert patched, f"Could not find a patchable string in {binary_path.name}"

    # Signature should now be broken
    assert not codesign_verify(work), (
        f"Signature should be broken after patching {binary_path.name}"
    )

    # Re-sign with arwen
    identifier = f"com.test.{binary_path.stem}"
    result = arwen_adhoc_sign(arwen_bin, work, identifier)
    assert result.returncode == 0, (
        f"arwen adhoc-sign failed for {binary_path.name}: {result.stderr or result.stdout}"
    )

    # Verify the new signature is valid
    assert codesign_verify(work), (
        f"Re-signed binary {binary_path.name} has invalid signature"
    )

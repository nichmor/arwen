#!/usr/bin/env python3
"""
Test framework for comparing arwen's ad-hoc code signing against Apple's codesign.

This script tests ad-hoc code signing and compares the output of arwen's
implementation (arwen-codesign crate) against Apple's official codesign tool.

Usage with pytest:
    pytest tests/python_integration/codesign/test_codesign.py [options]

Pytest options:
    --goblin-tool PATH       Path to the codesign tool binary
    --strict-codesign        Require bit-for-bit identical output
    --keep-temp              Keep temporary files for inspection
    -v                       Verbose output
"""

import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from enum import Enum, auto
from pathlib import Path
from typing import Optional, List, Tuple

import pytest


class TestResult(Enum):
    PASS = auto()
    FAIL = auto()
    SKIP = auto()
    ERROR = auto()


@dataclass
class TestCase:
    name: str
    result: TestResult = TestResult.SKIP
    error_message: str = ""
    goblin_size: int = 0
    apple_size: int = 0
    diff_offset: Optional[int] = None


# Helper functions are now in conftest.py


def get_codesign_info(path: Path) -> dict:
    """Get code signature information using codesign -d."""
    info = {
        "signed": False,
        "identifier": None,
        "flags": None,
        "adhoc": False,
        "linker_signed": False,
    }

    try:
        result = subprocess.run(
            ["codesign", "-d", "-vvv", str(path)],
            capture_output=True,
            text=True,
            timeout=10,
        )
        output = result.stderr  # codesign outputs to stderr

        if "code object is not signed at all" in output.lower():
            return info

        info["signed"] = True

        for line in output.split("\n"):
            if line.startswith("Identifier="):
                info["identifier"] = line.split("=", 1)[1]
            elif line.startswith("CodeDirectory"):
                if "flags=0x" in line:
                    # Parse flags
                    flags_part = line.split("flags=")[1].split()[0]
                    if "adhoc" in line.lower():
                        info["adhoc"] = True
                    if "linker-signed" in line.lower() or "linkerSigned" in line:
                        info["linker_signed"] = True

    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass

    return info


def compare_binaries_bitwise(path1: Path, path2: Path) -> tuple[bool, str, Optional[int]]:
    """Compare two binary files byte-by-byte."""
    data1 = path1.read_bytes()
    data2 = path2.read_bytes()

    if data1 == data2:
        return True, "Identical", None

    # Find first difference
    min_len = min(len(data1), len(data2))
    for i in range(min_len):
        if data1[i] != data2[i]:
            context_start = max(0, i - 8)
            context_end = min(min_len, i + 8)
            hex1 = data1[context_start:context_end].hex()
            hex2 = data2[context_start:context_end].hex()
            return False, f"Diff at 0x{i:x}: {hex1} vs {hex2}", i

    if len(data1) != len(data2):
        return False, f"Size diff: {len(data1)} vs {len(data2)} bytes", min_len

    return False, "Unknown difference", None


def compare_code_signature(path1: Path, path2: Path) -> tuple[bool, str]:
    """Compare the code signatures of two binaries structurally."""
    info1 = get_codesign_info(path1)
    info2 = get_codesign_info(path2)

    if info1["signed"] != info2["signed"]:
        return False, f"Signed status differs: {info1['signed']} vs {info2['signed']}"

    if not info1["signed"]:
        return True, "Both unsigned"

    differences = []

    if info1["identifier"] != info2["identifier"]:
        differences.append(f"identifier: {info1['identifier']} vs {info2['identifier']}")

    if info1["adhoc"] != info2["adhoc"]:
        differences.append(f"adhoc: {info1['adhoc']} vs {info2['adhoc']}")

    if differences:
        return False, "; ".join(differences)

    return True, "Signatures match structurally"


def verify_signature(path: Path) -> tuple[bool, str]:
    """Verify a code signature using codesign -v.

    Returns (True, "") if valid, (False, error_message) if invalid.
    """
    try:
        result = subprocess.run(
            ["codesign", "-v", "--strict", str(path)],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode == 0:
            return True, ""
        # codesign outputs errors to stderr
        error = result.stderr.strip() or result.stdout.strip() or "Unknown verification error"
        return False, error
    except subprocess.TimeoutExpired:
        return False, "Verification timeout"
    except FileNotFoundError:
        return False, "codesign not found"


def sign_with_apple_codesign(input_path: Path, output_path: Path, identifier: str,
                             hardened_runtime: bool = False,
                             preserve_entitlements: bool = False) -> tuple[bool, str]:
    """Sign a binary with Apple's codesign tool."""
    shutil.copy(input_path, output_path)
    os.chmod(output_path, 0o755)

    cmd = ["codesign", "-s", "-", "-f", "-i", identifier]

    if hardened_runtime:
        cmd.extend(["--options", "runtime"])

    cmd.append(str(output_path))

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            return False, result.stderr.strip()
        return True, ""
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except FileNotFoundError:
        return False, "codesign not found"


def sign_with_goblin(goblin_tool: Path, input_path: Path, output_path: Path,
                     identifier: str, hardened_runtime: bool = False,
                     preserve_entitlements: bool = False) -> tuple[bool, str]:
    """Sign a binary using arwen CLI."""
    shutil.copy(input_path, output_path)
    os.chmod(output_path, 0o755)

    # Detect if binary is linker-signed
    info = get_codesign_info(input_path)
    is_linker_signed = info.get("linker_signed", False)

    # Use arwen CLI: arwen macho adhoc-sign [OPTIONS] --identifier <ID> <FILE>
    cmd = [str(goblin_tool), "macho", "adhoc-sign", "--identifier", identifier]

    if hardened_runtime:
        cmd.append("--hardened-runtime")

    if preserve_entitlements:
        cmd.append("--preserve-entitlements")

    if is_linker_signed:
        cmd.append("--linker-signed")

    cmd.append(str(output_path))

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            return False, result.stderr.strip() or result.stdout.strip() or "Unknown error"
        return True, ""
    except subprocess.TimeoutExpired:
        return False, "Timeout"
    except FileNotFoundError:
        return False, f"Arwen CLI not found at {goblin_tool}"


def _run_adhoc_signing_test(goblin_tool: Path, binary_path: Path, tmpdir: Path,
                             identifier: str, hardened_runtime: bool = False,
                             verbose: bool = False, strict: bool = False) -> TestCase:
    """Test ad-hoc signing comparing Apple and arwen implementations.

    Args:
        strict: If True, require bit-for-bit identical output.
                If False (default), pass if signatures are structurally equivalent.
    """
    name = f"adhoc{'_hardened' if hardened_runtime else ''}"
    tc = TestCase(name=name)

    # Use the same filename for both to ensure identical identifiers
    # (codesign uses filename as default identifier component)
    test_file = tmpdir / "testbinary"
    apple_result = tmpdir / "apple_result"

    # Sign with Apple's codesign
    apple_ok, apple_err = sign_with_apple_codesign(
        binary_path, test_file, identifier, hardened_runtime
    )
    if not apple_ok:
        tc.result = TestResult.SKIP
        tc.error_message = f"Apple codesign failed: {apple_err}"
        return tc

    # Save Apple's result
    shutil.copy(test_file, apple_result)
    tc.apple_size = apple_result.stat().st_size

    # Sign with goblin tool (overwrites test_file)
    goblin_ok, goblin_err = sign_with_goblin(
        goblin_tool, binary_path, test_file, identifier, hardened_runtime
    )
    if not goblin_ok:
        tc.result = TestResult.ERROR
        tc.error_message = f"Goblin signing failed: {goblin_err}"
        return tc

    tc.goblin_size = test_file.stat().st_size

    # First verify that macOS accepts our signature
    verify_ok, verify_err = verify_signature(test_file)
    if not verify_ok:
        tc.result = TestResult.FAIL
        tc.error_message = f"Goblin signature rejected by codesign -v: {verify_err}"
        return tc

    # Compare byte-by-byte first
    match, msg, diff_offset = compare_binaries_bitwise(test_file, apple_result)
    tc.diff_offset = diff_offset

    if match:
        tc.result = TestResult.PASS
        if verbose:
            print(f"    Bit-for-bit identical!")
    else:
        # Check if signatures are at least structurally equivalent
        struct_match, struct_msg = compare_code_signature(test_file, apple_result)
        if struct_match:
            if strict:
                tc.result = TestResult.FAIL
                tc.error_message = f"Structural match but byte diff: {msg}"
            else:
                # Non-strict mode: structural match is good enough
                tc.result = TestResult.PASS
                if verbose:
                    print(f"    Structural match (not bit-for-bit)")
        else:
            tc.result = TestResult.FAIL
            tc.error_message = f"Signature mismatch: {struct_msg}; {msg}"

    return tc


def _run_resign_test(goblin_tool: Path, binary_path: Path, tmpdir: Path,
                      identifier: str, verbose: bool = False, strict: bool = False) -> TestCase:
    """Test re-signing a linker-signed binary."""
    tc = TestCase(name="resign_linker_signed")

    test_file = tmpdir / "testbinary"
    apple_result = tmpdir / "apple_result"

    # First check if the binary is linker-signed
    info = get_codesign_info(binary_path)
    if not info.get("linker_signed"):
        tc.result = TestResult.SKIP
        tc.error_message = "Binary is not linker-signed"
        return tc

    # Sign with Apple's codesign
    apple_ok, apple_err = sign_with_apple_codesign(binary_path, test_file, identifier)
    if not apple_ok:
        tc.result = TestResult.SKIP
        tc.error_message = f"Apple codesign failed: {apple_err}"
        return tc

    shutil.copy(test_file, apple_result)
    tc.apple_size = apple_result.stat().st_size

    # Sign with goblin tool
    goblin_ok, goblin_err = sign_with_goblin(goblin_tool, binary_path, test_file, identifier)
    if not goblin_ok:
        tc.result = TestResult.ERROR
        tc.error_message = f"Goblin signing failed: {goblin_err}"
        return tc

    tc.goblin_size = test_file.stat().st_size

    # First verify that macOS accepts our signature
    verify_ok, verify_err = verify_signature(test_file)
    if not verify_ok:
        tc.result = TestResult.FAIL
        tc.error_message = f"Goblin signature rejected by codesign -v: {verify_err}"
        return tc

    # Compare
    match, msg, diff_offset = compare_binaries_bitwise(test_file, apple_result)
    tc.diff_offset = diff_offset

    if match:
        tc.result = TestResult.PASS
        if verbose:
            print(f"    Bit-for-bit identical!")
    else:
        # Check if signatures are at least structurally equivalent
        struct_match, struct_msg = compare_code_signature(test_file, apple_result)
        if struct_match:
            if strict:
                tc.result = TestResult.FAIL
                tc.error_message = f"Structural match but byte diff: {msg}"
            else:
                # Non-strict mode: structural match is good enough
                tc.result = TestResult.PASS
                if verbose:
                    print(f"    Structural match (not bit-for-bit)")
        else:
            tc.result = TestResult.FAIL
            tc.error_message = f"Signature mismatch: {struct_msg}; {msg}"

    return tc


# ============================================================================
# Test Helper - Creates identifier for binary
# ============================================================================

def get_identifier_for_binary(binary_path: Path) -> str:
    """Get a consistent identifier for a test binary."""
    return f"com.test.{binary_path.stem}"


# ============================================================================
# Pytest Tests
# ============================================================================
# Note: Fixtures and pytest configuration are in conftest.py

@pytest.mark.codesign
@pytest.mark.macos_only
@pytest.mark.skipif(sys.platform != "darwin", reason="Only runs on macOS")
@pytest.mark.parametrize("hardened_runtime", [False, True], ids=["basic", "hardened"])
def test_adhoc_signing_against_apple(
    goblin_tool,
    test_binaries,
    tmp_path,
    strict_mode,
    verbose_mode,
    hardened_runtime,
):
    """Test ad-hoc signing against Apple's codesign tool.

    This test compares arwen's ad-hoc signing implementation against Apple's
    official codesign tool, verifying both structural equivalence and (optionally)
    bit-for-bit compatibility.

    Args:
        goblin_tool: Path to the codesign tool binary (fixture)
        test_binaries: List of test binaries to sign (fixture)
        tmp_path: Temporary directory for test files (pytest builtin fixture)
        strict_mode: Whether to require bit-for-bit compatibility (fixture)
        verbose_mode: Whether to enable verbose output (fixture)
        hardened_runtime: Whether to enable hardened runtime flag
    """
    # Test each binary
    for bin_type, binary_path in test_binaries:
        # Create identifier
        identifier = get_identifier_for_binary(binary_path)

        # Create working directory for this test
        work_dir = tmp_path / f"{binary_path.stem}_{hardened_runtime}"
        work_dir.mkdir(exist_ok=True)

        # Run the test
        tc = _run_adhoc_signing_test(
            goblin_tool,
            binary_path,
            work_dir,
            identifier,
            hardened_runtime=hardened_runtime,
            verbose=verbose_mode,
            strict=strict_mode,
        )

        # Build descriptive message
        test_desc = f"{bin_type} ({binary_path.name})"
        if hardened_runtime:
            test_desc += " with hardened runtime"

        # Assert based on result
        if tc.result == TestResult.FAIL:
            fail_msg = f"{test_desc} failed: {tc.error_message}"
            if tc.goblin_size and tc.apple_size:
                fail_msg += f"\nSizes: arwen={tc.goblin_size}, apple={tc.apple_size}"
            if tc.diff_offset is not None:
                fail_msg += f"\nFirst diff at offset: 0x{tc.diff_offset:x}"
            pytest.fail(fail_msg)
        elif tc.result == TestResult.ERROR:
            pytest.fail(f"{test_desc} encountered error: {tc.error_message}")
        elif tc.result == TestResult.SKIP:
            pytest.skip(f"{test_desc} skipped: {tc.error_message}")

        # If we get here, test passed
        if verbose_mode:
            print(f"✓ {test_desc} passed")


@pytest.mark.codesign
@pytest.mark.macos_only
@pytest.mark.skipif(sys.platform != "darwin", reason="Only runs on macOS")
def test_resign_linker_signed(
    goblin_tool,
    test_binaries,
    tmp_path,
    strict_mode,
    verbose_mode,
):
    """Test re-signing a linker-signed binary.

    Args:
        goblin_tool: Path to the codesign tool binary (fixture)
        test_binaries: List of test binaries (fixture)
        tmp_path: Temporary directory for test files (pytest builtin fixture)
        verbose_mode: Whether to enable verbose output (fixture)
    """
    # Find a linker-signed binary
    for bin_type, binary_path in test_binaries:
        info = get_codesign_info(binary_path)
        if not info.get("linker_signed"):
            continue

        # Create identifier
        identifier = get_identifier_for_binary(binary_path)

        # Create working directory
        work_dir = tmp_path / f"{binary_path.stem}_resign"
        work_dir.mkdir(exist_ok=True)

        # Run the resign test
        tc = _run_resign_test(
            goblin_tool,
            binary_path,
            work_dir,
            identifier,
            verbose=verbose_mode,
            strict=strict_mode,
        )

        test_desc = f"Re-sign {bin_type} ({binary_path.name})"

        # Assert based on result
        if tc.result == TestResult.FAIL:
            fail_msg = f"{test_desc} failed: {tc.error_message}"
            if tc.goblin_size and tc.apple_size:
                fail_msg += f"\nSizes: arwen={tc.goblin_size}, apple={tc.apple_size}"
            if tc.diff_offset is not None:
                fail_msg += f"\nFirst diff at offset: 0x{tc.diff_offset:x}"
            pytest.fail(fail_msg)
        elif tc.result == TestResult.ERROR:
            pytest.fail(f"{test_desc} encountered error: {tc.error_message}")
        elif tc.result == TestResult.SKIP:
            pytest.skip(f"{test_desc} skipped: {tc.error_message}")

        # If we get here, test passed
        if verbose_mode:
            print(f"✓ {test_desc} passed")

        # We only need to test one linker-signed binary
        return

    # If we didn't find any linker-signed binaries, skip
    pytest.skip("No linker-signed binaries available for testing")

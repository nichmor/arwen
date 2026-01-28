"""Pytest configuration for codesign integration tests."""

import os
import shutil
import subprocess
from pathlib import Path
from typing import List, Tuple, Optional

import pytest


def pytest_addoption(parser):
    """Add custom pytest command line options for codesign tests."""
    parser.addoption(
        "--goblin-tool",
        type=str,
        default=None,
        help="Path to arwen CLI binary",
    )
    parser.addoption(
        "--strict-codesign",
        action="store_true",
        default=False,
        help="Require bit-for-bit identical output (default: structural equivalence)",
    )
    parser.addoption(
        "--keep-temp",
        action="store_true",
        default=False,
        help="Keep temporary files for inspection",
    )


def pytest_configure(config):
    """Configure pytest for codesign tests."""
    config.addinivalue_line(
        "markers", "codesign: mark test as a code signing integration test"
    )
    config.addinivalue_line(
        "markers", "macos_only: mark test as macOS-only"
    )


# ============================================================================
# Helper Functions
# ============================================================================

def build_goblin_tool(project_root: Path) -> Optional[Path]:
    """Build the arwen CLI tool."""
    print("Building arwen CLI...")

    # Build the main arwen binary
    result = subprocess.run(
        ["cargo", "build", "--release"],
        cwd=project_root,
        capture_output=True,
        text=True,
    )

    if result.returncode != 0:
        print(f"Failed to build arwen: {result.stderr}")
        return None

    tool_path = project_root / "target" / "release" / "arwen"
    if tool_path.exists():
        return tool_path

    print(f"Built successfully but tool not found at {tool_path}")
    return None


def get_prebuilt_assets() -> Optional[Path]:
    """Get the path to pre-built test assets if available."""
    script_dir = Path(__file__).parent
    # Assets are now in tests/data/macho/codesign/
    assets_dir = script_dir.parent.parent / "data" / "macho" / "codesign"
    if assets_dir.exists() and (assets_dir / "test_exe_linker_signed").exists():
        return assets_dir
    return None


def copy_asset_to_temp(assets_dir: Path, asset_name: str, tmpdir: Path) -> Optional[Path]:
    """Copy a pre-built asset to the temp directory."""
    src = assets_dir / asset_name
    if not src.exists():
        return None
    dst = tmpdir / asset_name
    shutil.copy(src, dst)
    os.chmod(dst, 0o755)
    return dst


def create_test_binary(tmpdir: Path, name: str, is_dylib: bool = False) -> Optional[Path]:
    """Create a minimal test binary using clang."""
    MINIMAL_MAIN = '''
int main(void) {
    return 0;
}
'''
    MINIMAL_DYLIB = '''
__attribute__((visibility("default")))
int add(int a, int b) {
    return a + b;
}
'''
    source_file = tmpdir / f"{name}.c"
    output_file = tmpdir / (f"lib{name}.dylib" if is_dylib else name)

    source_code = MINIMAL_DYLIB if is_dylib else MINIMAL_MAIN
    source_file.write_text(source_code)

    cmd = ["clang", "-o", str(output_file), str(source_file)]
    if is_dylib:
        cmd.extend(["-dynamiclib", "-install_name", f"@rpath/lib{name}.dylib"])

    try:
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            print(f"Failed to compile {name}: {result.stderr}")
            return None
        return output_file
    except (subprocess.TimeoutExpired, FileNotFoundError) as e:
        print(f"Failed to compile {name}: {e}")
        return None


# ============================================================================
# Pytest Fixtures
# ============================================================================

@pytest.fixture(scope="session")
def goblin_tool(request) -> Path:
    """Pytest fixture that builds or locates the arwen CLI tool.

    Can be overridden via pytest command line: --goblin-tool=<path>

    Note: The fixture is named 'goblin_tool' for historical reasons (migrated from goblin-ext),
    but it now provides the arwen CLI binary.
    """
    # Check if provided via pytest command line
    tool_path = request.config.getoption("--goblin-tool", default=None)

    if tool_path is not None:
        tool_path = Path(tool_path)
        if not tool_path.exists():
            pytest.fail(f"Codesign tool not found at {tool_path}")
        return tool_path

    # Build the tool
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent  # Go up to workspace root
    tool_path = build_goblin_tool(project_root)

    if tool_path is None:
        pytest.fail("Could not build arwen codesign tool")

    if not tool_path.exists():
        pytest.fail(f"Codesign tool not found at {tool_path}")

    return tool_path


@pytest.fixture(scope="session")
def test_binaries(tmp_path_factory) -> List[Tuple[str, Path]]:
    """Fixture providing test binaries for codesign tests.

    Returns a list of (binary_type, binary_path) tuples.
    """
    tmpdir = tmp_path_factory.mktemp("binaries")
    test_bins = []

    # Try to use pre-built assets first
    # assets_dir = get_prebuilt_assets()
    assets_dir = None  # Disable pre-built assets for now

    if assets_dir:
        print(f"Using pre-built assets from: {assets_dir}")

        # Copy assets to temp directory for testing
        exe = copy_asset_to_temp(assets_dir, "test_exe_linker_signed", tmpdir)
        if exe:
            test_bins.append(("executable", exe))

        # Note: Fat (universal) binaries are not yet supported by adhoc_sign
        # fat = copy_asset_to_temp(assets_dir, "test_exe_fat", tmpdir)
        # if fat:
        #     test_bins.append(("fat_binary", fat))
    else:
        # Fall back to compiling test binaries
        print("Creating test binaries (no pre-built assets found)...")

        exe = create_test_binary(tmpdir, "test_exe", is_dylib=False)
        if exe:
            test_bins.append(("executable", exe))

        dylib = create_test_binary(tmpdir, "test", is_dylib=True)
        if dylib:
            test_bins.append(("dylib", dylib))

    if not test_bins:
        pytest.fail("Could not create any test binaries")

    return test_bins


@pytest.fixture
def strict_mode(request) -> bool:
    """Fixture for strict mode setting."""
    return request.config.getoption("--strict-codesign", default=False)


@pytest.fixture
def keep_temp(request) -> bool:
    """Fixture for keep-temp setting."""
    return request.config.getoption("--keep-temp", default=False)


@pytest.fixture
def verbose_mode(request) -> bool:
    """Fixture for verbose mode setting."""
    return request.config.getoption("-v", default=False) > 0

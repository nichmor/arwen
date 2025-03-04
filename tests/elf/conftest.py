import pytest
import shutil
import os
from pathlib import Path


@pytest.fixture
def arwen() -> Path:
    return Path(__file__).parent.joinpath("../../target/release/arwen")


@pytest.fixture
def tmp_files(tmp_path: Path) -> Path:
    dot_pixi = tmp_path.joinpath("arwen-test")
    dot_pixi.mkdir()

    return tmp_path


@pytest.fixture
def bin_for_patchelf(tmp_files):
    current_dir = Path(__file__).parent

    test_linux_bash = os.path.join(current_dir, "linux-x64-bash")

    test_bin_patchelf = os.path.join(tmp_files, "test_patchelf")
    shutil.copy2(test_linux_bash, test_bin_patchelf)
    return test_bin_patchelf


@pytest.fixture
def bin_for_arwen(tmp_files):
    current_dir = Path(__file__).parent

    test_linux_bash = os.path.join(current_dir, "linux-x64-bash")

    test_bin_arwen = os.path.join(tmp_files, "test_arwen")
    shutil.copy2(test_linux_bash, test_bin_arwen)
    return test_bin_arwen

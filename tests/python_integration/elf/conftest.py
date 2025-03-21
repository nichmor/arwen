import pytest
import shutil
import os
from pathlib import Path

# from python_integration.conftest import tests_elf_data_dir


def tests_elf_data_dir() -> Path:
    path = Path(__file__).parent.parent.joinpath("../data/elf")
    return path


@pytest.fixture(params=[p for p in tests_elf_data_dir().glob("*/exec/*")])
def bin_for_patchelf(tmp_files, request):
    test_bin_patchelf = os.path.join(tmp_files, "test_patchelf")
    shutil.copy2(request.param, test_bin_patchelf)
    return test_bin_patchelf


@pytest.fixture(params=[p for p in tests_elf_data_dir().glob("*/exec/*")])
def bin_for_arwen(tmp_files, request):
    test_bin_arwen = os.path.join(tmp_files, "test_arwen")
    shutil.copy2(request.param, test_bin_arwen)
    return test_bin_arwen

from pathlib import Path
import shutil
import pytest


@pytest.fixture
def tmp_files(tmp_path: Path) -> Path:
    py_arwen_folder = tmp_path.joinpath("py_arwen_test")
    py_arwen_folder.mkdir()

    return py_arwen_folder


@pytest.fixture
def elf_bin(tmp_files):
    current_dir = Path(__file__).parent

    test_linux_bash = current_dir / "data" / "linux-x64-bash"

    test_bin_patchelf = Path(tmp_files) / "test_elf"
    shutil.copy2(test_linux_bash, test_bin_patchelf)
    return str(test_bin_patchelf)


@pytest.fixture
def macho_bin(tmp_files):
    current_dir = Path(__file__).parent

    test_linux_bash = current_dir / "data" / "hello_with_rpath"

    test_bin_macho = Path(tmp_files) / "test_macho"
    shutil.copy2(test_linux_bash, test_bin_macho)
    return str(test_bin_macho)


@pytest.fixture
def macho_dylib(tmp_files):
    current_dir = Path(__file__).parent

    test_linux_bash = current_dir / "data" / "libmylib.dylib"

    test_dylib_macho = Path(tmp_files) / "test_macho_dylib"
    shutil.copy2(test_linux_bash, test_dylib_macho)
    return str(test_dylib_macho)

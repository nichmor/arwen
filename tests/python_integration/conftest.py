import pytest
from pathlib import Path


@pytest.fixture
def arwen() -> Path:
    return Path(__file__).parent.joinpath("../../target/release/arwen")


# @pytest.fixture
def tests_elf_data_dir() -> Path:
    return Path(__file__).parent.joinpath("../data/elf")


@pytest.fixture
def tmp_files(tmp_path: Path) -> Path:
    dot_pixi = tmp_path.joinpath("arwen-test")
    dot_pixi.mkdir()

    return tmp_path

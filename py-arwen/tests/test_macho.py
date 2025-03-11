import os
from arwen.macho import MachoContainer


def test_from_path(macho_bin):
    # Assuming test_macho.bin is a valid Mach-O file for testing
    macho = MachoContainer.from_path(macho_bin)
    assert isinstance(macho, MachoContainer)


def test_add_rpath(macho_bin):
    macho = MachoContainer.from_path(macho_bin)
    macho.add_rpath("/new/rpath")


def test_change_rpath(macho_bin):
    macho = MachoContainer.from_path(macho_bin)
    macho.change_rpath("path_graf", "/new/rpath")


def test_remove_rpath(macho_bin):
    macho = MachoContainer.from_path(macho_bin)
    macho.remove_rpath("path_graf")


def test_change_install_id(macho_dylib):
    macho = MachoContainer.from_path(macho_dylib)
    macho.change_install_id("new_id")


def test_change_install_name(macho_bin):
    macho = MachoContainer.from_path(macho_bin)
    macho.change_install_name("/usr/lib/libSystem.B.dylib", "new_name")


def test_save(tmp_files, macho_bin):
    macho = MachoContainer.from_path(macho_bin)
    macho.save(os.path.join(tmp_files, "modified_macho.bin"))

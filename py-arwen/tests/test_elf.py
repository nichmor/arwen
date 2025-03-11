import os
from arwen.elf import ElfContainer


def test_from_path(elf_bin):
    # Assuming test_elf.bin is a valid ELF file for testing
    elf = ElfContainer.from_path(elf_bin)
    assert isinstance(elf, ElfContainer)


def test_add_runpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.add_runpath("/new/runpath")
    # Add assertions to verify the runpath was added


def test_set_runpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.set_runpath("/set/runpath")
    # Add assertions to verify the runpath was set


def test_remove_runpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.remove_runpath()
    # Add assertions to verify the runpath was removed


def test_get_runpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    runpath = elf.get_runpath()
    assert runpath is not None
    # Add assertions to verify the runpath


def test_force_rpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.force_rpath()
    # Add assertions to verify the rpath was forced


def test_shrink_rpath(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.shrink_rpath(["/prefix1", "/prefix2"])
    # Add assertions to verify the rpath was shrunk


def test_add_needed(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.add_needed(["libneeded.so"])
    # Add assertions to verify the needed library was added


def test_remove_needed(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.remove_needed(["libneeded.so"])
    # Add assertions to verify the needed library was removed


def test_replace_needed(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.replace_needed({"old_lib.so": "new_lib.so"})
    # Add assertions to verify the needed library was replaced


def test_get_needed(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    needed = elf.get_needed()
    assert isinstance(needed, list)
    # Add assertions to verify the needed libraries


def test_set_interpreter(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.set_interpreter("/new/interpreter")
    # Add assertions to verify the interpreter was set


def test_get_interpreter(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    interpreter = elf.get_interpreter()
    assert interpreter is not None
    # Add assertions to verify the interpreter


def test_set_os_abi(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.set_os_abi("freebsd")


def test_get_os_abi(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    os_abi = elf.get_os_abi()
    assert isinstance(os_abi, int)


def test_set_soname(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.set_soname("new_soname")
    soname = elf.get_soname()
    # Verify the SONAME was set
    # and we can get it
    assert soname == "new_soname"


def test_no_default_lib(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.no_default_lib()


def test_clear_version_symbol(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.clear_version_symbol("symbol_name")


def test_add_debug_tag(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.add_debug_tag()


def test_clear_exec_stack(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.clear_exec_stack()


def test_set_exec_stack(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.set_exec_stack()


def test_is_exec_stack(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    exec_stack = elf.is_exec_stack()
    assert exec_stack is not None


def test_rename_dynamic_symbols(elf_bin):
    elf = ElfContainer.from_path(elf_bin)
    elf.rename_dynamic_symbols({"old_symbol": "new_symbol"})


def test_save(elf_bin, tmp_files):
    elf = ElfContainer.from_path(elf_bin)
    elf.save(os.path.join(tmp_files, "modified_elf"))

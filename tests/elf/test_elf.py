#!/usr/bin/env python3
"""
Test suite for comparing arwen and patchelf functionality using pytest.
"""

import os
import shutil
import subprocess
import tempfile
import pytest


@pytest.fixture(scope="session")
def test_env():
    """Create test environment and compile test binaries."""
    # Create temporary test directory
    test_dir = tempfile.mkdtemp(prefix="arwen_test_")
    libs_dir = os.path.join(test_dir, "libs")
    os.makedirs(libs_dir, exist_ok=True)

    # Save current directory
    original_dir = os.getcwd()

    # Create test files
    _create_test_files(test_dir)

    # Compile test files
    _compile_test_files(test_dir, libs_dir)

    print(f"Test environment created at: {test_dir}")

    # Return test environment details
    yield {"test_dir": test_dir, "libs_dir": libs_dir, "original_dir": original_dir}

    # Clean up test environment
    shutil.rmtree(test_dir)
    print("Test environment cleaned up")


def _create_test_files(test_dir):
    """Create source files for testing."""
    # libfoo.c
    with open(os.path.join(test_dir, "libfoo.c"), "w") as f:
        f.write("""
#include <stdio.h>

void foo() {
    printf("Hello from libfoo!\\n");
}
""")

    # libbar.c
    with open(os.path.join(test_dir, "libbar.c"), "w") as f:
        f.write("""
#include <stdio.h>

void bar() {
    printf("Hello from libbar!\\n");
}
""")

    # main.c
    with open(os.path.join(test_dir, "main.c"), "w") as f:
        f.write("""
#include <stdio.h>

extern void foo();

int main() {
    printf("Starting main program\\n");
    foo();
    return 0;
}
""")

    # main_with_bar.c
    with open(os.path.join(test_dir, "main_with_bar.c"), "w") as f:
        f.write("""
#include <stdio.h>

extern void foo();
extern void bar();

int main() {
    printf("Starting main program\\n");
    foo();
    bar();
    return 0;
}
""")


def _compile_test_files(test_dir, libs_dir):
    """Compile test files."""
    # Compile libfoo.so
    subprocess.run(
        [
            "gcc",
            "-shared",
            "-o",
            os.path.join(libs_dir, "libfoo.so"),
            os.path.join(test_dir, "libfoo.c"),
            "-fPIC",
        ],
        check=True,
    )

    # Compile libbar.so
    subprocess.run(
        [
            "gcc",
            "-shared",
            "-o",
            os.path.join(libs_dir, "libbar.so"),
            os.path.join(test_dir, "libbar.c"),
            "-fPIC",
        ],
        check=True,
    )

    # Compile main with rpath
    subprocess.run(
        [
            "gcc",
            "-o",
            os.path.join(test_dir, "main_with_rpath"),
            os.path.join(test_dir, "main.c"),
            "-L",
            libs_dir,
            "-lfoo",
            "-Wl,-rpath,$ORIGIN/libs",
        ],
        check=True,
    )

    # Compile main without rpath
    subprocess.run(
        [
            "gcc",
            "-o",
            os.path.join(test_dir, "main_no_rpath"),
            os.path.join(test_dir, "main.c"),
            "-L",
            libs_dir,
            "-lfoo",
        ],
        check=True,
    )

    # Compile main with dependencies
    subprocess.run(
        [
            "gcc",
            "-o",
            os.path.join(test_dir, "main_with_deps"),
            os.path.join(test_dir, "main_with_bar.c"),
            "-L",
            libs_dir,
            "-lfoo",
            "-lbar",
            "-Wl,-rpath,$ORIGIN/libs",
        ],
        check=True,
    )


# @pytest.fixture
# def test_files(test_env):
#     """Set up test files for each test."""
#     test_dir = test_env["test_dir"]
#     libs_dir = test_env["libs_dir"]

#     # Create fresh copies of binaries for each test
#     # test_bin = os.path.join(test_dir, "main_with_rpath")


#     current_dir = Path(__file__).parent

#     test_linux_bash = os.path.join(current_dir, "linux-x64-bash")


#     test_bin_patchelf = os.path.join(test_dir, "test_patchelf")
#     test_bin_arwen = os.path.join(test_dir, "test_arwen")

#     shutil.copy2(test_linux_bash, test_bin_patchelf)
#     shutil.copy2(test_linux_bash, test_bin_arwen)

#     # # Library for testing
#     # test_lib = os.path.join(libs_dir, "libfoo.so")
#     # test_lib_copy1 = os.path.join(libs_dir, "libfoo_patchelf.so")
#     # test_lib_copy2 = os.path.join(libs_dir, "libfoo_arwen.so")

#     # shutil.copy2(test_lib, test_lib_copy1)
#     # shutil.copy2(test_lib, test_lib_copy2)

#     return {
#         "test_bin_patchelf": test_bin_patchelf,
#         "test_bin_arwen": test_bin_arwen,
#         # "test_bin": test_bin,
#         # "test_bin_copy1": test_bin_copy1,
#         # "test_bin_copy2": test_bin_copy2,
#         # "test_lib": test_lib,
#         # "test_lib_copy1": test_lib_copy1,
#         # "test_lib_copy2": test_lib_copy2,
#         # "test_dir": test_dir,
#         # "libs_dir": libs_dir
#     }


def run_command(cmd, check=True):
    """Run a command and return its output."""
    result = subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=check,
        universal_newlines=True,
    )
    return result.stdout.strip(), result.stderr.strip(), result.returncode


# def test_print_interpreter(test_files):
#     """Test --print-interpreter functionality."""
#     patchelf_out, _, _ = run_command(["patchelf", "--print-interpreter", test_files["test_bin_copy1"]])
#     arwen_out, _, _ = run_command(["arwen", "--print-interpreter", test_files["test_bin_copy2"]])

#     assert patchelf_out == arwen_out, "Interpreters don't match"


def test_set_interpreter(arwen, bin_for_arwen, bin_for_patchelf):
    """Test --set-interpreter functionality."""

    # Get original interpreter
    orig_interp, _, _ = run_command(
        ["patchelf", "--print-interpreter", bin_for_patchelf]
    )

    # Set a test interpreter
    test_interp = "/lib64/test-ld-linux.so"

    run_command(["patchelf", "--set-interpreter", test_interp, bin_for_patchelf])
    run_command([arwen, "elf", "set-interpreter", test_interp, bin_for_arwen])

    # Compare results
    patchelf_out, _, _ = run_command(
        ["patchelf", "--print-interpreter", bin_for_patchelf]
    )
    arwen_out, _, _ = run_command(["patchelf", "--print-interpreter", bin_for_arwen])

    assert patchelf_out == arwen_out, "Set interpreter doesn't match"


# def test_print_rpath(test_files):
#     """Test --print-rpath functionality."""
#     patchelf_out, _, _ = run_command(["patchelf", "--print-rpath", test_files["test_bin_copy1"]])
#     arwen_out, _, _ = run_command(["arwen", "--print-rpath", test_files["test_bin_copy2"]])

#     assert patchelf_out == arwen_out, "RPATHs don't match"


def test_set_rpath(bin_for_arwen, bin_for_patchelf):
    """Test --set-rpath functionality."""

    test_rpath = "/opt/test/lib:/usr/local/lib"

    run_command(["patchelf", "--set-rpath", test_rpath, bin_for_patchelf])
    run_command(["arwen", "elf", "set-rpath", test_rpath, bin_for_arwen])

    patchelf_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_arwen])

    # arwen_out, _, _ = run_command(["arwen", "--print-rpath", test_bin_patchelf])

    assert patchelf_out == arwen_out, "Set RPATH doesn't match"
    # assert patchelf_out == test_rpath, "Patchelf didn't set RPATH correctly"


def test_add_rpath(bin_for_arwen, bin_for_patchelf):
    """Test --add-rpath functionality."""
    # Get original rpath
    orig_rpath, _, _ = run_command(["patchelf", "--print-rpath", bin_for_patchelf])

    # Add a path
    add_path = "/opt/added/lib"

    run_command(["patchelf", "--add-rpath", add_path, bin_for_patchelf])
    run_command(["arwen", "elf", "add-rpath", add_path, bin_for_arwen])

    patchelf_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_arwen])

    assert patchelf_out == arwen_out, "Add RPATH doesn't match"
    # assert add_path in patchelf_out, "Patchelf didn't add RPATH correctly"


def test_remove_rpath(bin_for_arwen, bin_for_patchelf):
    """Test --remove-rpath functionality."""
    run_command(["patchelf", "--remove-rpath", bin_for_patchelf])
    run_command(["arwen", "elf", "remove-rpath", bin_for_arwen])

    # Get rpath after removal (might fail, so don't check=True)
    patchelf_out, _, _ = run_command(
        ["patchelf", "--print-rpath", bin_for_patchelf], check=True
    )
    arwen_out, _, _ = run_command(
        ["patchelf", "--print-rpath", bin_for_arwen], check=True
    )

    patchelf_out == arwen_out


# def test_print_soname(test_files):
#     """Test --print-soname functionality."""
#     # This might fail as the libraries may not have SONAME set initially
#     patchelf_out, patchelf_err, patchelf_code = run_command(
#         ["patchelf", "--print-soname", test_files["test_lib_copy1"]], check=False)
#     arwen_out, arwen_err, arwen_code = run_command(
#         ["arwen", "--print-soname", test_files["test_lib_copy2"]], check=False)

#     assert patchelf_code == arwen_code, "Different return codes for print SONAME"
#     if patchelf_code == 0:
#         assert patchelf_out == arwen_out, "SONAME doesn't match"


def test_set_soname(bin_for_arwen, bin_for_patchelf):
    """Test --set-soname functionality."""
    new_soname = "libfoo_patched.so.1"

    run_command(["patchelf", "--set-soname", new_soname, bin_for_patchelf])
    run_command(["arwen", "elf", "set-soname", new_soname, bin_for_arwen])

    patchelf_out, _, _ = run_command(["patchelf", "--print-soname", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-soname", bin_for_arwen])

    assert patchelf_out == arwen_out, "Set SONAME doesn't match"
    # assert patchelf_out == new_soname, "Patchelf didn't set SONAME correctly"


# def test_print_needed(test_files):
#     """Test --print-needed functionality."""
#     # Use main_with_deps which has multiple needed libraries
#     bin_with_deps = os.path.join(test_files["test_dir"], "main_with_deps")

#     patchelf_out, _, _ = run_command(["patchelf", "--print-needed", bin_with_deps])
#     arwen_out, _, _ = run_command(["arwen", "--print-needed", bin_with_deps])

#     # Sort the output to handle different ordering
#     patchelf_libs = sorted(patchelf_out.split('\n'))
#     arwen_libs = sorted(arwen_out.split('\n'))

#     assert patchelf_libs == arwen_libs, "NEEDED entries don't match"


def test_add_needed(bin_for_arwen, bin_for_patchelf):
    """Test --add-needed functionality."""
    new_needed = "libextra.so.1"

    run_command(["patchelf", "--add-needed", new_needed, bin_for_patchelf])
    run_command(["arwen", "elf", "add-needed", bin_for_arwen, new_needed])

    patchelf_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_arwen])

    # # Sort the output to handle different ordering
    # patchelf_libs = sorted(patchelf_out.split('\n'))
    # arwen_libs = sorted(arwen_out.split('\n'))

    assert patchelf_out == arwen_out, "NEEDED entries don't match after add"
    # assert new_needed in patchelf_libs, "Patchelf didn't add NEEDED correctly"


def test_remove_needed(bin_for_arwen, bin_for_patchelf):
    """Test --remove-needed functionality."""
    # Add a test needed library to the binary using patchelf
    run_command(["patchelf", "--add-needed", "libtestbar.so", bin_for_patchelf])
    run_command(["patchelf", "--add-needed", "libtestbar.so", bin_for_arwen])

    run_command(["patchelf", "--remove-needed", "libtestbar.so", bin_for_patchelf])
    run_command(["arwen", "elf", "remove-needed", bin_for_arwen, "libtestbar.so"])

    patchelf_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_arwen])

    assert patchelf_out == arwen_out, "NEEDED entries don't match after removal"


def test_replace_needed(bin_for_arwen, bin_for_patchelf):
    """Test --replace-needed functionality."""
    # Add a test needed library to the binary using patchelf
    run_command(["patchelf", "--add-needed", "libtestbar.so", bin_for_patchelf])
    run_command(["patchelf", "--add-needed", "libtestbar.so", bin_for_arwen])

    run_command(
        [
            "patchelf",
            "--replace-needed",
            "libtestbar.so",
            "libtestbar_new.so",
            bin_for_patchelf,
        ]
    )
    run_command(
        [
            "arwen",
            "elf",
            "replace-needed",
            bin_for_arwen,
            "libtestbar.so=libtestbar_new.so",
        ]
    )

    patchelf_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-needed", bin_for_arwen])

    # # Sort the output to handle different ordering
    # patchelf_libs = sorted(patchelf_out.split('\n'))
    # arwen_libs = sorted(arwen_out.split('\n'))

    assert patchelf_out == arwen_out, "NEEDED entries don't match after replacement"
    # assert "libbar_new.so" in patchelf_libs, "Patchelf didn't replace NEEDED correctly"
    # assert "libbar.so" not in patchelf_libs, "Patchelf didn't remove old NEEDED correctly"


def test_shrink_rpath(bin_for_arwen, bin_for_patchelf, tmp_files):
    """Test --shrink-rpath functionality."""

    # let's create a DT_NEEDED what exist only in tmp_files
    new_needed = "libextra.so.1"

    touch_needed = os.path.join(tmp_files, "lib_path", new_needed)

    run_command(["patchelf", "--add-needed", new_needed, bin_for_patchelf])
    run_command(["patchelf", "--add-needed", new_needed, bin_for_arwen])

    # Set complex rpath with multiple entries
    complex_rpath = f"/not/needed/path:/another/path:{touch_needed}"

    run_command(["patchelf", "--set-rpath", complex_rpath, bin_for_patchelf])
    run_command(["patchelf", "--set-rpath", complex_rpath, bin_for_arwen])

    # Now shrink the rpath
    # we expect that /not/needed/path and /another/path will be removed
    run_command(["patchelf", "--shrink-rpath", bin_for_patchelf])
    run_command(["arwen", "elf", "shrink-rpath", bin_for_arwen])

    patchelf_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_arwen])

    assert patchelf_out == arwen_out, "RPATH doesn't match after shrinking"


def test_force_rpath(bin_for_arwen, bin_for_patchelf):
    """Test --force-rpath functionality."""
    new_rpath = "/opt/test/lib"

    run_command(
        ["patchelf", "--force-rpath", "--set-rpath", new_rpath, bin_for_patchelf]
    )
    run_command(["arwen", "elf", "force-rpath", bin_for_arwen])
    run_command(["arwen", "elf", "set-rpath", new_rpath, bin_for_arwen])

    patchelf_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_patchelf])
    arwen_out, _, _ = run_command(["patchelf", "--print-rpath", bin_for_arwen])

    assert patchelf_out == arwen_out, "RPATH doesn't match after force-rpath"


def test_no_default_lib(bin_for_arwen, bin_for_patchelf):
    """Test --no-default-lib functionality."""

    run_command(["patchelf", "--no-default-lib", bin_for_patchelf])
    run_command(["arwen", "elf", "no-default-lib", bin_for_arwen])

    # reading readelf output to check if the flag is set
    patchelf_out = subprocess.run(
        ["greadelf", "-d", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "-d", bin_for_arwen], capture_output=True, text=True
    )

    assert "(FLAGS_1)            Flags: NODEFLIB" in patchelf_out.stdout, (
        "no-default-lib flag not set"
    )
    assert "(FLAGS_1)            Flags: NODEFLIB" in arwen_out.stdout, (
        "no-default-lib flag not set"
    )


def test_clear_symbol_versions(bin_for_arwen, bin_for_patchelf):
    """Test --clear-symbol-versions functionality."""
    # we now that linux-x64-bash has symbol versions for chdir@GLIBC_2.2.5
    # assert that they exist before clearing

    patchelf_out = subprocess.run(
        ["greadelf", "--syms", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "--syms", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" in patchelf_out.stdout, "symbol versions not found"
    assert "chdir@GLIBC_2.2.5" in arwen_out.stdout, "symbol versions not found"

    run_command(["patchelf", "--clear-symbol-version", "chdir", bin_for_patchelf])
    run_command(["arwen", "elf", "clear-symbol-version", "chdir", bin_for_arwen])

    # reading readelf output to check if the symbol version was removed
    patchelf_out = subprocess.run(
        ["greadelf", "--syms", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "--syms", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" not in patchelf_out.stdout, (
        "symbol versions werent removed"
    )
    assert "chdir@GLIBC_2.2.5" not in arwen_out.stdout, "symbol versions werent removed"

    assert "chdir" in patchelf_out.stdout, "symbol versions werent removed"
    assert "chdir" in arwen_out.stdout, "symbol versions werent removed"


def test_add_debug_tag(bin_for_arwen, bin_for_patchelf):
    """Test --add-debug-tag functionality."""
    # we now that linux-x64-bash has symbol versions for chdir@GLIBC_2.2.5
    # assert that they exist before clearing

    patchelf_out = subprocess.run(
        ["greadelf", "--syms", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "--syms", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" in patchelf_out.stdout, "symbol versions not found"
    assert "chdir@GLIBC_2.2.5" in arwen_out.stdout, "symbol versions not found"

    run_command(["patchelf", "--clear-symbol-version", "chdir", bin_for_patchelf])
    run_command(["arwen", "elf", "clear-symbol-version", "chdir", bin_for_arwen])

    # reading readelf output to check if the symbol version was removed
    patchelf_out = subprocess.run(
        ["greadelf", "--syms", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "--syms", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" not in patchelf_out.stdout, (
        "symbol versions werent removed"
    )
    assert "chdir@GLIBC_2.2.5" not in arwen_out.stdout, "symbol versions werent removed"

    assert "chdir" in patchelf_out.stdout, "symbol versions werent removed"
    assert "chdir" in arwen_out.stdout, "symbol versions werent removed"


def test_clear_execstack(bin_for_arwen, bin_for_patchelf):
    """Test --clear-execstack functionality."""

    run_command(["patchelf", "--clear-execstack", bin_for_patchelf])
    run_command(["arwen", "elf", "clear-exec-stack", bin_for_arwen])

    # reading readelf output to check if the symbol version was removed
    patchelf_out = subprocess.run(
        ["patchelf", "--print-execstack", bin_for_patchelf],
        capture_output=True,
        text=True,
    )
    arwen_out = subprocess.run(
        ["patchelf", "--print-execstack", bin_for_arwen], capture_output=True, text=True
    )

    assert patchelf_out.stdout == arwen_out.stdout, "execstack flag not removed"


def test_set_execstack(bin_for_arwen, bin_for_patchelf):
    """Test --set-execstack functionality."""

    run_command(["patchelf", "--set-execstack", bin_for_patchelf])
    run_command(["arwen", "elf", "set-exec-stack", bin_for_arwen])

    # reading readelf output to check if the symbol version was removed
    patchelf_out = subprocess.run(
        ["patchelf", "--print-execstack", bin_for_patchelf],
        capture_output=True,
        text=True,
    )
    arwen_out = subprocess.run(
        ["patchelf", "--print-execstack", bin_for_arwen], capture_output=True, text=True
    )

    assert patchelf_out.stdout == arwen_out.stdout, "execstack flag not set"


def test_rename_dynamic_symbols(bin_for_arwen, bin_for_patchelf, tmp_files):
    """Test --rename-dynamic-symbols functionality."""
    patchelf_out = subprocess.run(
        ["greadelf", "--syms", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["greadelf", "--syms", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" in patchelf_out.stdout, "symbol versions not found"
    assert "chdir@GLIBC_2.2.5" in arwen_out.stdout, "symbol versions not found"

    # write a temporary file to test the rename
    test_renmap = tmp_files / "renmap.txt"
    test_renmap.write_text("chdir chdir_new")

    run_command(["patchelf", "--rename-dynamic-symbols", test_renmap, bin_for_patchelf])
    run_command(
        ["arwen", "elf", "rename-dynamic-symbols", bin_for_arwen, "chdir=chdir_new"]
    )

    # reading readelf output to check if the symbol was renamed
    patchelf_out = subprocess.run(
        ["nm", "-D", bin_for_patchelf], capture_output=True, text=True
    )
    arwen_out = subprocess.run(
        ["nm", "-D", bin_for_arwen], capture_output=True, text=True
    )

    assert "chdir@GLIBC_2.2.5" not in patchelf_out.stdout, (
        "symbol versions werent renamed"
    )
    assert "chdir@GLIBC_2.2.5" not in arwen_out.stdout, "symbol versions werent renamed"

    assert "chdir_new@GLIBC_2.2.5" in patchelf_out.stdout, (
        "symbol versions werent renamed"
    )
    assert "chdir_new@GLIBC_2.2.5" in arwen_out.stdout, "symbol versions werent renamed"

"""
ELF binary manipulation module for Linux/Unix systems.

This module provides functionality for working with ELF binary files,
allowing operations like modifying runpaths, changing interpreters, and more.
"""

from typing import Optional, Dict, List


from arwen.arwen import PyElfContainer


class ElfContainer:
    """
    Python wrapper for manipulating ELF binary files.

    This class provides methods to modify various aspects of ELF binaries,
    such as runpaths, needed libraries, interpreter paths, and more.
    """

    _inner: PyElfContainer

    @staticmethod
    def from_path(path: str) -> "ElfContainer":
        """
        Create a new ElfContainer by parsing an ELF binary file.

        Args:
            path: Path to the ELF binary file

        Returns:
            A new ElfContainer instance

        Raises:
            ElfError: If the file cannot be parsed as an ELF binary
            Exception: If the file cannot be read
        """
        return ElfContainer._from_py(PyElfContainer.from_path(path))

    def add_runpath(self, new_runpath: str) -> None:
        """
        Add a new runpath to the ELF file.

        Args:
            new_runpath: The runpath to add

        Raises:
            ElfError: If the runpath cannot be added
        """
        self._inner.add_runpath(new_runpath)

    def set_runpath(self, set_runpath: str) -> None:
        """
        Set runpath in the ELF file.

        Args:
            set_runpath: The runpath to set

        Raises:
            ElfError: If the runpath cannot be set
        """
        self._inner.set_runpath(set_runpath)

    def remove_runpath(self) -> None:
        """
        Remove any existing runpath from the ELF file.

        Raises:
            ElfError: If the runpath cannot be removed
        """
        self._inner.remove_runpath()

    def get_runpath(self) -> Optional[str]:
        """
        Get the current runpath from the ELF file.

        Returns:
            The current runpath, or None if no runpath is set
        """
        return self._inner.get_runpath()

    def force_rpath(self) -> None:
        """
        Force the ELF file to use DT_RPATH instead of DT_RUNPATH.

        Raises:
            ElfError: If the operation cannot be performed
        """
        self._inner.force_rpath()

    def shrink_rpath(self, rpath_prefixes: List[str]) -> None:
        """
        Shrink rpath to only include specified prefixes.

        Args:
            rpath_prefixes: List of prefixes to include in the rpath

        Raises:
            ElfError: If the rpath cannot be shrunk
        """
        self._inner.shrink_rpath(rpath_prefixes)

    def add_needed(self, needed_libs: List[str]) -> None:
        """
        Add needed libraries to the ELF file.

        Args:
            needed_libs: List of needed libraries to add

        Raises:
            ElfError: If the needed libraries cannot be added
        """
        self._inner.add_needed(needed_libs)

    def remove_needed(self, needed_libs: List[str]) -> None:
        """
        Remove needed libraries from the ELF file.

        Args:
            needed_libs: List of needed libraries to remove

        Raises:
            ElfError: If the needed libraries cannot be removed
        """
        self._inner.remove_needed(needed_libs)

    def replace_needed(self, needed_mappings: Dict[str, str]) -> None:
        """
        Replace needed libraries in the ELF file.

        Args:
            needed_mappings: Dictionary mapping old library names to new library names

        Raises:
            ElfError: If the needed libraries cannot be replaced
        """
        self._inner.replace_needed(needed_mappings)

    def get_needed(self) -> List[str]:
        """
        Get the list of needed libraries from the ELF file.

        Returns:
            List of needed libraries
        """
        return self._inner.get_needed()

    def set_interpreter(self, new_interpreter: str) -> None:
        """
        Change the interpreter path in the ELF file.

        Args:
            new_interpreter: The new interpreter path

        Raises:
            ElfError: If the interpreter cannot be changed
        """
        self._inner.set_interpreter(new_interpreter)

    def get_interpreter(self) -> Optional[str]:
        """
        Get the current interpreter from the ELF file.

        Returns:
            The current interpreter, or None if no interpreter is set
        """
        return self._inner.get_interpreter()

    def set_os_abi(self, os_abi: str) -> None:
        """
        Set the OS ABI in the ELF file.

        Args:
            os_abi: The OS ABI to set

        Raises:
            ElfError: If the OS ABI cannot be set
        """
        self._inner.set_os_abi(os_abi)

    def get_os_abi(self) -> int:
        """
        Get the OS ABI from the ELF file.

        Returns:
            The OS ABI
        """
        return self._inner.get_os_abi()

    def set_soname(self, soname: str) -> None:
        """
        Set the SONAME of the ELF file.

        Args:
            soname: The SONAME to set

        Raises:
            ElfError: If the SONAME cannot be set
        """
        self._inner.set_soname(soname)

    def get_soname(self) -> Optional[str]:
        """
        Get the SONAME of the ELF file.

        Returns:
            The SONAME, or None if no SONAME is set
        """
        return self._inner.get_soname()

    def no_default_lib(self) -> None:
        """
        Disable the default library search paths.

        Raises:
            ElfError: If the operation cannot be performed
        """
        self._inner.no_default_lib()

    def clear_version_symbol(self, symbol_name: str) -> None:
        """
        Clear a specific symbol version in the ELF file.

        Args:
            symbol_name: The name of the symbol whose version to clear

        Raises:
            ElfError: If the version cannot be cleared
        """
        self._inner.clear_version_symbol(symbol_name)

    def add_debug_tag(self) -> None:
        """
        Add a debug tag to the ELF file.

        Raises:
            ElfError: If the debug tag cannot be added
        """
        self._inner.add_debug_tag()

    def clear_exec_stack(self) -> None:
        """
        Clear the executable stack flag in the ELF file.

        Raises:
            ElfError: If the executable stack flag cannot be cleared
        """
        self._inner.clear_exec_stack()

    def set_exec_stack(self) -> None:
        """
        Set the executable stack flag in the ELF file.

        Raises:
            ElfError: If the executable stack flag cannot be set
        """
        self._inner.set_exec_stack()

    def is_exec_stack(self) -> Optional[bool]:
        """
        Get the executable stack status.

        Returns:
            True if the executable stack flag is set, False otherwise, or None if unknown
        """
        return self._inner.is_exec_stack()

    def rename_dynamic_symbols(self, symbols_map: Dict[str, str]) -> None:
        """
        Rename dynamic symbols in the ELF file.

        Args:
            symbols_map: Dictionary mapping old symbol names to new symbol names

        Raises:
            ElfError: If the symbols cannot be renamed
        """
        self._inner.rename_dynamic_symbols(symbols_map)

    def save(self, path: str) -> None:
        """
        Save the modified ELF file to a path.

        Args:
            path: Path where the modified file will be saved

        Raises:
            Exception: If the file cannot be written
        """
        self._inner.save(path)

    @classmethod
    def _from_py(cls, py_elf: PyElfContainer) -> "ElfContainer":
        elf_container = cls.__new__(cls)
        elf_container._inner = py_elf

        return elf_container

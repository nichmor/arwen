"""
Mach-O binary manipulation module for macOS.

This module provides functionality for working with Mach-O binary files,
allowing operations like modifying rpaths, changing install names, and more.
"""

from arwen.arwen import PyMachoContainer


class MachoContainer:
    """
    Python wrapper for manipulating Mach-O binary files.

    This class provides methods to modify various aspects of Mach-O binaries,
    such as rpaths, install names, and install IDs.
    """

    _inner: PyMachoContainer

    @staticmethod
    def from_path(path: str) -> "MachoContainer":
        """
        Create a new MachoContainer by parsing a Mach-O binary file.

        Args:
            path: Path to the Mach-O binary file

        Returns:
            A new MachoContainer instance

        Raises:
            MachoError: If the file cannot be parsed as a Mach-O binary
            Exception: If the file cannot be read
        """
        return MachoContainer._from_py(PyMachoContainer.from_path(path))

    def add_rpath(self, new_rpath: str) -> None:
        """
        Add a new rpath to the Mach-O file.

        Args:
            new_rpath: The rpath to add

        Raises:
            MachoError: If the rpath cannot be added
        """
        self._inner.add_rpath(new_rpath)

    def change_rpath(self, old_rpath: str, new_rpath: str) -> None:
        """
        Change an existing rpath in the Mach-O file.

        Args:
            old_rpath: The existing rpath to change
            new_rpath: The new rpath value

        Raises:
            RpathMissingError: If the old_rpath doesn't exist
            MachoError: If the rpath cannot be changed
        """
        self._inner.change_rpath(old_rpath, new_rpath)

    def remove_rpath(self, old_rpath: str) -> None:
        """
        Remove an existing rpath from the Mach-O file.

        Args:
            old_rpath: The rpath to remove

        Raises:
            RpathMissingError: If the specified rpath doesn't exist
            MachoError: If the rpath cannot be removed
        """
        self._inner.remove_rpath(old_rpath)

    def change_install_id(self, new_id: str) -> None:
        """
        Change the install ID of the Mach-O file (for shared libraries).

        Args:
            new_id: The new install ID

        Raises:
            DylibIdMissingError: If the file is not a shared library
            MachoError: If the install ID cannot be changed
        """
        self._inner.change_install_id(new_id)

    def change_install_name(self, old_name: str, new_name: str) -> None:
        """
        Change the install name of a dependency in the Mach-O file.

        Args:
            old_name: The existing install name to change
            new_name: The new install name

        Raises:
            DylibNameMissingError: If the old_name doesn't exist
            MachoError: If the install name cannot be changed
        """
        self._inner.change_install_name(old_name, new_name)

    def save(self, path: str) -> None:
        """
        Save the modified Mach-O file to a path.

        Args:
            path: Path where the modified file will be saved

        Raises:
            Exception: If the file cannot be written
        """
        self._inner.save(path)

    @classmethod
    def _from_py(cls, py_macho: PyMachoContainer) -> "MachoContainer":
        macho_container = cls.__new__(cls)
        macho_container._inner = py_macho

        return macho_container

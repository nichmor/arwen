## Why Patch Executables and Libraries?

Modern operating systems rely heavily on **dynamic linking**.

Instead of embedding the code for every function an application needs directly into the executable file (which is called **static linking** and results in very large files), dynamic linking allows executables to use shared libraries (`.so` files on Linux/ELF, `.dylib` files on macOS/Mach-O) that are loaded into memory only when the program runs.

This saves disk space and memory, and allows libraries to be updated independently of the applications using them.

!!! note
    About Linking Types:

    **Static Linking**: Copies all necessary library code directly into the final executable during compilation. This creates larger, self-contained files. If a library needs an update, any application using it must be recompiled.

    **Dynamic Linking**: The executable only stores references (like names or paths) to external shared library files. The operating system's dynamic linker loads these libraries at runtime. This leads to smaller executables, allows libraries to be shared in memory by multiple applications, and enables independent library updates (though this can sometimes lead to versioning issues).



### The Problem: Finding Libraries at Runtime

When you run an executable that uses dynamic linking, the operating system's **dynamic linker** (or loader) has a crucial job: it must find and load all the required shared libraries mentioned by the executable. How does it find them? It typically searches in a specific order, which might include:

1.  Paths explicitly encoded within the executable itself (**RPATH**, **RUNPATH** for ELF; **Install Names**, **RPATH** using `@rpath` for Mach-O).
2.  Paths specified by environment variables (e.g., `LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS - often discouraged for general use).
3.  Default system library directories (e.g., `/lib`, `/usr/lib`, `/usr/local/lib`).


The challenge arises because the assumptions made during compilation about where libraries will be located might not hold true when the application is actually deployed or run. Hardcoded absolute paths might be wrong, default system paths might not contain the right library versions, and relative paths need careful management.


### A small practical example for ELF file

Let's say you run an executable program located at `/usr/bin/my_app`. This program requires a shared library called `libdata.so.1`.

When you execute `/usr/bin/my_app`, the Linux kernel loads the `my_app` executable into memory. It sees from the executable's header that it needs a dynamic linker, typically `/lib64/ld-linux-x86-64.so.2` (on a 64-bit system). The kernel loads this dynamic linker and passes control to it.

Dynamic linker (our `ld-linux.so.2`) inspects `my_app`'s internal structure (specifically the `.dynamic` section) and finds an entry indicating it needs `libdata.so.1`.

It starts searching for the file `libdata.so.1` in a specific order (this is simplified list):

- `DT_RUNPATH`: It checks if `my_app` has a `DT_RUNPATH` entry.
  Let's imagine `my_app` was patched to have `DT_RUNPATH=$ORIGIN/../lib`. `$ORIGIN` means "the directory containing the executable", so the linker looks for `/usr/bin/../lib/libdata.so.1` (which resolves to `/usr/lib/libdata.so.1`). If it exists, the linker proceeds to step 4.

- `LD_LIBRARY_PATH` Check: If the library wasn't found via RUNPATH (or if RUNPATH doesn't exist), the linker checks the `LD_LIBRARY_PATH` environment variable. If you had run the app like `LD_LIBRARY_PATH=/opt/custom_libs /usr/bin/my_app`, the linker would look for `/opt/custom_libs/libdata.so.1`. If found here, it uses this one and skips the `RPATH` check below.

- `DT_RPATH`: If not found yet, and if my_app has an `RPATH` entry (which is different from `RUNPATH`), the linker checks there.
For example, if `RPATH=/usr/local/special_libs`, it looks for `/usr/local/special_libs/libdata.so.1`.

- `System Cache/Paths`: If still not found, the linker consults the system library cache (usually `/etc/ld.so.cache`, built from `/etc/ld.so.conf`) and then checks standard default paths like `/lib, /usr/lib, /lib64, /usr/lib64`. It looks for `/lib/libdata.so.1, /usr/lib/libdata.so.1`, etc.

- `Loading`: This is the last part. Once `libdata.so.1` is found the dynamic linker maps its code and data segments into the memory space of the `my_app` process.


**This is where patching becomes essential.** Tools like `arwen` allow you to modify the information embedded within the executable *after* it has been compiled and linked, telling the dynamic linker where to find the necessary libraries in the *actual* runtime environment.

**Common Scenarios Requiring Patching:**

Here are some frequent situations where you'll need to patch executables or libraries using a tool like `arwen`:

- **Relocation / Non-Standard Installs:**
    Software is installed in a non-standard location (e.g., `/opt/myapp`, `/home/user/bin`) instead of system paths like `/usr/bin`. Binaries might have been compiled assuming standard library locations.
    Patch the binaries to add the application's own library directory (e.g., `/opt/myapp/lib`) to their runtime search path (`RPATH`/`RUNPATH` or Mach-O `LC_RPATH`).

- **Cross-Compilation:**
    When building software for a different architecture or operating system, the embedded paths might reflect the host system's layout, not the target's. Patch the resulting binaries to use paths appropriate for the target system.

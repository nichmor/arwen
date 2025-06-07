## Practical CLI Tutorial: Patching an ELF Binary

This tutorial demonstrates a common use case for `arwen`: patching an ELF executable so it can find its required shared library (`.so` file) using a relative path. This makes the application "relocatable," meaning you can install it in different locations without breaking its ability to find its dependencies.

**Goal:**

To modify an application (`my_app`) that depends on a library (`libcustom.so`) so that `my_app` can find `libcustom.so` when they are placed in a specific directory structure (e.g., `bin/` and `lib/` subdirectories within a main installation folder).

**Scenario:**

1. We have a simple ELF executable named `my_app`.
2. `my_app` depends on a custom ELF shared library named `libcustom.so`.
3. Initially, `my_app` expects `libcustom.so` to be in a standard system library path (like `/usr/lib`) or a path listed in `LD_LIBRARY_PATH`.
4. We want to install the application into `/opt/my_app/`, placing the executable at `/opt/my_app/bin/my_app` and the library at `/opt/my_app/lib/libcustom.so`.
5. We will use `arwen` to patch `/opt/my_app/bin/my_app` so it automatically looks for `libcustom.so` in the adjacent `../lib` directory.

**Prerequisites:**

* The `arwen` command-line tool installed and in your PATH.
* A simple ELF executable (`my_app`) and an ELF shared library (`libcustom.so`) it depends on.
  * *Note: For experimentation, you can often use simple existing tools or create minimal examples if you have a C compiler like GCC:*
    * `libcustom.c`: `int custom_function() { return 42; }`
    * `my_app.c`: `int custom_function(); int main() { return custom_function(); }`
    * Compile:
      * `gcc -shared -fPIC -o libcustom.so libcustom.c`
      * `gcc my_app.c -o my_app -L. -lcustom` (Link against the library in the current dir)

**Step 1: Inspecting the Initial State**

First, let's see how `my_app` currently finds (or fails to find) its dependency and what its embedded paths look like. Assume `my_app` and `libcustom.so` are in your current directory for now.

```bash
# Check dynamic dependencies using the standard ldd tool
# note that this command is not available on macos
ldd ./my_app
```

You'll likely see output similar to this, indicating libcustom.so is needed but perhaps not found yet in standard locations:
```
        linux-vdso.so.1 (0x...)
        libcustom.so => not found  # <--- The problem! Or it might point to /usr/lib if installed there
        libc.so.6 (0x...) => /lib/x86_64-linux-gnu/libc.so.6 (...)
        /lib64/ld-linux-x86-64.so.2 (0x...)
```

Or you can use `arwen` to verify the `rpaths` the ELF file directly:

```bash
arwen elf print-rpath my_app
```

You can see that we have no `RPATH` set, which means it will not look in any custom directories for `libcustom.so`.

**Step 2: Patching the ELF Binary**
Now, we will patch `my_app` to add a relative `RPATH` that points to the `lib/` directory where `libcustom.so` will be located.

```bash
arwen elf set-rpath my_app ../lib
```

**Step 2: Patching the ELF Binary**
Now, we will patch `my_app` to add a relative `RPATH` that points to the `lib/` directory where `libcustom.so` will be located.

```bash
arwen elf set-rpath my_app ../lib
```

If we run `arwen elf print-rpath my_app` again, we should see the new `RPATH` set to `../lib`.
```bash
arwen elf print-rpath my_app
```

```
../lib
```

**Step 3: Removing the DT_NEEDED**
Sometimes you might want to remove the `DT_NEEDED` entry for a library, for example when stripping dependencies.

```bash
arwen elf remove-needed my_app libcustom.so
```

Now, if we check the dependencies again with `ldd` or `arwen`, we should see that `libcustom.so` is no longer listed as a required dependency.

```bash
ldd ./my_app
```
or
```bash
arwen elf print-needed my_app
```

**Conclusion:**
In this small tutorial, we took a look in a practical example how to use `arwen` for patching binaries and setup a different `RPATH` for an ELF binary. This allows the application to find its dependencies in a custom directory structure, making it more portable and easier to deploy.

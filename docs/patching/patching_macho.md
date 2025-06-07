## Practical CLI Tutorial: Patching a Mach-O Binary

This tutorial demonstrates a common use case for `arwen`: patching a Mach-O executable so it can find its required shared library (`.dylib` file) using a relative path. This makes the application "relocatable," meaning you can install it in different locations without breaking its ability to find its dependencies. This is typically achieved using `@rpath`.

**Goal:**

To modify an application (`my_app`) that depends on a library (`libcustom.dylib`) so that `my_app` can find `libcustom.dylib` when they are placed in a specific directory structure (e.g., `bin/` and `lib/` subdirectories within a main installation folder).

**Scenario:**

1. We have a simple Mach-O executable named `my_app`.
2. `my_app` depends on a custom Mach-O shared library named `libcustom.dylib`.
3. Initially, `my_app` is compiled to look for `libcustom.dylib` in a path specified by `@rpath`, but the application itself does not yet have an `rpath` configured.
4. We want to install the application into `/opt/my_app/`, placing the executable at `/opt/my_app/bin/my_app` and the library at `/opt/my_app/lib/libcustom.dylib`.
5. We will use `arwen` to patch `/opt/my_app/bin/my_app` to add an `rpath` that tells it to look for `libcustom.dylib` in the adjacent `../lib` directory.

**Prerequisites:**

* The `arwen` command-line tool installed and in your PATH.
* A simple Mach-O executable (`my_app`) and a Mach-O shared library (`libcustom.dylib`) it depends on.
  * *Note: You can create these with a C compiler like `clang` (standard on macOS):*
    * `libcustom.c`: `int custom_function() { return 42; }`
    * `my_app.c`: `int custom_function(); int main() { return custom_function(); }`
    * Compile:
      * `clang -dynamiclib -o libcustom.dylib libcustom.c -install_name "@rpath/libcustom.dylib"`
      * `clang my_app.c -o my_app -L. -lcustom` (Link against the library in the current dir)

**Step 1: Inspecting the Initial State**

First, let's see how `my_app` currently finds its dependency and what its embedded paths look like. Assume `my_app` and `libcustom.dylib` are in your current directory.

```bash
# Check dynamic dependencies using the standard otool tool on macOS
otool -L ./my_app
```

You'll see output similar to this. Note the `@rpath/libcustom.dylib` entry. This tells the loader to search for the library in the runtime search paths (rpaths).
```bash
./my_app:
	@rpath/libcustom.dylib (compatibility version 0.0.0, current version 0.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1319.100.3)
```


You will see no output, as we have not set any `rpath` yet. This means the application won't find `libcustom.dylib` if you try to run it.

**Step 2: Patching the Mach-O Binary**
Now, we will patch `my_app` to add a relative `rpath` that points to where `libcustom.dylib` will be located. We use `@executable_path` which is a special variable that resolves to the directory containing the binary.

```bash
arwen macho add-rpath my_app @executable_path/../lib
```

If we run `arwen macho print-rpaths my_app` again, we should see the new `rpath`:
```bash
arwen macho print-rpaths my_app
```

```
@executable_path/../lib
```
Now, if you were to place `my_app` in a `bin/` directory and `libcustom.dylib` in a sibling `lib/` directory, the executable would successfully find its library at runtime.

**Step 3: Removing a Dependency**
Sometimes you might want to remove a dependency entry from the binary, for example when stripping dependencies.

```bash
arwen macho remove-dependency my_app libcustom.dylib
```

Now, if we check the dependencies again with `otool`, we should see that `libcustom.dylib` is no longer listed as a required dependency.

```bash
otool -L ./my_app
```
or
```bash
arwen macho print-dependencies my_app
```

**Conclusion:**
In this small tutorial, we took a look in a practical example how to use `arwen` for patching Mach-O binaries and setup a different `rpath`. This allows the application to find its dependencies in a custom directory structure using `@executable_path`, making it more portable and easier to deploy on macOS.

## ELF (Executable and Linkable Format)
`ELF` is the standard binary format used on Linux, BSD variants, Solaris, and many other Unix-like operating systems for `executables`, `shared libraries (.so files)`, `object code`, and `core dumps`.

- **Executables**: These are files containing machine code instructions that the operating system can load directly into memory and run as a program. They are the result of compiling and linking source code. On Linux, common executables like `/bin/bash` or `/usr/bin/grep` are typically in ELF format.

- **Shared Libraries (.so files)**: Standing for `"Shared Object"`, these files contain compiled code and data (functions, variables) designed to be used by multiple executable programs at the same time while they are running `(this is dynamic linking)`. Instead of each program having its own copy of the library code, they share a single copy loaded in memory. Examples include `libc.so.6` (the standard C library) or `libssl.so` (for SSL/TLS functions).

- **Object Code (.o files)**: This is the intermediate output produced by a compiler when it translates source code (like `.c` or `.rs` files) into machine code for a specific architecture. Object files aren't runnable on their own. They contain the compiled code and data but also information needed by the linker to combine them with other object files and libraries to create a final executable or shared library.

- **Core Dumps**: This is a file created by the operating system when a program crashes or terminates unexpectedly. It contains a snapshot of the program's state at the time of the crash, including its memory contents (the "core") and often CPU register values. Developers use core dump files with debuggers (like `gdb`) to perform post-mortem analysis and figure out why the program failed.

## ELF File Structure Overview


- **ELF Header**: Always located at the very beginning of the file (offset 0).

- **Program Header Table (PHT)**: Optional, but present in executables and shared libraries. Describes segments used for loading the file into memory. Its location and size are specified in the ELF Header.

- **Sections**: Contain the actual code, data, symbol tables, string tables, and linking information. Their locations and sizes are described by the `Section Header Table`.

- **Section Header Table (SHT)**: Present except sometimes in stripped executables. Describes the file's sections. Its location and size are specified in the ELF Header, often placed near the end of the file.

A simple ELF file structure looks like this:

```
+---------------------+
| ELF Header          |  (File Offset 0)
+---------------------+
| Program Header Table|  (Offset specified in ELF Header)
| (Optional)          |
+---------------------+
|                     |
| Segment 1 / Section |  (e.g., Code .text)
|                     |
+---------------------+
|                     |
| Segment 2 / Section |  (e.g., Data .data, .bss)
|                     |
+---------------------+
| ...                 |  (Other segments/sections like .dynamic)
+---------------------+
|                     |
| Section Header Table|  (Offset specified in ELF Header, often near end)
| (Optional)          |
+---------------------+
```

!!! note
    Sections are often contained within Segments.

    The PHT describes the file from a loading/memory perspective (Segments), while the SHT describes it from a linking/content perspective (Sections).


### More detailed view of the components

- **`1. ELF Header (Elf64_Ehdr / Elf32_Ehdr)`**
  - **Location:** Start of the file (offset 0).
  - **Purpose:** Identifies the file as ELF and provides essential metadata and pointers.
  - **Key Fields:**
    - `e_ident`: Magic number (`\x7fELF`) and other info (class 32/64-bit, data encoding, ABI version).
    - `e_type`: File type (e.g., `ET_EXEC` for executable, `ET_DYN` for shared library/position-independent executable, `ET_REL` for relocatable object file).
    - `e_machine`: Target architecture (e.g., `EM_X86_64`, `EM_AARCH64`).
    - `e_version`: ELF version (usually 1).
    - `e_entry`: Virtual memory address of the program's entry point (where execution begins).
    - `e_phoff`: File offset to the start of the Program Header Table.
    - `e_shoff`: File offset to the start of the Section Header Table.
    - `e_flags`: Processor-specific flags.
    - `e_ehsize`: Size of this ELF header.
    - `e_phentsize`: Size of a single entry in the Program Header Table.
    - `e_phnum`: Number of entries in the Program Header Table.
    - `e_shentsize`: Size of a single entry in the Section Header Table.
    - `e_shnum`: Number of entries in the Section Header Table.
    - `e_shstrndx`: Section header table index of the section containing section names.


- **2. Program Header Table (PHT)**
  -  **Location:** At the file offset specified by `e_phoff` in the ELF Header. It's an array of `e_phnum` entries, each `e_phentsize` bytes long.
  - **Purpose:** Describes *segments* – contiguous chunks of the file that need to be mapped into memory by the system loader when creating a process image. This is the "execution view" of the file.
  -  **`Key Entry Types (p_type):`**
     - `PT_LOAD`: Describes a loadable segment (e.g., code, data). Specifies file offset (`p_offset`), virtual address (`p_vaddr`), physical address (`p_paddr` - often ignored), file size (`p_filesz`), memory size (`p_memsz` - can be larger for `.bss`), and permissions (`p_flags` - Read/Write/Execute).
     - `PT_DYNAMIC`: Points to the segment containing dynamic linking information (the `.dynamic` section). Specifies offset (`p_offset`) and size (`p_filesz`). Essential for executables/libraries using dynamic linking.
     - `PT_INTERP`: Points to a null-terminated string specifying the path of the program interpreter (dynamic linker, e.g., `/lib64/ld-linux-x86-64.so.2`).
     - Other types exist for notes (`PT_NOTE`), TLS (`PT_TLS`), etc.

- **3. Sections**
  - **Location:** Scattered throughout the file, as defined by the Section Header Table.
  - **Purpose:** Hold the actual content: compiled code, data, symbol tables, string tables, relocation information, dynamic linking structures, debugging info, etc. Sections represent the "linking view" of the file.
  - **Common Sections:**
  - `.text`: Executable code.
    - `.data`: Initialized data (global/static variables with initial values).
    - `.bss`: Uninitialized data (global/static variables without explicit initial values; occupies no file space but reserves memory space).
    - `.rodata`: Read-only data (constants, string literals).
    - `.symtab`: Symbol table (for linking/debugging).
    - `.strtab`: String table for `.symtab`.
    - `.shstrtab`: String table for section names themselves.
    - `.dynamic`: Holds the array of dynamic linking tags (see below).
    - `.dynsym`: Minimal symbol table needed for dynamic linking.
    - `.dynstr`: String table for `.dynsym` and `.dynamic` entries requiring strings (like library names in `DT_NEEDED`).
    - `.interp`: Contains the path string for the program interpreter (pointed to by `PT_INTERP`).


- **4. Section Header Table (SHT)**
  - **Location:** At the file offset specified by `e_shoff` in the ELF Header. An array of `e_shnum` entries, each `e_shentsize` bytes long.
  - **Purpose:** Describes each section in the file. Essential for linkers and debuggers, less so for the runtime loader (which uses the PHT).
  - **`Key Fields in an Entry (Elf64_Shdr / Elf32_Shdr):`**
    - `sh_name`: Offset into the `.shstrtab` section giving the section's name.
    - `sh_type`: Section type (e.g., `SHT_PROGBITS` for code/data, `SHT_SYMTAB` for symbols, `SHT_STRTAB` for strings, `SHT_NOBITS` for `.bss`, `SHT_DYNAMIC` for dynamic tags).
    - `sh_flags`: Attributes like `SHF_WRITE`, `SHF_ALLOC` (occupies memory), `SHF_EXECINSTR` (executable code).
    - `sh_addr`: Virtual memory address if the section is loaded.
    - `sh_offset`: File offset of the section's start.
    - `sh_size`: Size of the section in the file.
    - `sh_link`, `sh_info`: Interpretation depends on section type (e.g., for `.dynamic`, `sh_link` points to the string table `.dynstr`).
    - `sh_addralign`: Required alignment.
    - `sh_entsize`: Size of entries if the section holds a table (like symbol table).


## Dynamic Linking
For dynamic linking, the crucial parts are:

1.  The `PT_INTERP` program header entry (and the `.interp` section it points to) tells the kernel which dynamic linker to execute.
2.  The `PT_DYNAMIC` program header entry points to the segment containing the `.dynamic` section.
3.  The **`.dynamic`** section contains the array of tags and values that drive the dynamic linker. `arwen` modifies values associated with specific tags within this section (or the string table `.dynstr` they point to):
    * **`DT_INTERP`**: (Tag only, value points into `.dynstr`) Path to the dynamic linker. Modifying the string in `.dynstr` changes the interpreter.
    * **`DT_NEEDED`**: (Value points into `.dynstr`) Name of a required library. Modifying the string in `.dynstr` changes the dependency. `arwen` can also add/remove entries in the `.dynamic` array itself.
    * **`DT_RPATH`** / **`DT_RUNPATH`**: (Value points into `.dynstr`) Library search paths. Modifying the string in `.dynstr` changes these paths. Remember `$ORIGIN` is expanded by the linker to the directory of the object being processed.

**How the Dynamic Linker Uses This Information (Simplified version)**

The dynamic linker (`ld-linux.so.2` or similar), specified by `PT_INTERP` / `DT_INTERP`, reads the `.dynamic` section (found via `PT_DYNAMIC`). It processes `DT_NEEDED` entries to find required libraries, searching in paths derived from `DT_RUNPATH`, `LD_LIBRARY_PATH`, `DT_RPATH`, and system defaults, then loads them into memory (using their own ELF structures) and resolves symbols.

Understanding this workflow, helps understanding how `arwen` modifies an RPATH or a needed library. It's mostly targeting specific entries within the `.dynamic` section or the associated `.dynstr` string table within the ELF file layout.

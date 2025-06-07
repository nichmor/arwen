
To effectively use a tool like `arwen`, it helps to understand what you are actually modifying inside an executable file or shared library. Both `ELF (Executable and Linkable Format)` and `Mach-O (Mach Object)` are complex binary file formats that tell the operating system how to load and run code.


This tutorial won't cover every detail of these formats but will focus on the specific parts relevant to dynamic linking and the common patching operations performed by `arwen`.

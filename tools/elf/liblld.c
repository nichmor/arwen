// Source of tests/data/elf-lld/liblld.so, a shared library linked with lld.
// lld leaves no spare entries in .dynamic and places .got right after it, so
// adding a dynamic entry means moving .dynamic.
//
// Built without the compiler driver's link step, so no host paths end up in it
// (gcc 15.3, LLD 21.1.8):
//   gcc -O2 -fPIC -c -o liblld.o tools/elf/liblld.c
//   ld.lld -shared -o tests/data/elf-lld/liblld.so liblld.o -soname liblld.so \
//      -z relro -z now -s
extern int bar_counter;

int bar_next(void) { return ++bar_counter; }

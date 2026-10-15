// Source of tests/data/elf/x64/lib/libfoo.so, a small shared library fixture.
// It imports a versioned libc symbol (setlocale) and links libdl so that the
// DT_NEEDED and symbol-version operations have something to act on.
// Its PT_GNU_RELRO is padded past its last section to a page boundary (see foo_table).
//
// Built without the compiler driver's link step, so no host paths end up in it:
//   gcc -O2 -fPIC -c -o libfoo.o tools/elf/libfoo.c
//   ld -shared -o tests/data/elf/x64/lib/libfoo.so libfoo.o -soname libfoo.so.1 \
//      --no-as-needed -L<glibc>/lib -ldl -lc -z relro -z now \
//      -rpath /opt/foo/lib --enable-new-dtags -s
#include <dlfcn.h>
#include <locale.h>

const char *foo_locale(void) { return setlocale(LC_ALL, ""); }

void *foo_open(const char *path) { return dlopen(path, RTLD_NOW); }

// A 32-byte aligned table in .data.rel.ro. The linker can then only shift the
// start of PT_GNU_RELRO in 32-byte steps, so it pads the end of the segment up
// to the page boundary, beyond the last section it contains.
__attribute__((aligned(32))) const void *const foo_table[] = {(const void *)foo_locale,
                                                               (const void *)foo_open};

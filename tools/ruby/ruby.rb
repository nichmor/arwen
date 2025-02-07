require 'macho'

# MachO::Tools.add_rpath("x32/ls", "path_graf")
# MachO::Tools.delete_rpath("src/hello_with_change_rpath_arwen", "abababababababababababaabbababababababababab")
# MachO::Tools.change_rpath("tools/ruby/hello_with_rpath", "path_graf", "asdasdadasdadadsadasdasdsadadsasdasdad")
# MachO::Tools.change_install_name("src/hello_with_rpath", "/usr/lib/libSystem.B.dylib", "/usr/lib/libSystem.B.dylib")
MachO::Tools.change_dylib_id("x32/libhello.dylib", "graf_lib")

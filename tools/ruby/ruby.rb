require 'macho'

# MachO::Tools.delete_rpath("src/hello_with_change_rpath_arwen", "abababababababababababaabbababababababababab")
MachO::Tools.change_rpath("tools/ruby/hello_with_rpath", "path_graf", "asdasdadasdadadsadasdasdsadadsasdasdad")
# MachO::Tools.change_install_name("src/hello_with_rpath", "/usr/lib/libSystem.B.dylib", "/usr/lib/libSystem.B.dylib")




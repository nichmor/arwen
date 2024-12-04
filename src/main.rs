use std::mem;

use goblin::mach::{header::{self, Header, Header64, SIZEOF_HEADER_64}, load_command::{self, RpathCommand, Section64, SegmentCommand32, SegmentCommand64, LC_DYSYMTAB, LC_RPATH, LC_SEGMENT, LC_SEGMENT_64, SIZEOF_RPATH_COMMAND, SIZEOF_SECTION_64, SIZEOF_SEGMENT_COMMAND_64}, segment::{self, Segment}, MachO};
use scroll::{ctx::SizeWith, Pread, Pwrite, SizeWith};

use goblin::mach::load_command::CommandVariant::*;


fn modify_rpath(mut parsed_macho: MachO) -> Vec<u8> {

    // let's find existing LC_RPATH
    for load_command in &parsed_macho.load_commands {
        match load_command.command {
            Rpath(rpath) => {
                let existing_offset = load_command.offset;
                // write graf_path
                let mut existing_data = parsed_macho.data.to_vec();
                eprintln!("existing rpath size {:?}", rpath.path as usize);
                existing_data.pwrite("graf_path", existing_offset + rpath.path as usize).unwrap();

                return existing_data;
            },
            _ => {}
        }
    }

    return parsed_macho.data.to_vec();

}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // let write_obj = Object::new(object::BinaryFormat::MachO, object::Architecture::Arm, Endianness::default());

    let bytes_of_file = std::fs::read(&args[1]).unwrap();

    // modify rpath
    // let bytes_of_file: Vec<u8> = modify_rpath(MachO::parse(&bytes_of_file, 0).unwrap());


    // let's calculate the total size of all the header and commands
    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();


    let mut total_size = 0;

    let mut initial_offset = 0;

    let mut buffer = vec![0u8; parsed_macho.data.len()];

    let mut header = parsed_macho.header;
    header.ncmds += 1;
    // header.sizeofcmds -= SIZEOF_RPATH_COMMAND as u32;

    eprintln!("header size {:?}", header);

    // let wroted = buffer.pwrite(header, initial_offset).unwrap();

    eprintln!("all segments are {:?}", parsed_macho.segments);


    // let's start with load commands
    // initial_offset += wroted;

    for load_command in parsed_macho.load_commands {        
        match load_command.command {
            Segment64(segment_command64) => {                
                let wroted = buffer.pwrite(segment_command64, load_command.offset).unwrap();
                eprintln!("segment name {:?}", segment_command64.name());
                eprintln!("at the offset {:?}", load_command.offset);

                let fileoff = if segment_command64.fileoff < wroted as u64{
                    wroted - segment_command64.fileoff as usize
                } else {
                    segment_command64.fileoff as usize
                };

                eprintln!("segment data starts at {:?} and ends at {:?}", segment_command64.fileoff, segment_command64.fileoff + segment_command64.filesize);



                // and also write the data of segment
                let segment_bytes = parsed_macho.data[fileoff as usize..(fileoff as u64 + segment_command64.filesize) as usize].to_vec();
                let wroted = buffer.pwrite(segment_bytes.as_slice(), fileoff as usize).unwrap();

                // initial_offset += wroted;
            },
            Rpath(rpath_command) => {
                // duplicate it
                
                dbg!(rpath_command);

                let raw_str = c"costesti_bratka";
                let raw_str_size = raw_str.count_bytes();
                // and make it divible by 4
                let raw_str_size =  ((raw_str_size + 1 + 3) / 4) * 4;

                let cmd_size = SIZEOF_RPATH_COMMAND as u32 + raw_str_size as u32;
                let cmd_size =  ((cmd_size + 1 + 3) / 4) * 4;



                let new_rpath = RpathCommand{
                    cmd: LC_RPATH,
                    cmdsize: cmd_size,
                    path: raw_str_size as u32,
                };

                dbg!(new_rpath);
                

                eprintln!("rpath command is at {:?} offset", load_command.offset);
                // let's remove it
                let size_of_rpath = new_rpath.cmdsize;
                header.sizeofcmds += size_of_rpath;

                eprintln!("size of cmds {:?}", header.sizeofcmds);

                let mut rpath_buffer = vec![0u8; raw_str_size as usize];
                rpath_buffer.fill(0);

                // write the raw string and then fill it with zero bytes
                rpath_buffer.pwrite(raw_str, 0).unwrap();
                
                dbg!(raw_str_size);
                dbg!(rpath_buffer.len());
                dbg!(&rpath_buffer);

                // fill entire buffer with ending zero
                buffer.pwrite(new_rpath, load_command.offset + rpath_command.cmdsize as usize).unwrap();

                // write the path itself
                buffer.pwrite(rpath_buffer.clone().as_slice(), load_command.offset + rpath_command.cmdsize  as usize + rpath_command.path as usize).unwrap();

                // buffer.pwrite(rpath_buffer.as_slice(), load_command.offset).unwrap();
                // write the header back
                let wroted = buffer.pwrite(header, initial_offset).unwrap();
            },

            _ => {
                eprintln!("skipping for now");
            }
        }
        
        
        // let wroted = buffer.pwrite(load_command.command, initial_offset).unwrap();
        // initial_offset += wroted;
    }


    // panic!();

    
    // // write load commands
    // for load_command in parsed_macho.load_commands {
    //     let mut cmd_buffer = vec![0u8; load_command.command.cmdsize()];

        
    //     match load_command.command {
    //         Segment32(segment_command32) => {
    //             buffer.pwrite(segment_command32, 0).unwrap();
    //             eprintln!("WE HAD THIS ONE");
    //         },
    //         Segment64(segment_command64) => {
    //             // write load command
    //             // buffer.pwrite(segment_command64, 0).unwrap();
    //             // real_buffer.push(buffer.clone());

                
    //             let segment = &all_segments[sgm_index];
    //             let cloned_seg = SegmentCommand64{
    //                 cmd: segment.cmd,
    //                 cmdsize: segment.cmdsize,
    //                 segname: segment.segname,
    //                 vmaddr: segment.vmaddr,
    //                 vmsize: segment.vmsize,
    //                 fileoff: segment.fileoff,
    //                 filesize: segment.filesize,
    //                 maxprot: segment.maxprot,
    //                 initprot: segment.initprot,
    //                 nsects: segment.nsects,
    //                 flags: segment.flags,
    //             };
    //             // let mut buffer = vec![0u8; SIZEOF_SEGMENT_COMMAND_64];
    //             cmd_buffer.pwrite(cloned_seg, 0).unwrap();
    //             real_buffer.push(buffer.clone());
        
    //             // now write sections
    //             for (section, data) in segment.sections().unwrap(){            
    //                 eprintln!("section is {:?}", section);
    //                 eprintln!("data of is {:?}", data);
    //                 let section_as_64: Section64 = section.into();

    //                 let mut buffer = vec![0u8; SIZEOF_SECTION_64];
        
    //                 buffer.pwrite(section_as_64, 0).unwrap();
    //                 eprintln!("buffer is {:?}", buffer);
        
    //                 real_buffer.push(buffer.clone());
    //                 // real_buffer.push(data.to_vec());
        
    //             }
    //             // real_buffer.push(segment.dsata.to_vec());
    //             sgm_index += 1;

    //             // // buffer.pwrite(segment_command64, 0).unwrap();

    //             // println!("buffer of segment 64 {:?}", buffer);
    //             // continue;
    //         },
    //         Uuid(uuid_command) => {
    //             buffer.pwrite(uuid_command, 0).unwrap();
    //         },
    //         Symtab(symtab_command) => {
    //             eprintln!("symtab command is {:?}", symtab_command);
    //             buffer.pwrite(symtab_command, 0).unwrap();
    //         },
    //         Symseg(symseg_command) => {
    //             buffer.pwrite(symseg_command, 0).unwrap();
    //         },
    //         // Thread(thread_command) => {
    //         //     buffer.pwrite(thread_command, 0).unwrap();
    //         // },
    //         // Unixthread(thread_command) => {
    //         //     buffer.pwrite(thread_command, 0).unwrap();
    //         // },
    //         LoadFvmlib(fvmlib_command) => {
    //             buffer.pwrite(fvmlib_command, 0).unwrap();
    //         },
    //         IdFvmlib(fvmlib_command) => {
    //             buffer.pwrite(fvmlib_command, 0).unwrap();
    //         },
    //         Ident(ident_command) => {
    //             buffer.pwrite(ident_command, 0).unwrap();
    //         },
    //         Fvmfile(fvmfile_command) => {
    //             buffer.pwrite(fvmfile_command, 0).unwrap();
    //         },
    //         Prepage(load_command_header) => {
    //             buffer.pwrite(load_command_header, 0).unwrap();
    //         },
    //         Dysymtab(dysymtab_command) => {
    //             buffer.pwrite(dysymtab_command, 0).unwrap();
    //         },
    //         LoadDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         IdDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         LoadDylinker(dylinker_command) => {
    //             buffer.pwrite(dylinker_command, 0).unwrap();
    //         },
    //         IdDylinker(dylinker_command) => {
    //             buffer.pwrite(dylinker_command, 0).unwrap();
    //         },
    //         PreboundDylib(prebound_dylib_command) => {
    //             buffer.pwrite(prebound_dylib_command, 0).unwrap();
    //         },
    //         Routines32(routines_command32) => {
    //             buffer.pwrite(routines_command32, 0).unwrap();
    //         },
    //         Routines64(routines_command64) => {
    //             buffer.pwrite(routines_command64, 0).unwrap();
    //         },
    //         SubFramework(sub_framework_command) => {
    //             buffer.pwrite(sub_framework_command, 0).unwrap();
    //         },
    //         SubUmbrella(sub_umbrella_command) => {
    //             buffer.pwrite(sub_umbrella_command, 0).unwrap();
    //         },
    //         SubClient(sub_client_command) => {
    //             buffer.pwrite(sub_client_command, 0).unwrap();
    //         },
    //         SubLibrary(sub_library_command) => {
    //             buffer.pwrite(sub_library_command, 0).unwrap();
    //         },
    //         TwolevelHints(twolevel_hints_command) => {
    //             buffer.pwrite(twolevel_hints_command, 0).unwrap();
    //         },
    //         PrebindCksum(prebind_cksum_command) => {
    //             buffer.pwrite(prebind_cksum_command, 0).unwrap();
    //         },
    //         LoadWeakDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         Rpath(rpath_command) => {
    //             buffer.pwrite(rpath_command, 0).unwrap();
    //         },
    //         CodeSignature(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         SegmentSplitInfo(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         ReexportDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         LazyLoadDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         EncryptionInfo32(encryption_info_command32) => {
    //             buffer.pwrite(encryption_info_command32, 0).unwrap();
    //         },
    //         EncryptionInfo64(encryption_info_command64) => {
    //             buffer.pwrite(encryption_info_command64, 0).unwrap();
    //         },
    //         DyldInfo(dyld_info_command) => {
    //             buffer.pwrite(dyld_info_command, 0).unwrap();
    //         },
    //         DyldInfoOnly(dyld_info_command) => {
    //             buffer.pwrite(dyld_info_command, 0).unwrap();
    //         },
    //         LoadUpwardDylib(dylib_command) => {
    //             buffer.pwrite(dylib_command, 0).unwrap();
    //         },
    //         VersionMinMacosx(version_min_command) => {
    //             buffer.pwrite(version_min_command, 0).unwrap();
    //         },
    //         VersionMinIphoneos(version_min_command) => {
    //             buffer.pwrite(version_min_command, 0).unwrap();
    //         },
    //         FunctionStarts(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         DyldEnvironment(dylinker_command) => {
    //             buffer.pwrite(dylinker_command, 0).unwrap();
    //         },
    //         Main(entry_point_command) => {
    //             buffer.pwrite(entry_point_command, 0).unwrap();
    //         },
    //         DataInCode(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         BuildVersion(build_version_command) => {
    //             buffer.pwrite(build_version_command, 0).unwrap();
    //         },
    //         FilesetEntry(fileset_entry_command) => {
    //             buffer.pwrite(fileset_entry_command, 0).unwrap();
    //         },
    //         SourceVersion(source_version_command) => {
    //             buffer.pwrite(source_version_command, 0).unwrap();
    //         },
    //         DylibCodeSignDrs(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         LinkerOption(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         LinkerOptimizationHint(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         VersionMinTvos(version_min_command) => {
    //             buffer.pwrite(version_min_command, 0).unwrap();
    //         },
    //         VersionMinWatchos(version_min_command) => {
    //             buffer.pwrite(version_min_command, 0).unwrap();
    //         },
    //         DyldExportsTrie(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         DyldChainedFixups(linkedit_data_command) => {
    //             buffer.pwrite(linkedit_data_command, 0).unwrap();
    //         },
    //         Note(note_command) => {
    //             buffer.pwrite(note_command, 0).unwrap();
    //         },
    //         Unimplemented(load_command_header) => {
    //             buffer.pwrite(load_command_header, 0).unwrap();
    //         },
    //         _ => unimplemented!("Not implemented"),
    //     }
        
    // }
    // //     // buffer.pwrite(load_command.command, offset).unwrap();

    // //     real_buffer.push(buffer.clone());
    // //     // break;
        
    // //     // buffer.clear();
        
    // //     // offset += offset + load_command.command.cmdsize();
    // // }

    // eprintln!("TOTAL SEGMENTS {:?}", all_segments.len());

    // eprintln!("USED {:?}", sgm_index);
    // let buffer_concat = real_buffer.concat();

    // // eprintln!("real buffer is {:?}", buffer_concat);

    // // write back into a binary file
    std::fs::write("hello_with_removed", buffer).unwrap();



}

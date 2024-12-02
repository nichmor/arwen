use std::mem;

use goblin::mach::{header::Header64, load_command::{Section64, SegmentCommand32, SegmentCommand64, LC_DYSYMTAB, LC_SEGMENT, LC_SEGMENT_64, SIZEOF_SECTION_64, SIZEOF_SEGMENT_COMMAND_64}, segment::{self, Segment}, MachO};
use scroll::{Pread, Pwrite, SizeWith};

use goblin::mach::load_command::CommandVariant::*;


fn main() {
    let args: Vec<String> = std::env::args().collect();

    // let write_obj = Object::new(object::BinaryFormat::MachO, object::Architecture::Arm, Endianness::default());




    let bytes_of_file = std::fs::read(&args[1]).unwrap();

    let parsed_macho = MachO::parse(&bytes_of_file, 0).unwrap();

    eprintln!("all data size {:?}", parsed_macho.data.len());

    
    let header_64: Header64 = parsed_macho.header.into();




    eprintln!("header sizeofcmds {:?}", header_64.sizeofcmds);




    let mut buffer = vec![0u8; header_64.size()];


    let mut real_buffer = Vec::new();

    let mut offset = 0;

    // Writing header
    buffer.pwrite(header_64, 0).unwrap();

    // real_buffer.extend_from_slice(&buffer.clone());

    real_buffer.push(buffer.clone());


    offset += header_64.size();

    let all_segments = parsed_macho.segments;
    let mut sgm_index = 0;

    // Write only segments

    // writing segments
    // Writing segments and sections
    for segment in &all_segments {
        eprintln!("SEGMENT: {:?}", segment);

        // Write the segment header
        let mut buffer = vec![0u8; SIZEOF_SEGMENT_COMMAND_64];
        let segment_command = SegmentCommand64 {
            cmd: segment.cmd,
            cmdsize: segment.cmdsize,
            segname: segment.segname,
            vmaddr: segment.vmaddr,
            vmsize: segment.vmsize,
            fileoff: segment.fileoff,
            filesize: segment.filesize,
            maxprot: segment.maxprot,
            initprot: segment.initprot,
            nsects: segment.nsects,
            flags: segment.flags,
        };

        // Write the segment header to the buffer
        let wrote = buffer.pwrite(segment_command, 0).unwrap();
        eprintln!("Segment header size: {}, wrote: {}", buffer.len(), wrote);
        real_buffer.push(buffer.clone());

        // Write each section in the segment
        for (section, data) in segment.sections().unwrap() {
            eprintln!("SECTION: {:?}", section);

            let section_64 = Section64 {
                sectname: section.sectname,
                segname: section.segname,
                addr: section.addr,
                size: section.size,
                offset: section.offset,
                align: section.align,
                reloff: section.reloff,
                nreloc: section.nreloc,
                flags: section.flags,
                reserved1: 0,
                reserved2: 0,
                reserved3: 0,
            };

            // Write section header
            let mut section_buffer = vec![0u8; SIZEOF_SECTION_64];
            let wrote = section_buffer.pwrite(section_64, 0).unwrap();
            eprintln!("Section header size: {}, wrote: {}", section_buffer.len(), wrote);
            real_buffer.push(section_buffer.clone());

            // Align the section data if necessary
            let align_mask = (1 << section.align) - 1;
            if real_buffer.len() & align_mask != 0 {
                let padding = align_mask - (real_buffer.len() & align_mask);
                eprintln!("Adding padding of {} bytes for alignment", padding);
                let padding_vec = vec![0u8; padding];
                
                real_buffer.push(padding_vec);
            }

            // Write the section data
            real_buffer.push(data.to_vec());
        }

        // Write segment data (if any)
        if !segment.data.is_empty() {
            eprintln!("Writing segment data, size: {}", segment.data.len());
            real_buffer.push(segment.data.to_vec());
        }
    }

    

    // write load commands
    // Real trick here is that if we hit load_segment,
    // we need to write it separately and then write the sections
    // for load_command in parsed_macho.load_commands {
    //     let mut buffer = vec![0u8; load_command.command.cmdsize()];
        
    //     match load_command.command {
    //         Segment32(segment_command32) => {
    //             buffer.pwrite(segment_command32, 0).unwrap();
    //             eprintln!("WE HAD THIS ONE");
    //         },
    //         Segment64(segment_command64) => {
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
    //             let mut buffer = vec![0u8; SIZEOF_SEGMENT_COMMAND_64];
    //             buffer.pwrite(cloned_seg, 0).unwrap();
    //             real_buffer.push(buffer.clone());
        
    //             // now write sections
    //             for (section, data) in segment.sections().unwrap(){            
    //                 eprintln!("section is {:?}", section);
    //                 eprintln!("data of is {:?}", data);
    //                 let section_as_64: Section64 = section.into();

    //                 let mut buffer = vec![0u8; SIZEOF_SECTION_64];
        
    //                 buffer.pwrite(section_as_64, 0).unwrap();
    //                 eprintln!("buffer is {:?}", buffer);
        
    //                 // real_buffer.push(buffer.clone());
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
        
        
    //     // buffer.pwrite(load_command.command, offset).unwrap();

    //     real_buffer.push(buffer.clone());
    //     // break;
        
    //     // buffer.clear();
        
    //     // offset += offset + load_command.command.cmdsize();
    // }

    eprintln!("TOTAL SEGMENTS {:?}", all_segments.len());

    eprintln!("USED {:?}", sgm_index);
    let buffer_concat = real_buffer.concat();

    // eprintln!("real buffer is {:?}", buffer_concat);

    // write back into a binary file
    std::fs::write("new_binary", buffer_concat).unwrap();



}

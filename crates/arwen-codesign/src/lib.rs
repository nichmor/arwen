//! Ad-hoc code signing for Mach-O binaries
//!
//! This module implements ad-hoc code signing for Mach-O binaries,
//! matching Apple's codesign behavior for linker-signed binaries.
//!
//! # Example
//!
//! ```ignore
//! use arwen_codesign::{adhoc_sign, AdhocSignOptions, Entitlements};
//!
//! // Simple signing with just an identifier
//! let signed = adhoc_sign(data, &AdhocSignOptions::new("com.example.myapp"))?;
//!
//! // With hardened runtime and preserved entitlements
//! let options = AdhocSignOptions::new("com.example.myapp")
//!     .with_hardened_runtime()
//!     .with_entitlements(Entitlements::Preserve);
//! let signed = adhoc_sign(data, &options)?;
//! ```

extern crate alloc;
use alloc::vec::Vec;
use goblin::mach::{
    header::Header,
    load_command::{LC_CODE_SIGNATURE, LC_SEGMENT, LC_SEGMENT_64},
    parse_magic_and_ctx,
};
use goblin::{container, error};

use scroll::{
    ctx::{SizeWith, TryIntoCtx},
    Endian, Pread, Pwrite, BE,
};

/// Code signature magic numbers and constants
pub mod constants {
    /// Magic number for embedded signature SuperBlob
    pub const CSMAGIC_EMBEDDED_SIGNATURE: u32 = 0xfade0cc0;
    /// Magic number for CodeDirectory blob
    pub const CSMAGIC_CODEDIRECTORY: u32 = 0xfade0c02;
    /// Magic number for Requirements blob
    pub const CSMAGIC_REQUIREMENTS: u32 = 0xfade0c01;
    /// Magic number for embedded entitlements (plist format)
    pub const CSMAGIC_EMBEDDED_ENTITLEMENTS: u32 = 0xfade7171;
    /// Magic number for embedded entitlements (DER format)
    pub const CSMAGIC_EMBEDDED_ENTITLEMENTS_DER: u32 = 0xfade7172;
    /// Slot index for CodeDirectory
    pub const CSSLOT_CODEDIRECTORY: u32 = 0;
    /// Slot index for Info.plist (special slot -1)
    pub const CSSLOT_INFOSLOT: u32 = 1;
    /// Slot index for Requirements (special slot -2)
    pub const CSSLOT_REQUIREMENTS: u32 = 2;
    /// Slot index for entitlements
    pub const CSSLOT_ENTITLEMENTS: u32 = 5;
    /// Slot index for DER entitlements
    pub const CSSLOT_ENTITLEMENTS_DER: u32 = 7;
    /// SHA-256 hash type
    pub const CS_HASHTYPE_SHA256: u8 = 2;
    /// Ad-hoc signature flag
    pub const CS_ADHOC: u32 = 0x0002;
    /// Hardened runtime flag (--options runtime)
    pub const CS_RUNTIME: u32 = 0x10000;
    /// Linker-signed flag
    pub const CS_LINKER_SIGNED: u32 = 0x20000;
    /// Main binary exec segment flag
    pub const CS_EXECSEG_MAIN_BINARY: u64 = 0x1;
    /// Code signature page size (4KB)
    pub const CS_PAGE_SIZE: usize = 4096;
    /// Code signature page size as log2
    pub const CS_PAGE_SIZE_LOG2: u8 = 12;
    /// CodeDirectory version
    pub const CS_VERSION: u32 = 0x20400;
}

/// How to handle entitlements during ad-hoc signing
#[derive(Debug, Clone, Default)]
pub enum Entitlements<'a> {
    /// No entitlements
    #[default]
    None,
    /// Preserve existing entitlements from the binary's current signature
    Preserve,
    /// Use custom entitlements plist data
    Custom(&'a [u8]),
}

/// Options for ad-hoc code signing
///
/// # Example
/// ```ignore
/// use arwen_codesign::{adhoc_sign, AdhocSignOptions, Entitlements};
///
/// let options = AdhocSignOptions {
///     identifier: "com.example.myapp",
///     hardened_runtime: true,
///     entitlements: Entitlements::Preserve,
/// };
/// let signed = adhoc_sign(binary_data, &options)?;
/// ```
#[derive(Debug, Clone)]
pub struct AdhocSignOptions<'a> {
    /// The identifier to embed in the signature (e.g., "com.example.myapp")
    pub identifier: &'a str,
    /// Enable hardened runtime (equivalent to `codesign --options runtime`)
    pub hardened_runtime: bool,
    /// How to handle entitlements
    pub entitlements: Entitlements<'a>,
    /// Set the linker-signed flag (CS_LINKER_SIGNED).
    /// This should be true when re-signing a linker-signed binary,
    /// false when doing a fresh ad-hoc sign like `codesign -s -`.
    pub linker_signed: bool,
}

impl<'a> AdhocSignOptions<'a> {
    /// Create new options with just an identifier (no hardened runtime, no entitlements)
    pub fn new(identifier: &'a str) -> Self {
        Self {
            identifier,
            hardened_runtime: false,
            entitlements: Entitlements::None,
            linker_signed: false,
        }
    }

    /// Enable hardened runtime
    pub fn with_hardened_runtime(mut self) -> Self {
        self.hardened_runtime = true;
        self
    }

    /// Set entitlements handling
    pub fn with_entitlements(mut self, entitlements: Entitlements<'a>) -> Self {
        self.entitlements = entitlements;
        self
    }

    /// Set the linker-signed flag (use when re-signing linker-signed binaries)
    pub fn with_linker_signed(mut self) -> Self {
        self.linker_signed = true;
        self
    }
}

use constants::*;
use sha2::{Digest, Sha256};

/// SuperBlob header for embedded signature
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct SuperBlob {
    magic: u32,
    length: u32,
    count: u32,
}

/// Blob index entry
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct BlobIndex {
    typ: u32,
    offset: u32,
}

/// CodeDirectory structure (version 0x20400)
#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct CodeDirectory {
    magic: u32,
    length: u32,
    version: u32,
    flags: u32,
    hash_offset: u32,
    ident_offset: u32,
    n_special_slots: u32,
    n_code_slots: u32,
    code_limit: u32,
    hash_size: u8,
    hash_type: u8,
    _pad1: u8,
    page_size: u8,
    _pad2: u32,
    scatter_offset: u32,
    team_offset: u32,
    _pad3: u32,
    code_limit64: u64,
    exec_seg_base: u64,
    exec_seg_limit: u64,
    exec_seg_flags: u64,
}

impl TryIntoCtx<Endian> for SuperBlob {
    type Error = scroll::Error;

    fn try_into_ctx(self, dst: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.magic, offset, ctx)?;
        dst.gwrite_with(self.length, offset, ctx)?;
        dst.gwrite_with(self.count, offset, ctx)?;
        Ok(*offset)
    }
}

impl TryIntoCtx<Endian> for BlobIndex {
    type Error = scroll::Error;

    fn try_into_ctx(self, dst: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.typ, offset, ctx)?;
        dst.gwrite_with(self.offset, offset, ctx)?;
        Ok(*offset)
    }
}

impl TryIntoCtx<Endian> for CodeDirectory {
    type Error = scroll::Error;

    fn try_into_ctx(self, dst: &mut [u8], ctx: Endian) -> Result<usize, Self::Error> {
        let offset = &mut 0;
        dst.gwrite_with(self.magic, offset, ctx)?;
        dst.gwrite_with(self.length, offset, ctx)?;
        dst.gwrite_with(self.version, offset, ctx)?;
        dst.gwrite_with(self.flags, offset, ctx)?;
        dst.gwrite_with(self.hash_offset, offset, ctx)?;
        dst.gwrite_with(self.ident_offset, offset, ctx)?;
        dst.gwrite_with(self.n_special_slots, offset, ctx)?;
        dst.gwrite_with(self.n_code_slots, offset, ctx)?;
        dst.gwrite_with(self.code_limit, offset, ctx)?;
        dst.gwrite(self.hash_size, offset)?;
        dst.gwrite(self.hash_type, offset)?;
        dst.gwrite(self._pad1, offset)?;
        dst.gwrite(self.page_size, offset)?;
        dst.gwrite_with(self._pad2, offset, ctx)?;
        dst.gwrite_with(self.scatter_offset, offset, ctx)?;
        dst.gwrite_with(self.team_offset, offset, ctx)?;
        dst.gwrite_with(self._pad3, offset, ctx)?;
        dst.gwrite_with(self.code_limit64, offset, ctx)?;
        dst.gwrite_with(self.exec_seg_base, offset, ctx)?;
        dst.gwrite_with(self.exec_seg_limit, offset, ctx)?;
        dst.gwrite_with(self.exec_seg_flags, offset, ctx)?;
        Ok(*offset)
    }
}

// =============================================================================
// Mach-O load command parsing helpers
// =============================================================================

/// Information extracted from Mach-O load commands needed for code signing
#[derive(Debug, Clone, Default)]
struct MachOLoadInfo {
    /// Offset of LC_CODE_SIGNATURE load command
    codesig_cmd_offset: Option<usize>,
    /// File offset where code signature data starts
    codesig_data_offset: usize,
    /// Size of existing code signature data
    codesig_data_size: usize,
    /// Offset of __LINKEDIT segment command
    linkedit_cmd_offset: Option<usize>,
    /// File offset of __LINKEDIT segment
    linkedit_fileoff: u64,
    /// File offset of __TEXT segment
    text_fileoff: u64,
    /// File size of __TEXT segment
    text_filesize: u64,
    /// Whether this is a 64-bit binary
    is_64bit: bool,
    /// Whether this is a main executable (MH_EXECUTE)
    is_executable: bool,
}

/// Parse Mach-O load commands and extract code signing related information
fn parse_macho_load_info(data: &[u8]) -> error::Result<MachOLoadInfo> {
    let (_, ctx_opt) = parse_magic_and_ctx(data, 0)?;
    let ctx = ctx_opt.ok_or(error::Error::Malformed("Invalid Mach-O magic".into()))?;
    let header: Header = data.pread_with(0, ctx)?;
    let is_64bit = ctx.container == container::Container::Big;
    let header_size = Header::size_with(&ctx);

    let mut info = MachOLoadInfo {
        is_64bit,
        is_executable: header.filetype == 2, // MH_EXECUTE
        ..Default::default()
    };

    let mut offset = header_size;
    for _ in 0..header.ncmds {
        let cmd: u32 = data.pread_with(offset, ctx.le)?;
        let cmdsize: u32 = data.pread_with(offset + 4, ctx.le)?;

        match cmd {
            LC_CODE_SIGNATURE => {
                info.codesig_cmd_offset = Some(offset);
                info.codesig_data_offset = data.pread_with::<u32>(offset + 8, ctx.le)? as usize;
                info.codesig_data_size = data.pread_with::<u32>(offset + 12, ctx.le)? as usize;
            }
            LC_SEGMENT_64 => {
                let segname = parse_segment_name(&data[offset + 8..offset + 24]);
                match segname {
                    "__LINKEDIT" => {
                        info.linkedit_cmd_offset = Some(offset);
                        info.linkedit_fileoff = data.pread_with(offset + 40, ctx.le)?;
                    }
                    "__TEXT" => {
                        info.text_fileoff = data.pread_with(offset + 40, ctx.le)?;
                        info.text_filesize = data.pread_with(offset + 48, ctx.le)?;
                    }
                    _ => {}
                }
            }
            LC_SEGMENT => {
                let segname = parse_segment_name(&data[offset + 8..offset + 24]);
                match segname {
                    "__LINKEDIT" => {
                        info.linkedit_cmd_offset = Some(offset);
                        info.linkedit_fileoff = data.pread_with::<u32>(offset + 32, ctx.le)? as u64;
                    }
                    "__TEXT" => {
                        info.text_fileoff = data.pread_with::<u32>(offset + 32, ctx.le)? as u64;
                        info.text_filesize = data.pread_with::<u32>(offset + 36, ctx.le)? as u64;
                    }
                    _ => {}
                }
            }
            _ => {}
        }

        offset += cmdsize as usize;
    }

    Ok(info)
}

/// Parse segment name from 16-byte buffer
fn parse_segment_name(bytes: &[u8]) -> &str {
    core::str::from_utf8(bytes)
        .unwrap_or("")
        .trim_end_matches('\0')
}

// =============================================================================
// SuperBlob parsing helpers
// =============================================================================

/// Entry from a SuperBlob's blob index
#[derive(Debug, Clone, Copy)]
struct BlobEntry {
    /// Blob type (slot index)
    blob_type: u32,
    /// Offset of blob data within the signature
    blob_offset: usize,
}

/// Iterator over blob entries in a SuperBlob
struct SuperBlobIter<'a> {
    sig_data: &'a [u8],
    count: usize,
    current: usize,
}

impl<'a> Iterator for SuperBlobIter<'a> {
    type Item = BlobEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let idx_offset = 12 + self.current * 8;
        if idx_offset + 8 > self.sig_data.len() {
            return None;
        }

        let blob_type: u32 = self.sig_data.pread_with(idx_offset, BE).ok()?;
        let blob_offset: u32 = self.sig_data.pread_with(idx_offset + 4, BE).ok()?;

        self.current += 1;
        Some(BlobEntry {
            blob_type,
            blob_offset: blob_offset as usize,
        })
    }
}

/// Parse a SuperBlob and return an iterator over its blob entries
fn iter_superblob(sig_data: &[u8]) -> Option<SuperBlobIter<'_>> {
    if sig_data.len() < 12 {
        return None;
    }

    let magic: u32 = sig_data.pread_with(0, BE).ok()?;
    if magic != CSMAGIC_EMBEDDED_SIGNATURE {
        return None;
    }

    let count: u32 = sig_data.pread_with(8, BE).ok()?;
    let count = count as usize;
    Some(SuperBlobIter {
        sig_data,
        count,
        current: 0,
    })
}

/// Extract entitlements plist data from a code signature blob
fn extract_entitlements_from_superblob(sig_data: &[u8]) -> Option<Vec<u8>> {
    let iter = iter_superblob(sig_data)?;

    for entry in iter {
        if entry.blob_type == CSSLOT_ENTITLEMENTS && entry.blob_offset + 8 <= sig_data.len() {
            let blob_magic: u32 = sig_data.pread_with(entry.blob_offset, BE).ok()?;
            let blob_length: u32 = sig_data.pread_with(entry.blob_offset + 4, BE).ok()?;
            let blob_length = blob_length as usize;

            if blob_magic == CSMAGIC_EMBEDDED_ENTITLEMENTS
                && entry.blob_offset + blob_length <= sig_data.len()
            {
                let plist_data = &sig_data[entry.blob_offset + 8..entry.blob_offset + blob_length];
                return Some(plist_data.to_vec());
            }
        }
    }
    None
}

/// Check if a code signature blob has the linker-signed flag
fn check_linker_signed_in_superblob(sig_data: &[u8]) -> bool {
    let Some(iter) = iter_superblob(sig_data) else {
        return false;
    };

    for entry in iter {
        if entry.blob_type == CSSLOT_CODEDIRECTORY && entry.blob_offset + 16 <= sig_data.len() {
            if let Ok(flags) = sig_data.pread_with::<u32>(entry.blob_offset + 12, BE) {
                return (flags & CS_LINKER_SIGNED) != 0;
            }
        }
    }
    false
}

/// Check if a Mach-O binary has a linker-signed code signature
///
/// This function parses the binary to find the code signature and checks
/// if it has the CS_LINKER_SIGNED flag (0x20000). Returns false if no
/// code signature is found or if it doesn't have the linker-signed flag.
pub fn is_linker_signed(data: &[u8]) -> bool {
    let Ok(info) = parse_macho_load_info(data) else {
        return false;
    };

    if info.codesig_cmd_offset.is_none() || info.codesig_data_size == 0 {
        return false;
    }

    let end = info.codesig_data_offset + info.codesig_data_size;
    if end > data.len() {
        return false;
    }

    let sig_data = &data[info.codesig_data_offset..end];
    check_linker_signed_in_superblob(sig_data)
}

/// Extract entitlements from a Mach-O binary's code signature
///
/// Returns the raw entitlements plist data if present, or None if no entitlements are found.
/// This is useful for implementing `--preserve-metadata=entitlements` functionality.
pub fn extract_entitlements(data: &[u8]) -> Option<Vec<u8>> {
    let info = parse_macho_load_info(data).ok()?;

    if info.codesig_cmd_offset.is_none() || info.codesig_data_size == 0 {
        return None;
    }

    let end = info.codesig_data_offset + info.codesig_data_size;
    if end > data.len() {
        return None;
    }

    let sig_data = &data[info.codesig_data_offset..end];
    extract_entitlements_from_superblob(sig_data)
}

/// Generate an ad-hoc code signature for a Mach-O binary
///
/// This is a low-level function. Most users should use [`adhoc_sign`] instead.
///
/// # Arguments
/// * `data` - The binary data (will be modified in place)
/// * `identifier` - The identifier string to use in the signature
/// * `codesig_cmd_offset` - Offset of LC_CODE_SIGNATURE load command
/// * `codesig_data_offset` - Offset where code signature data starts
/// * `linkedit_cmd_offset` - Offset of __LINKEDIT segment command
/// * `linkedit_fileoff` - File offset of __LINKEDIT segment
/// * `text_fileoff` - File offset of __TEXT segment
/// * `text_filesize` - File size of __TEXT segment
/// * `is_64bit` - Whether this is a 64-bit binary
/// * `is_executable` - Whether this is a main executable (MH_EXECUTE)
/// * `hardened_runtime` - Whether to enable hardened runtime (CS_RUNTIME flag)
/// * `linker_signed` - Whether to set the linker-signed flag (CS_LINKER_SIGNED)
/// * `entitlements` - Optional entitlements plist data to embed
///
/// Returns the new binary data with updated signature
#[allow(clippy::too_many_arguments)]
pub fn generate_adhoc_signature(
    mut data: Vec<u8>,
    identifier: &str,
    codesig_cmd_offset: usize,
    codesig_data_offset: usize,
    linkedit_cmd_offset: usize,
    linkedit_fileoff: u64,
    text_fileoff: u64,
    text_filesize: u64,
    is_64bit: bool,
    is_executable: bool,
    hardened_runtime: bool,
    linker_signed: bool,
    entitlements: Option<&[u8]>,
) -> error::Result<Vec<u8>> {
    // Calculate sizes
    let id_bytes = identifier.as_bytes();
    let id_len = id_bytes.len() + 1; // Include null terminator
    let n_hashes = codesig_data_offset.div_ceil(CS_PAGE_SIZE);

    // Check if we have entitlements
    let has_entitlements = entitlements.is_some();
    let entitlements_data = entitlements.unwrap_or(&[]);

    // Entitlements blob: 8-byte header (magic + length) + plist data
    let entitlements_blob_size = if has_entitlements {
        8 + entitlements_data.len()
    } else {
        0
    };

    // Number of blobs: CodeDirectory + optional Entitlements
    let blob_count = if has_entitlements { 2 } else { 1 };
    let n_special_slots: u32 = if has_entitlements {
        CSSLOT_ENTITLEMENTS // 5 - we need slots 1-5
    } else {
        0
    };

    let superblob_size = 12; // SuperBlob header
    let blob_indices_size = blob_count * 8; // BlobIndex entries
    let codedir_size = 88; // CodeDirectory header

    // Special slots hashes come BEFORE code hashes in the hash array
    // They are stored at negative indices: slot 1 is at -1, slot 5 is at -5
    let special_hashes_size = n_special_slots as usize * 32;
    let code_hashes_size = n_hashes * 32;

    let hash_offset = codedir_size + id_len + special_hashes_size;
    let codedir_total = codedir_size + id_len + special_hashes_size + code_hashes_size;

    // Calculate total blob content size
    let blob_content_size =
        superblob_size + blob_indices_size + codedir_total + entitlements_blob_size;
    // Apple aligns code signature datasize to 8 bytes
    let padded_sig_size = (blob_content_size + 7) & !7;

    // Calculate blob offsets
    let codedir_offset = superblob_size + blob_indices_size;
    let entitlements_offset = codedir_offset + codedir_total;

    // Build the signature
    let superblob = SuperBlob {
        magic: CSMAGIC_EMBEDDED_SIGNATURE,
        length: blob_content_size as u32,
        count: blob_count as u32,
    };

    // Update LC_CODE_SIGNATURE command FIRST (before hashing)
    let datasize_offset = codesig_cmd_offset + 12;
    data[datasize_offset..datasize_offset + 4]
        .copy_from_slice(&(padded_sig_size as u32).to_le_bytes());

    // Update __LINKEDIT segment filesize FIRST (before hashing)
    let new_linkedit_filesize =
        codesig_data_offset as u64 + padded_sig_size as u64 - linkedit_fileoff;
    if is_64bit {
        let filesize_offset = linkedit_cmd_offset + 48;
        data[filesize_offset..filesize_offset + 8]
            .copy_from_slice(&new_linkedit_filesize.to_le_bytes());
    } else {
        let filesize_offset = linkedit_cmd_offset + 36;
        data[filesize_offset..filesize_offset + 4]
            .copy_from_slice(&(new_linkedit_filesize as u32).to_le_bytes());
    }

    // Build signature blob content - pre-allocate and use gwrite_with
    let mut sig = vec![0u8; padded_sig_size];
    let mut offset = 0usize;

    // Write SuperBlob header
    sig.gwrite_with(superblob, &mut offset, BE)
        .map_err(|e| error::Error::Malformed(e.to_string()))?;

    // Write BlobIndex entries
    let codedir_index = BlobIndex {
        typ: CSSLOT_CODEDIRECTORY,
        offset: codedir_offset as u32,
    };
    sig.gwrite_with(codedir_index, &mut offset, BE)
        .map_err(|e| error::Error::Malformed(e.to_string()))?;

    if has_entitlements {
        let ent_index = BlobIndex {
            typ: CSSLOT_ENTITLEMENTS,
            offset: entitlements_offset as u32,
        };
        sig.gwrite_with(ent_index, &mut offset, BE)
            .map_err(|e| error::Error::Malformed(e.to_string()))?;
    }

    // Calculate entitlements hash for special slot (if present)
    let entitlements_hash: [u8; 32] = if has_entitlements {
        // Build the entitlements blob first to hash it
        let mut ent_blob = vec![0u8; entitlements_blob_size];
        let mut ent_offset = 0usize;
        ent_blob
            .gwrite_with(CSMAGIC_EMBEDDED_ENTITLEMENTS, &mut ent_offset, BE)
            .map_err(|e| error::Error::Malformed(e.to_string()))?;
        ent_blob
            .gwrite_with(entitlements_blob_size as u32, &mut ent_offset, BE)
            .map_err(|e| error::Error::Malformed(e.to_string()))?;
        ent_blob[ent_offset..].copy_from_slice(entitlements_data);

        let mut hasher = Sha256::new();
        hasher.update(&ent_blob);
        hasher.finalize().into()
    } else {
        [0u8; 32]
    };

    // Build CodeDirectory
    let mut flags = CS_ADHOC;
    if linker_signed {
        flags |= CS_LINKER_SIGNED;
    }
    if hardened_runtime {
        flags |= CS_RUNTIME;
    }
    let codedir = CodeDirectory {
        magic: CSMAGIC_CODEDIRECTORY,
        length: codedir_total as u32,
        version: CS_VERSION,
        flags,
        hash_offset: hash_offset as u32,
        ident_offset: codedir_size as u32,
        n_special_slots,
        n_code_slots: n_hashes as u32,
        code_limit: codesig_data_offset as u32,
        hash_size: 32,
        hash_type: CS_HASHTYPE_SHA256,
        _pad1: 0,
        page_size: CS_PAGE_SIZE_LOG2,
        _pad2: 0,
        scatter_offset: 0,
        team_offset: 0,
        _pad3: 0,
        code_limit64: 0,
        exec_seg_base: text_fileoff,
        exec_seg_limit: text_filesize,
        exec_seg_flags: if is_executable {
            CS_EXECSEG_MAIN_BINARY
        } else {
            0
        },
    };

    sig.gwrite_with(codedir, &mut offset, BE)
        .map_err(|e| error::Error::Malformed(e.to_string()))?;

    // Write identifier
    sig[offset..offset + id_bytes.len()].copy_from_slice(id_bytes);
    offset += id_bytes.len();
    sig[offset] = 0; // Null terminator
    offset += 1;

    // Write special slot hashes (in reverse order: slot 5 first, then 4, 3, 2, 1)
    // Special slots are at negative offsets from hash_offset
    if has_entitlements {
        // Slots 1-4 are empty (zeros), slot 5 is entitlements
        for slot in (1..=n_special_slots).rev() {
            if slot == CSSLOT_ENTITLEMENTS {
                sig[offset..offset + 32].copy_from_slice(&entitlements_hash);
            }
            // zeros are already in place from initialization
            offset += 32;
        }
    }

    // Calculate and write page hashes
    let mut hasher = Sha256::new();
    let mut data_offset = 0;
    while data_offset < codesig_data_offset {
        let end = core::cmp::min(data_offset + CS_PAGE_SIZE, codesig_data_offset);
        hasher.update(&data[data_offset..end]);
        let hash: [u8; 32] = hasher.finalize_reset().into();
        sig[offset..offset + 32].copy_from_slice(&hash);
        offset += 32;
        data_offset = end;
    }

    // Write entitlements blob (if present)
    if has_entitlements {
        sig.gwrite_with(CSMAGIC_EMBEDDED_ENTITLEMENTS, &mut offset, BE)
            .map_err(|e| error::Error::Malformed(e.to_string()))?;
        sig.gwrite_with(entitlements_blob_size as u32, &mut offset, BE)
            .map_err(|e| error::Error::Malformed(e.to_string()))?;
        sig[offset..offset + entitlements_data.len()].copy_from_slice(entitlements_data);
    }

    // Padding is already in place from initialization with zeros

    // Resize and write signature
    data.resize(codesig_data_offset + padded_sig_size, 0);
    data[codesig_data_offset..].copy_from_slice(&sig);

    Ok(data)
}

/// Sign a Mach-O binary with an ad-hoc signature
///
/// This function handles the complete flow of ad-hoc signing:
/// 1. Parse the binary to find code signature and segment information
/// 2. Generate a new ad-hoc signature with the specified identifier
/// 3. Update the load commands and write the new signature
///
/// # Arguments
/// * `data` - The Mach-O binary data
/// * `options` - Signing options (identifier, hardened runtime, entitlements)
///
/// # Returns
/// The signed binary data, or an error if signing failed
///
/// # Example
/// ```ignore
/// use arwen_codesign::{adhoc_sign, AdhocSignOptions, Entitlements};
///
/// // Simple signing with just an identifier
/// let signed = adhoc_sign(data, &AdhocSignOptions::new("com.example.myapp"))?;
///
/// // With hardened runtime and preserved entitlements
/// let options = AdhocSignOptions::new("com.example.myapp")
///     .with_hardened_runtime()
///     .with_entitlements(Entitlements::Preserve);
/// let signed = adhoc_sign(data, &options)?;
///
/// // With custom entitlements
/// let entitlements_plist = b"<?xml version=\"1.0\"?>...";
/// let options = AdhocSignOptions {
///     identifier: "com.example.myapp",
///     hardened_runtime: true,
///     entitlements: Entitlements::Custom(entitlements_plist),
/// };
/// let signed = adhoc_sign(data, &options)?;
/// ```
pub fn adhoc_sign(data: Vec<u8>, options: &AdhocSignOptions) -> error::Result<Vec<u8>> {
    // Resolve entitlements based on the option
    let entitlements: Option<Vec<u8>> = match &options.entitlements {
        Entitlements::None => None,
        Entitlements::Preserve => extract_entitlements(&data),
        Entitlements::Custom(ent_data) => Some(ent_data.to_vec()),
    };

    // Parse load commands
    let info = parse_macho_load_info(&data)?;

    let codesig_cmd_offset = info
        .codesig_cmd_offset
        .ok_or_else(|| error::Error::Malformed("No LC_CODE_SIGNATURE found".into()))?;
    let linkedit_cmd_offset = info
        .linkedit_cmd_offset
        .ok_or_else(|| error::Error::Malformed("No __LINKEDIT segment found".into()))?;

    generate_adhoc_signature(
        data,
        options.identifier,
        codesig_cmd_offset,
        info.codesig_data_offset,
        linkedit_cmd_offset,
        info.linkedit_fileoff,
        info.text_fileoff,
        info.text_filesize,
        info.is_64bit,
        info.is_executable,
        options.hardened_runtime,
        options.linker_signed,
        entitlements.as_deref(),
    )
}

/// Sign a Mach-O binary file with an ad-hoc signature (file-based API)
///
/// This function uses streaming I/O to avoid loading the entire binary into memory.
/// It writes to a temporary file and atomically replaces the original.
///
/// # Arguments
/// * `path` - Path to the Mach-O binary file
/// * `options` - Signing options (identifier, hardened runtime, entitlements)
///
/// # Returns
/// Ok(()) on success, or an error if signing failed
///
/// # Example
/// ```ignore
/// use arwen_codesign::{adhoc_sign_file, AdhocSignOptions, Entitlements};
/// use std::path::Path;
///
/// let options = AdhocSignOptions::new("com.example.myapp")
///     .with_entitlements(Entitlements::Preserve);
/// adhoc_sign_file(Path::new("/path/to/binary"), &options)?;
/// ```
/// Helper to convert goblin errors to io errors
fn to_io_error(e: error::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

/// Helper to convert scroll errors to io errors
fn scroll_to_io_error(e: scroll::Error) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
}

pub fn adhoc_sign_file(path: &std::path::Path, options: &AdhocSignOptions) -> std::io::Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};

    // Open the input file
    let input_file = std::fs::File::open(path)?;
    let file_size = input_file.metadata()?.len() as usize;
    let mut reader = BufReader::new(input_file);

    // Read the header to determine architecture and get basic info
    // We need at least the header (32 bytes for 64-bit, 28 for 32-bit)
    let mut header_buf = [0u8; 32];
    reader.read_exact(&mut header_buf)?;
    reader.seek(SeekFrom::Start(0))?;

    // Parse magic to determine if 64-bit
    let (_, ctx_opt) = parse_magic_and_ctx(&header_buf, 0).map_err(to_io_error)?;
    let ctx = ctx_opt.ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid Mach-O magic")
    })?;
    let is_64bit = ctx.container == container::Container::Big;
    let header_size = Header::size_with(&ctx);

    // Read the full header
    let header: Header = header_buf.pread_with(0, ctx).map_err(to_io_error)?;

    // Calculate size needed for header + all load commands
    let load_commands_size = header.sizeofcmds as usize;
    let header_and_cmds_size = header_size + load_commands_size;

    // Read header + load commands
    let mut header_and_cmds = vec![0u8; header_and_cmds_size];
    reader.read_exact(&mut header_and_cmds)?;

    // Parse load commands to find what we need
    let mut codesig_cmd_offset = None;
    let mut codesig_data_offset = 0usize;
    let mut codesig_data_size = 0usize;
    let mut linkedit_cmd_offset = None;
    let mut linkedit_fileoff = 0u64;
    let mut text_fileoff = 0u64;
    let mut text_filesize = 0u64;

    let mut offset = header_size;
    for _ in 0..header.ncmds {
        let cmd: u32 = header_and_cmds
            .pread_with(offset, ctx.le)
            .map_err(scroll_to_io_error)?;
        let cmdsize: u32 = header_and_cmds
            .pread_with(offset + 4, ctx.le)
            .map_err(scroll_to_io_error)?;

        if cmd == LC_CODE_SIGNATURE {
            codesig_cmd_offset = Some(offset);
            let dataoff: u32 = header_and_cmds
                .pread_with(offset + 8, ctx.le)
                .map_err(scroll_to_io_error)?;
            let datasize: u32 = header_and_cmds
                .pread_with(offset + 12, ctx.le)
                .map_err(scroll_to_io_error)?;
            codesig_data_offset = dataoff as usize;
            codesig_data_size = datasize as usize;
        } else if cmd == LC_SEGMENT_64 {
            let segname_bytes = &header_and_cmds[offset + 8..offset + 24];
            let segname = core::str::from_utf8(segname_bytes)
                .unwrap_or("")
                .trim_end_matches('\0');

            if segname == "__LINKEDIT" {
                linkedit_cmd_offset = Some(offset);
                linkedit_fileoff = header_and_cmds
                    .pread_with(offset + 32 + 8, ctx.le)
                    .map_err(scroll_to_io_error)?;
            } else if segname == "__TEXT" {
                text_fileoff = header_and_cmds
                    .pread_with(offset + 32 + 8, ctx.le)
                    .map_err(scroll_to_io_error)?;
                text_filesize = header_and_cmds
                    .pread_with(offset + 32 + 16, ctx.le)
                    .map_err(scroll_to_io_error)?;
            }
        } else if cmd == LC_SEGMENT {
            let segname_bytes = &header_and_cmds[offset + 8..offset + 24];
            let segname = core::str::from_utf8(segname_bytes)
                .unwrap_or("")
                .trim_end_matches('\0');

            if segname == "__LINKEDIT" {
                linkedit_cmd_offset = Some(offset);
                linkedit_fileoff = header_and_cmds
                    .pread_with::<u32>(offset + 28 + 4, ctx.le)
                    .map_err(scroll_to_io_error)? as u64;
            } else if segname == "__TEXT" {
                text_fileoff = header_and_cmds
                    .pread_with::<u32>(offset + 28 + 4, ctx.le)
                    .map_err(scroll_to_io_error)? as u64;
                text_filesize = header_and_cmds
                    .pread_with::<u32>(offset + 28 + 8, ctx.le)
                    .map_err(scroll_to_io_error)? as u64;
            }
        }

        offset += cmdsize as usize;
    }

    let codesig_cmd_offset = codesig_cmd_offset.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No LC_CODE_SIGNATURE found",
        )
    })?;
    let linkedit_cmd_offset = linkedit_cmd_offset.ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No __LINKEDIT segment found",
        )
    })?;

    // Extract existing entitlements if preserving
    let entitlements: Option<Vec<u8>> = match &options.entitlements {
        Entitlements::None => None,
        Entitlements::Custom(data) => Some(data.to_vec()),
        Entitlements::Preserve => {
            // Read the existing code signature blob to extract entitlements
            if codesig_data_size > 0 && codesig_data_offset + codesig_data_size <= file_size {
                reader.seek(SeekFrom::Start(codesig_data_offset as u64))?;
                let mut sig_blob = vec![0u8; codesig_data_size];
                reader.read_exact(&mut sig_blob)?;
                extract_entitlements_from_superblob(&sig_blob)
            } else {
                None
            }
        }
    };

    // Check if this is a main executable (MH_EXECUTE = 2)
    let is_executable = header.filetype == 2;

    // Calculate new signature parameters
    let id_bytes = options.identifier.as_bytes();
    let id_len = id_bytes.len() + 1;
    let n_hashes = codesig_data_offset.div_ceil(CS_PAGE_SIZE);

    let has_entitlements = entitlements.is_some();
    let entitlements_data = entitlements.as_deref().unwrap_or(&[]);
    let entitlements_blob_size = if has_entitlements {
        8 + entitlements_data.len()
    } else {
        0
    };

    let blob_count = if has_entitlements { 2 } else { 1 };
    let n_special_slots: u32 = if has_entitlements {
        CSSLOT_ENTITLEMENTS
    } else {
        0
    };

    let superblob_size = 12;
    let blob_indices_size = blob_count * 8;
    let codedir_size = 88;

    let special_hashes_size = n_special_slots as usize * 32;
    let code_hashes_size = n_hashes * 32;

    let hash_offset = codedir_size + id_len + special_hashes_size;
    let codedir_total = codedir_size + id_len + special_hashes_size + code_hashes_size;

    let blob_content_size =
        superblob_size + blob_indices_size + codedir_total + entitlements_blob_size;
    let padded_sig_size = (blob_content_size + 7) & !7;

    // Update the header+cmds buffer with new values
    // Update LC_CODE_SIGNATURE datasize
    let datasize_offset = codesig_cmd_offset + 12;
    header_and_cmds[datasize_offset..datasize_offset + 4]
        .copy_from_slice(&(padded_sig_size as u32).to_le_bytes());

    // Update __LINKEDIT segment filesize
    let new_linkedit_filesize =
        codesig_data_offset as u64 + padded_sig_size as u64 - linkedit_fileoff;
    if is_64bit {
        let filesize_offset = linkedit_cmd_offset + 48;
        header_and_cmds[filesize_offset..filesize_offset + 8]
            .copy_from_slice(&new_linkedit_filesize.to_le_bytes());
    } else {
        let filesize_offset = linkedit_cmd_offset + 36;
        header_and_cmds[filesize_offset..filesize_offset + 4]
            .copy_from_slice(&(new_linkedit_filesize as u32).to_le_bytes());
    }

    // Create temp file in the same directory for atomic rename
    let parent_dir = path.parent().unwrap_or(std::path::Path::new("."));
    let mut temp_file = tempfile::NamedTempFile::new_in(parent_dir)?;
    let mut writer = BufWriter::new(&mut temp_file);

    // Write modified header + load commands
    writer.write_all(&header_and_cmds)?;

    // Stream copy the rest of the binary up to code signature, computing hashes
    reader.seek(SeekFrom::Start(header_and_cmds_size as u64))?;

    let mut page_hashes = Vec::with_capacity(n_hashes * 32);
    let mut hasher = Sha256::new();
    let mut bytes_written = header_and_cmds_size;
    let mut page_buf = vec![0u8; CS_PAGE_SIZE];

    // Hash the header+cmds we already have
    let mut hash_offset_in_file = 0;
    while hash_offset_in_file < header_and_cmds_size && hash_offset_in_file < codesig_data_offset {
        let page_end = core::cmp::min(hash_offset_in_file + CS_PAGE_SIZE, codesig_data_offset);
        let page_end_in_buf = core::cmp::min(page_end, header_and_cmds_size);

        if hash_offset_in_file < header_and_cmds_size {
            hasher.update(&header_and_cmds[hash_offset_in_file..page_end_in_buf]);
        }

        if page_end <= header_and_cmds_size {
            // Full page in header+cmds
            page_hashes.extend_from_slice(&hasher.finalize_reset());
        }

        hash_offset_in_file = page_end;
    }

    // Continue streaming the rest of the file
    while bytes_written < codesig_data_offset {
        let to_read = core::cmp::min(CS_PAGE_SIZE, codesig_data_offset - bytes_written);
        let bytes_read = reader.read(&mut page_buf[..to_read])?;
        if bytes_read == 0 {
            break;
        }

        writer.write_all(&page_buf[..bytes_read])?;

        // Update hash for current page
        hasher.update(&page_buf[..bytes_read]);
        bytes_written += bytes_read;

        // Check if we completed a page
        let current_page_start = (bytes_written - bytes_read) / CS_PAGE_SIZE * CS_PAGE_SIZE;
        let current_page_end =
            core::cmp::min(current_page_start + CS_PAGE_SIZE, codesig_data_offset);

        if bytes_written >= current_page_end || bytes_written >= codesig_data_offset {
            page_hashes.extend_from_slice(&hasher.finalize_reset());
        }
    }

    // Build the signature blob
    let codedir_offset_in_sig = superblob_size + blob_indices_size;
    let entitlements_offset_in_sig = codedir_offset_in_sig + codedir_total;

    // Calculate entitlements hash if needed
    let entitlements_hash: [u8; 32] = if has_entitlements {
        let mut ent_blob = vec![0u8; entitlements_blob_size];
        let mut ent_offset = 0usize;
        ent_blob
            .gwrite_with(CSMAGIC_EMBEDDED_ENTITLEMENTS, &mut ent_offset, BE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        ent_blob
            .gwrite_with(entitlements_blob_size as u32, &mut ent_offset, BE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        ent_blob[ent_offset..].copy_from_slice(entitlements_data);

        let mut hasher = Sha256::new();
        hasher.update(&ent_blob);
        hasher.finalize().into()
    } else {
        [0u8; 32]
    };

    // Build signature - pre-allocate and use gwrite_with
    let mut sig = vec![0u8; padded_sig_size];
    let mut sig_offset = 0usize;

    // SuperBlob header
    let superblob = SuperBlob {
        magic: CSMAGIC_EMBEDDED_SIGNATURE,
        length: blob_content_size as u32,
        count: blob_count as u32,
    };
    sig.gwrite_with(superblob, &mut sig_offset, BE)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // BlobIndex for CodeDirectory
    let codedir_index = BlobIndex {
        typ: CSSLOT_CODEDIRECTORY,
        offset: codedir_offset_in_sig as u32,
    };
    sig.gwrite_with(codedir_index, &mut sig_offset, BE)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // BlobIndex for entitlements (if present)
    if has_entitlements {
        let ent_index = BlobIndex {
            typ: CSSLOT_ENTITLEMENTS,
            offset: entitlements_offset_in_sig as u32,
        };
        sig.gwrite_with(ent_index, &mut sig_offset, BE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    }

    // CodeDirectory
    let mut flags = CS_ADHOC;
    if options.linker_signed {
        flags |= CS_LINKER_SIGNED;
    }
    if options.hardened_runtime {
        flags |= CS_RUNTIME;
    }

    let codedir = CodeDirectory {
        magic: CSMAGIC_CODEDIRECTORY,
        length: codedir_total as u32,
        version: CS_VERSION,
        flags,
        hash_offset: hash_offset as u32,
        ident_offset: codedir_size as u32,
        n_special_slots,
        n_code_slots: n_hashes as u32,
        code_limit: codesig_data_offset as u32,
        hash_size: 32,
        hash_type: CS_HASHTYPE_SHA256,
        _pad1: 0,
        page_size: CS_PAGE_SIZE_LOG2,
        _pad2: 0,
        scatter_offset: 0,
        team_offset: 0,
        _pad3: 0,
        code_limit64: 0,
        exec_seg_base: text_fileoff,
        exec_seg_limit: text_filesize,
        exec_seg_flags: if is_executable {
            CS_EXECSEG_MAIN_BINARY
        } else {
            0
        },
    };
    sig.gwrite_with(codedir, &mut sig_offset, BE)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    // Identifier
    sig[sig_offset..sig_offset + id_bytes.len()].copy_from_slice(id_bytes);
    sig_offset += id_bytes.len();
    sig[sig_offset] = 0; // Null terminator
    sig_offset += 1;

    // Special slot hashes (in reverse order)
    if has_entitlements {
        for slot in (1..=n_special_slots).rev() {
            if slot == CSSLOT_ENTITLEMENTS {
                sig[sig_offset..sig_offset + 32].copy_from_slice(&entitlements_hash);
            }
            // zeros are already in place from initialization
            sig_offset += 32;
        }
    }

    // Code page hashes
    sig[sig_offset..sig_offset + page_hashes.len()].copy_from_slice(&page_hashes);
    sig_offset += page_hashes.len();

    // Entitlements blob (if present)
    if has_entitlements {
        sig.gwrite_with(CSMAGIC_EMBEDDED_ENTITLEMENTS, &mut sig_offset, BE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        sig.gwrite_with(entitlements_blob_size as u32, &mut sig_offset, BE)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        sig[sig_offset..sig_offset + entitlements_data.len()].copy_from_slice(entitlements_data);
    }

    // Padding is already in place from initialization with zeros

    // Write signature
    writer.write_all(&sig)?;
    writer.flush()?;
    drop(writer);

    // Atomically replace the original file
    temp_file.persist(path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pread_be_u32() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF, 0x12, 0x34];
        assert_eq!(data.pread_with::<u32>(0, BE).unwrap(), 0xDEADBEEF);
        assert_eq!(data.pread_with::<u32>(2, BE).unwrap(), 0xBEEF1234);
    }

    #[test]
    fn test_parse_segment_name() {
        let bytes = b"__TEXT\0\0\0\0\0\0\0\0\0\0";
        assert_eq!(parse_segment_name(bytes), "__TEXT");

        let bytes = b"__LINKEDIT\0\0\0\0\0\0";
        assert_eq!(parse_segment_name(bytes), "__LINKEDIT");
    }

    #[test]
    fn test_adhoc_sign_options_builder() {
        let opts = AdhocSignOptions::new("com.example.test");
        assert_eq!(opts.identifier, "com.example.test");
        assert!(!opts.hardened_runtime);
        assert!(!opts.linker_signed);

        let opts = AdhocSignOptions::new("com.example.test")
            .with_hardened_runtime()
            .with_linker_signed();
        assert!(opts.hardened_runtime);
        assert!(opts.linker_signed);
    }

    #[test]
    fn test_superblob_iter() {
        // Valid SuperBlob with 2 entries
        let mut data = Vec::new();
        // Magic
        data.extend_from_slice(&CSMAGIC_EMBEDDED_SIGNATURE.to_be_bytes());
        // Length (will be updated)
        data.extend_from_slice(&0u32.to_be_bytes());
        // Count = 2
        data.extend_from_slice(&2u32.to_be_bytes());
        // BlobIndex 1: type=0, offset=28
        data.extend_from_slice(&0u32.to_be_bytes());
        data.extend_from_slice(&28u32.to_be_bytes());
        // BlobIndex 2: type=2, offset=36
        data.extend_from_slice(&2u32.to_be_bytes());
        data.extend_from_slice(&36u32.to_be_bytes());

        let iter = iter_superblob(&data).unwrap();
        let entries: Vec<_> = iter.collect();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].blob_type, 0);
        assert_eq!(entries[0].blob_offset, 28);
        assert_eq!(entries[1].blob_type, 2);
        assert_eq!(entries[1].blob_offset, 36);
    }

    #[test]
    fn test_superblob_iter_invalid_magic() {
        let data = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(iter_superblob(&data).is_none());
    }

    #[test]
    fn test_superblob_iter_too_short() {
        let data = [0xFA, 0xDE, 0x0C, 0xC0]; // Just the magic, too short
        assert!(iter_superblob(&data).is_none());
    }
}

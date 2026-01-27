#!/usr/bin/env python3
"""
Analyze and compare Mach-O code signatures in detail.

This script parses code signature structures (SuperBlob, CodeDirectory, etc.)
and can compare two binaries to identify exact differences.

Usage:
    python analyze_codesign.py <binary>              # Analyze single binary
    python analyze_codesign.py <binary1> <binary2>  # Compare two binaries
    python analyze_codesign.py --help
"""

import argparse
import struct
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional

# Magic numbers
CSMAGIC_EMBEDDED_SIGNATURE = 0xFADE0CC0
CSMAGIC_CODEDIRECTORY = 0xFADE0C02
CSMAGIC_REQUIREMENTS = 0xFADE0C01
CSMAGIC_EMBEDDED_ENTITLEMENTS = 0xFADE7171
CSMAGIC_EMBEDDED_ENTITLEMENTS_DER = 0xFADE7172
CSMAGIC_BLOBWRAPPER = 0xFADE0B01

# Slot types
CSSLOT_CODEDIRECTORY = 0
CSSLOT_INFOSLOT = 1
CSSLOT_REQUIREMENTS = 2
CSSLOT_RESOURCEDIR = 3
CSSLOT_APPLICATION = 4
CSSLOT_ENTITLEMENTS = 5
CSSLOT_ENTITLEMENTS_DER = 7
CSSLOT_ALTERNATE_CODEDIRECTORIES = 0x1000

# Flags
CS_ADHOC = 0x0002
CS_RUNTIME = 0x10000
CS_LINKER_SIGNED = 0x20000

# Mach-O constants
MH_MAGIC_64 = 0xFEEDFACF
MH_CIGAM_64 = 0xCFFAEDFE
MH_MAGIC = 0xFEEDFACE
MH_CIGAM = 0xCEFAEDFE
FAT_MAGIC = 0xCAFEBABE
FAT_CIGAM = 0xBEBAFECA

LC_CODE_SIGNATURE = 0x1D


def magic_name(magic: int) -> str:
    """Convert magic number to human-readable name."""
    names = {
        CSMAGIC_EMBEDDED_SIGNATURE: "CSMAGIC_EMBEDDED_SIGNATURE",
        CSMAGIC_CODEDIRECTORY: "CSMAGIC_CODEDIRECTORY",
        CSMAGIC_REQUIREMENTS: "CSMAGIC_REQUIREMENTS",
        CSMAGIC_EMBEDDED_ENTITLEMENTS: "CSMAGIC_EMBEDDED_ENTITLEMENTS",
        CSMAGIC_EMBEDDED_ENTITLEMENTS_DER: "CSMAGIC_EMBEDDED_ENTITLEMENTS_DER",
        CSMAGIC_BLOBWRAPPER: "CSMAGIC_BLOBWRAPPER",
    }
    return names.get(magic, f"0x{magic:08x}")


def slot_name(slot: int) -> str:
    """Convert slot index to human-readable name."""
    names = {
        CSSLOT_CODEDIRECTORY: "CodeDirectory",
        CSSLOT_INFOSLOT: "Info.plist",
        CSSLOT_REQUIREMENTS: "Requirements",
        CSSLOT_RESOURCEDIR: "ResourceDir",
        CSSLOT_APPLICATION: "Application",
        CSSLOT_ENTITLEMENTS: "Entitlements",
        CSSLOT_ENTITLEMENTS_DER: "EntitlementsDER",
    }
    if slot >= CSSLOT_ALTERNATE_CODEDIRECTORIES:
        return f"AltCodeDir[{slot - CSSLOT_ALTERNATE_CODEDIRECTORIES}]"
    return names.get(slot, f"Slot[{slot}]")


def flags_str(flags: int) -> str:
    """Convert flags to human-readable string."""
    parts = []
    if flags & CS_ADHOC:
        parts.append("adhoc")
    if flags & CS_RUNTIME:
        parts.append("runtime")
    if flags & CS_LINKER_SIGNED:
        parts.append("linker-signed")
    return f"0x{flags:x}({','.join(parts)})" if parts else f"0x{flags:x}"


@dataclass
class BlobEntry:
    """Entry in a SuperBlob's blob index."""
    slot_type: int
    offset: int
    magic: int = 0
    length: int = 0
    data: bytes = field(default_factory=bytes, repr=False)


@dataclass
class CodeDirectory:
    """Parsed CodeDirectory structure."""
    magic: int
    length: int
    version: int
    flags: int
    hash_offset: int
    ident_offset: int
    n_special_slots: int
    n_code_slots: int
    code_limit: int
    hash_size: int
    hash_type: int
    spare1: int
    page_size: int
    spare2: int
    scatter_offset: int = 0
    team_offset: int = 0
    spare3: int = 0
    code_limit_64: int = 0
    exec_seg_base: int = 0
    exec_seg_limit: int = 0
    exec_seg_flags: int = 0
    identifier: str = ""
    team_id: str = ""
    special_hashes: list = field(default_factory=list)
    code_hashes: list = field(default_factory=list)


@dataclass
class SignatureInfo:
    """Complete code signature information."""
    offset: int
    size: int
    superblob_magic: int = 0
    superblob_length: int = 0
    blob_count: int = 0
    blobs: list = field(default_factory=list)
    code_directory: Optional[CodeDirectory] = None
    requirements_data: bytes = field(default_factory=bytes, repr=False)
    entitlements_plist: bytes = field(default_factory=bytes, repr=False)


def read_be_u32(data: bytes, offset: int) -> int:
    """Read big-endian u32."""
    return struct.unpack(">I", data[offset:offset+4])[0]


def read_be_u64(data: bytes, offset: int) -> int:
    """Read big-endian u64."""
    return struct.unpack(">Q", data[offset:offset+8])[0]


def find_code_signature(data: bytes) -> tuple[int, int]:
    """Find LC_CODE_SIGNATURE in a Mach-O binary.

    Returns (offset, size) of the code signature data.
    """
    if len(data) < 8:
        return 0, 0

    # Check magic
    magic = struct.unpack("<I", data[0:4])[0]

    if magic == MH_MAGIC_64 or magic == MH_CIGAM_64:
        is_64 = True
        header_size = 32
    elif magic == MH_MAGIC or magic == MH_CIGAM:
        is_64 = False
        header_size = 28
    elif magic == FAT_MAGIC or magic == FAT_CIGAM:
        # Fat binary - just use the first slice for analysis
        nfat = struct.unpack(">I", data[4:8])[0]
        if nfat > 0:
            # First slice offset
            slice_offset = struct.unpack(">I", data[16:20])[0]
            return find_code_signature(data[slice_offset:])
        return 0, 0
    else:
        return 0, 0

    # Determine endianness
    if magic in (MH_CIGAM_64, MH_CIGAM):
        le = ">"  # Big-endian
    else:
        le = "<"  # Little-endian

    # Read header
    ncmds = struct.unpack(f"{le}I", data[16:20])[0]

    # Scan load commands
    offset = header_size
    for _ in range(ncmds):
        cmd = struct.unpack(f"{le}I", data[offset:offset+4])[0]
        cmdsize = struct.unpack(f"{le}I", data[offset+4:offset+8])[0]

        if cmd == LC_CODE_SIGNATURE:
            dataoff = struct.unpack(f"{le}I", data[offset+8:offset+12])[0]
            datasize = struct.unpack(f"{le}I", data[offset+12:offset+16])[0]
            return dataoff, datasize

        offset += cmdsize

    return 0, 0


def parse_code_directory(data: bytes, offset: int) -> Optional[CodeDirectory]:
    """Parse a CodeDirectory blob."""
    if offset + 44 > len(data):
        return None

    magic = read_be_u32(data, offset)
    if magic != CSMAGIC_CODEDIRECTORY:
        return None

    length = read_be_u32(data, offset + 4)
    version = read_be_u32(data, offset + 8)
    flags = read_be_u32(data, offset + 12)
    hash_offset = read_be_u32(data, offset + 16)
    ident_offset = read_be_u32(data, offset + 20)
    n_special_slots = read_be_u32(data, offset + 24)
    n_code_slots = read_be_u32(data, offset + 28)
    code_limit = read_be_u32(data, offset + 32)
    hash_size = data[offset + 36]
    hash_type = data[offset + 37]
    spare1 = data[offset + 38]
    page_size = data[offset + 39]
    spare2 = read_be_u32(data, offset + 40)

    cd = CodeDirectory(
        magic=magic,
        length=length,
        version=version,
        flags=flags,
        hash_offset=hash_offset,
        ident_offset=ident_offset,
        n_special_slots=n_special_slots,
        n_code_slots=n_code_slots,
        code_limit=code_limit,
        hash_size=hash_size,
        hash_type=hash_type,
        spare1=spare1,
        page_size=page_size,
        spare2=spare2,
    )

    # Version 0x20100+ has scatter_offset
    if version >= 0x20100 and offset + 48 <= len(data):
        cd.scatter_offset = read_be_u32(data, offset + 44)

    # Version 0x20200+ has team_offset
    if version >= 0x20200 and offset + 52 <= len(data):
        cd.team_offset = read_be_u32(data, offset + 48)

    # Version 0x20300+ has spare3
    if version >= 0x20300 and offset + 56 <= len(data):
        cd.spare3 = read_be_u32(data, offset + 52)

    # Version 0x20400+ has code_limit_64
    if version >= 0x20400 and offset + 64 <= len(data):
        cd.code_limit_64 = read_be_u64(data, offset + 56)

    # Version 0x20500+ has exec_seg fields
    if version >= 0x20500 and offset + 88 <= len(data):
        cd.exec_seg_base = read_be_u64(data, offset + 64)
        cd.exec_seg_limit = read_be_u64(data, offset + 72)
        cd.exec_seg_flags = read_be_u64(data, offset + 80)

    # Extract identifier
    if ident_offset > 0 and offset + ident_offset < len(data):
        end = data.find(b'\x00', offset + ident_offset)
        if end > 0:
            cd.identifier = data[offset + ident_offset:end].decode('utf-8', errors='replace')

    # Extract team ID
    if cd.team_offset > 0 and offset + cd.team_offset < len(data):
        end = data.find(b'\x00', offset + cd.team_offset)
        if end > 0:
            cd.team_id = data[offset + cd.team_offset:end].decode('utf-8', errors='replace')

    # Extract special slot hashes (stored before code hashes, in reverse order)
    hash_start = offset + hash_offset - (n_special_slots * hash_size)
    for i in range(n_special_slots):
        h_offset = hash_start + i * hash_size
        if h_offset + hash_size <= len(data):
            cd.special_hashes.append(data[h_offset:h_offset + hash_size])

    # Extract code hashes
    code_hash_start = offset + hash_offset
    for i in range(min(n_code_slots, 10)):  # Limit to first 10 for display
        h_offset = code_hash_start + i * hash_size
        if h_offset + hash_size <= len(data):
            cd.code_hashes.append(data[h_offset:h_offset + hash_size])

    return cd


def parse_signature(data: bytes) -> Optional[SignatureInfo]:
    """Parse the code signature from binary data."""
    sig_offset, sig_size = find_code_signature(data)
    if sig_offset == 0 or sig_size == 0:
        return None

    if sig_offset + sig_size > len(data):
        return None

    sig_data = data[sig_offset:sig_offset + sig_size]

    info = SignatureInfo(offset=sig_offset, size=sig_size)

    # Parse SuperBlob header
    if len(sig_data) < 12:
        return info

    info.superblob_magic = read_be_u32(sig_data, 0)
    info.superblob_length = read_be_u32(sig_data, 4)
    info.blob_count = read_be_u32(sig_data, 8)

    if info.superblob_magic != CSMAGIC_EMBEDDED_SIGNATURE:
        return info

    # Parse blob index
    for i in range(info.blob_count):
        idx_offset = 12 + i * 8
        if idx_offset + 8 > len(sig_data):
            break

        slot_type = read_be_u32(sig_data, idx_offset)
        blob_offset = read_be_u32(sig_data, idx_offset + 4)

        entry = BlobEntry(slot_type=slot_type, offset=blob_offset)

        if blob_offset + 8 <= len(sig_data):
            entry.magic = read_be_u32(sig_data, blob_offset)
            entry.length = read_be_u32(sig_data, blob_offset + 4)
            if blob_offset + entry.length <= len(sig_data):
                entry.data = sig_data[blob_offset:blob_offset + entry.length]

        info.blobs.append(entry)

        # Parse specific blobs
        if slot_type == CSSLOT_CODEDIRECTORY:
            info.code_directory = parse_code_directory(sig_data, blob_offset)
        elif slot_type == CSSLOT_REQUIREMENTS:
            if blob_offset + entry.length <= len(sig_data):
                info.requirements_data = sig_data[blob_offset:blob_offset + entry.length]
        elif slot_type == CSSLOT_ENTITLEMENTS:
            if blob_offset + 8 + entry.length <= len(sig_data):
                info.entitlements_plist = sig_data[blob_offset + 8:blob_offset + entry.length]

    return info


def print_signature_info(info: SignatureInfo, name: str = "", verbose: bool = False):
    """Print code signature information."""
    if name:
        print(f"\n{'=' * 60}")
        print(f"Signature Analysis: {name}")
        print('=' * 60)

    print(f"\nSignature Location:")
    print(f"  Offset: 0x{info.offset:x} ({info.offset})")
    print(f"  Size:   {info.size} bytes")

    print(f"\nSuperBlob:")
    print(f"  Magic:  {magic_name(info.superblob_magic)}")
    print(f"  Length: {info.superblob_length}")
    print(f"  Count:  {info.blob_count} blobs")

    print(f"\nBlob Index:")
    for i, blob in enumerate(info.blobs):
        print(f"  [{i}] {slot_name(blob.slot_type):20} offset=0x{blob.offset:x} "
              f"magic={magic_name(blob.magic)} length={blob.length}")

    if info.code_directory:
        cd = info.code_directory
        print(f"\nCodeDirectory:")
        print(f"  Version:         0x{cd.version:x}")
        print(f"  Flags:           {flags_str(cd.flags)}")
        print(f"  Identifier:      {cd.identifier}")
        if cd.team_id:
            print(f"  Team ID:         {cd.team_id}")
        print(f"  Hash Type:       {cd.hash_type} ({'SHA-256' if cd.hash_type == 2 else 'SHA-1' if cd.hash_type == 1 else 'unknown'})")
        print(f"  Hash Size:       {cd.hash_size}")
        print(f"  Page Size:       {1 << cd.page_size} (2^{cd.page_size})")
        print(f"  Code Limit:      {cd.code_limit}")
        if cd.code_limit_64:
            print(f"  Code Limit 64:   {cd.code_limit_64}")
        print(f"  Special Slots:   {cd.n_special_slots}")
        print(f"  Code Slots:      {cd.n_code_slots}")

        if cd.version >= 0x20500:
            print(f"  Exec Seg Base:   0x{cd.exec_seg_base:x}")
            print(f"  Exec Seg Limit:  0x{cd.exec_seg_limit:x}")
            print(f"  Exec Seg Flags:  0x{cd.exec_seg_flags:x}")

        if verbose and cd.special_hashes:
            print(f"\n  Special Slot Hashes (negative slots, stored in reverse):")
            for i, h in enumerate(cd.special_hashes):
                slot_idx = cd.n_special_slots - i
                is_zero = all(b == 0 for b in h)
                hash_str = h.hex() if not is_zero else "(zero)"
                print(f"    Slot -{slot_idx}: {hash_str}")

        if verbose and cd.code_hashes:
            print(f"\n  Code Hashes (first {len(cd.code_hashes)}):")
            for i, h in enumerate(cd.code_hashes):
                print(f"    Page {i}: {h.hex()[:32]}...")

    if info.requirements_data:
        print(f"\nRequirements Blob: {len(info.requirements_data)} bytes")

    if info.entitlements_plist:
        print(f"\nEntitlements: {len(info.entitlements_plist)} bytes")
        if verbose:
            try:
                print(info.entitlements_plist.decode('utf-8'))
            except:
                pass


def compare_signatures(info1: SignatureInfo, info2: SignatureInfo,
                       name1: str = "Binary 1", name2: str = "Binary 2"):
    """Compare two code signatures and print differences."""
    print(f"\n{'=' * 60}")
    print(f"Comparison: {name1} vs {name2}")
    print('=' * 60)

    differences = []

    # Compare sizes
    if info1.size != info2.size:
        differences.append(f"Signature size: {info1.size} vs {info2.size} (diff: {info2.size - info1.size})")

    # Compare blob counts
    if info1.blob_count != info2.blob_count:
        differences.append(f"Blob count: {info1.blob_count} vs {info2.blob_count}")

    # Compare blobs
    slots1 = {b.slot_type: b for b in info1.blobs}
    slots2 = {b.slot_type: b for b in info2.blobs}

    all_slots = set(slots1.keys()) | set(slots2.keys())
    for slot in sorted(all_slots):
        b1 = slots1.get(slot)
        b2 = slots2.get(slot)

        if b1 is None:
            differences.append(f"Slot {slot_name(slot)}: missing in {name1}")
        elif b2 is None:
            differences.append(f"Slot {slot_name(slot)}: missing in {name2}")
        elif b1.length != b2.length:
            differences.append(f"Slot {slot_name(slot)}: length {b1.length} vs {b2.length}")
        elif b1.data != b2.data:
            # Find first diff
            for i in range(min(len(b1.data), len(b2.data))):
                if b1.data[i] != b2.data[i]:
                    differences.append(f"Slot {slot_name(slot)}: first diff at offset {i}")
                    break

    # Compare CodeDirectory details
    cd1 = info1.code_directory
    cd2 = info2.code_directory

    if cd1 and cd2:
        if cd1.version != cd2.version:
            differences.append(f"CodeDirectory version: 0x{cd1.version:x} vs 0x{cd2.version:x}")
        if cd1.flags != cd2.flags:
            differences.append(f"CodeDirectory flags: {flags_str(cd1.flags)} vs {flags_str(cd2.flags)}")
        if cd1.n_special_slots != cd2.n_special_slots:
            differences.append(f"Special slots: {cd1.n_special_slots} vs {cd2.n_special_slots}")
        if cd1.n_code_slots != cd2.n_code_slots:
            differences.append(f"Code slots: {cd1.n_code_slots} vs {cd2.n_code_slots}")
        if cd1.identifier != cd2.identifier:
            differences.append(f"Identifier: '{cd1.identifier}' vs '{cd2.identifier}'")

        # Compare special hashes
        if cd1.n_special_slots == cd2.n_special_slots:
            for i in range(min(len(cd1.special_hashes), len(cd2.special_hashes))):
                h1 = cd1.special_hashes[i]
                h2 = cd2.special_hashes[i]
                if h1 != h2:
                    slot_idx = cd1.n_special_slots - i
                    differences.append(f"Special slot -{slot_idx} hash differs")

    if differences:
        print("\nDifferences found:")
        for d in differences:
            print(f"  - {d}")
    else:
        print("\nNo structural differences found!")

    return len(differences) == 0


def main():
    parser = argparse.ArgumentParser(
        description="Analyze and compare Mach-O code signatures"
    )
    parser.add_argument("binary1", type=Path, help="First binary to analyze")
    parser.add_argument("binary2", type=Path, nargs="?", help="Second binary to compare (optional)")
    parser.add_argument("-v", "--verbose", action="store_true", help="Show detailed hash information")

    args = parser.parse_args()

    if not args.binary1.exists():
        print(f"Error: {args.binary1} not found")
        sys.exit(1)

    data1 = args.binary1.read_bytes()
    info1 = parse_signature(data1)

    if info1 is None:
        print(f"Error: Could not parse signature from {args.binary1}")
        sys.exit(1)

    if args.binary2:
        if not args.binary2.exists():
            print(f"Error: {args.binary2} not found")
            sys.exit(1)

        data2 = args.binary2.read_bytes()
        info2 = parse_signature(data2)

        if info2 is None:
            print(f"Error: Could not parse signature from {args.binary2}")
            sys.exit(1)

        # Print both signatures
        print_signature_info(info1, args.binary1.name, args.verbose)
        print_signature_info(info2, args.binary2.name, args.verbose)

        # Compare
        compare_signatures(info1, info2, args.binary1.name, args.binary2.name)
    else:
        print_signature_info(info1, args.binary1.name, args.verbose)


if __name__ == "__main__":
    main()

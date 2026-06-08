use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use iced_x86::code_asm::*;

/// Patches the target DLL to change its entry point to `mov rax, 1; ret`
/// and forces the IMAGE_FILE_DLL characteristic bit.
pub fn patch_dll(read_path: &Path, write_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut read_file = OpenOptions::new().read(true).open(read_path)?;
    let mut write_file = OpenOptions::new().write(true).open(write_path)?;
    std::io::copy(&mut read_file, &mut write_file)?;

    let mut e_lfanew_buf = [0u8; 4];
    read_file.seek(SeekFrom::Start(0x3C))?;
    read_file.read_exact(&mut e_lfanew_buf)?;

    let file_header_offset = u32::from_le_bytes(e_lfanew_buf) as u64;
    let optional_header_offset = file_header_offset + 0x18;

    let bitness = {
        let mut buf = [0u8; 2];
        read_file.seek(SeekFrom::Start(file_header_offset + 0x04))?;
        read_file.read_exact(&mut buf)?;
        match u16::from_le_bytes(buf) {
            0x014C => 32,
            0x8664 => 64,
            _ => panic!(),
        }
    };

    let characteristics = {
        let mut buf = [0u8; 2];
        read_file.seek(SeekFrom::Start(file_header_offset + 0x16))?;
        read_file.read_exact(&mut buf)?;
        u16::from_le_bytes(buf)
    };

    let optional_header_size = {
        let mut buf = [0u8; 2];
        read_file.seek(SeekFrom::Start(file_header_offset + 0x14))?;
        read_file.read_exact(&mut buf)?;
        u16::from_le_bytes(buf) as u64
    };

    let entry_point_rva = {
        let mut buf = [0u8; 4];
        read_file.seek(SeekFrom::Start(optional_header_offset + 0x10))?;
        read_file.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    let section_table_offset = optional_header_offset + optional_header_size;

    // Assume that our relative virtual address is in `.text`, which is the first section.
    let text_section_offset = section_table_offset + (0 * 0x28);

    let mut buf = [0u8; 4];

    let _virtual_size = {
        read_file.seek(SeekFrom::Start(text_section_offset + 0x08))?;
        read_file.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    let virtual_addr = {
        read_file.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    let raw_data_size = {
        read_file.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    let raw_data_pointer = {
        read_file.read_exact(&mut buf)?;
        u32::from_le_bytes(buf)
    };

    assert!(entry_point_rva >= virtual_addr);
    assert!(entry_point_rva < virtual_addr + raw_data_size);

    let entry_point_file_offset = entry_point_rva
        .wrapping_add(raw_data_pointer)
        .wrapping_sub(virtual_addr);

    // x86_64 assembly: mov rax, 1; ret
    let patch_x64_data = {
        let mut assembler = CodeAssembler::new(bitness)?;
        assembler.mov(eax, 0x01)?;
        assembler.ret()?;
        assembler.assemble(entry_point_file_offset.into())?
    };

    write_file.seek(SeekFrom::Start(entry_point_file_offset as u64))?;
    write_file.write_all(&patch_x64_data)?;

    let new_charactertics = characteristics | 0x2000; // Sets IMAGE_FILE_DLL bit to 1.
    write_file.seek(SeekFrom::Start(file_header_offset + 0x16))?;
    write_file.write_all(&new_charactertics.to_le_bytes())?;
    Ok(())
}

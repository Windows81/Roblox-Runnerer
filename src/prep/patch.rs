use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use iced_x86::code_asm::*;

const IMAGE_FILE_DLL: u16 = 0x2000;

fn read_u16(f: &mut File, at: u64) -> std::io::Result<u16> {
    let mut buf = [0u8; 2];
    f.seek(SeekFrom::Start(at))?;
    f.read_exact(&mut buf)?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32(f: &mut File, at: u64) -> std::io::Result<u32> {
    let mut buf = [0u8; 4];
    f.seek(SeekFrom::Start(at))?;
    f.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

/// Copies the exe to `dll_name`, stubs its entry point with `mov eax, 1; ret`
/// and flips on IMAGE_FILE_DLL so LoadLibrary will take it.
///
/// Returns the RVA of the original entry point.
pub fn patch_dll(
    dir: &Path,
    exe_name: &str,
    dll_name: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut exe = File::open(dir.join(exe_name))?;
    let mut dll = File::create(dir.join(dll_name))?;
    std::io::copy(&mut exe, &mut dll)?;

    // e_lfanew lives at 0x3C in the DOS header
    let file_header = read_u32(&mut exe, 0x3C)? as u64;
    let opt_header = file_header + 0x18;

    let bitness = match read_u16(&mut exe, file_header + 0x04)? {
        0x014C => 32,
        0x8664 => 64,
        machine => return Err(format!("unknown machine type {machine:#06x}").into()),
    };
    if bitness != usize::BITS {
        return Err("Bitness does not match; LoadLibrary does not allow cross-arch usage.".into());
    }

    let characteristics = read_u16(&mut exe, file_header + 0x16)?;
    let opt_header_size = read_u16(&mut exe, file_header + 0x14)? as u64;
    let entry_rva = read_u32(&mut exe, opt_header + 0x10)?;

    // Assume the entry point is in .text, which is the first section.
    let text = opt_header + opt_header_size;
    let virtual_addr = read_u32(&mut exe, text + 0x0C)?;
    let raw_size = read_u32(&mut exe, text + 0x10)?;
    let raw_ptr = read_u32(&mut exe, text + 0x14)?;

    if entry_rva < virtual_addr || entry_rva >= virtual_addr + raw_size {
        return Err("Entry point is outside the first PE section".into());
    }

    let entry_offset = entry_rva - virtual_addr + raw_ptr;

    let stub = {
        let mut a = CodeAssembler::new(bitness)?;
        a.mov(eax, 1)?;
        a.ret()?;
        a.assemble(entry_offset.into())?
    };
    dll.seek(SeekFrom::Start(entry_offset as u64))?;
    dll.write_all(&stub)?;

    dll.seek(SeekFrom::Start(file_header + 0x16))?;
    dll.write_all(&(characteristics | IMAGE_FILE_DLL).to_le_bytes())?;

    Ok(entry_rva as _)
}

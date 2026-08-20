use std::collections::HashMap;

use crate::reader::DataReader;
use serde::Serialize;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
mod reader;

#[derive(Serialize)]
pub struct ElfBinary {
    header: Header,
    program_header_table: Vec<ProgramHeader>,
    section_header_table: Vec<SectionHeader>,
}

pub const MAGIC_HEADER: u32 = 0x464c457F; // header is read as little-endian and because of this reversed

#[derive(Serialize)]
pub enum ElfType {
    None,
    Rel,
    Exec,
    Dyn,
    Core,
    Os(u16),
    Proc(u16),
}

#[derive(Serialize)]
pub enum Machine {
    None,
    X86_64,
}

#[derive(Serialize)]
pub struct Header {
    identification: Identification,
    elf_type: ElfType,
    machine: Machine,
    version: u32,
    entry: u64,
    program_header_offset: u64,
    section_header_offset: u64,
    flags: u32,
    header_size: u16,
    program_header_entry_size: u16,
    program_header_num: u16,
    section_header_entry_size: u16,
    section_header_num: u16,
    section_header_string_table_index: u16,
}

#[derive(Serialize)]
pub enum Class {
    None,
    Class32,
    Class64,
}

#[derive(Serialize)]
pub enum DataEncoding {
    None,
    LSB,
    MSB,
}

#[derive(Serialize)]
pub struct Identification {
    class: Class,
    data_encoding: DataEncoding,
    version: u8,
    os_abi: u8,
    abi_version: u8,
}

#[derive(Serialize)]
pub struct ProgramHeader {
    p_type: SegmentType,
    flags: u32,
    offset: u64,
    virtual_address: u64,
    physical_address: u64,
    file_size: u64,
    mem_size: u64,
    align: u64,
}

#[derive(Serialize)]
pub enum SegmentType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    ShLib,
    PhHdr,
    Tls,
    Num,
    Os(u32),
    Proc(u32),
}

#[repr(u64)]
#[derive(Serialize, Copy, Clone)]
pub enum SectionFlag {
    Write,
    Alloc,
    ExecInstr,
    Merge,
    Strings,
    InfoLink,
    LinkOrder,
    OsNonConforming,
    Group,
    TLS,
    Compressed,
    MaskOs(u64),
    MaskProc(u64),
}

#[derive(Serialize)]
pub struct SectionHeader {
    name_index: u32,
    name: String,
    s_type: SectionType,
    flags: Vec<SectionFlag>,
    virtual_address: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    address_align: u64,
    entry_size: u64,
}

#[derive(Serialize, PartialEq)]
pub enum SectionType {
    Null,
    ProgBits,
    SymTab,
    StrTab,
    Rela,
    Hash,
    Dynamic,
    Note,
    NoBits,
    Rel,
    ShLib,
    DynSym,
    InitArray,
    FiniArray,
    PreinitArray,
    Group,
    SymTabIndex,
    Relr,
    Os(u32),
    Proc(u32),
    User(u32),
}

pub fn has_elf_magic_header(data: &[u8]) -> bool {
    let mut reader = DataReader::new(data);
    parse_magic_header(&mut reader).is_ok()
}

pub fn parse(data: &[u8]) -> Result<ElfBinary, String> {
    let mut reader = DataReader::new(data);
    let header = parse_header(&mut reader)?;
    let program_header_table = parse_program_header_table(&mut reader, &header)?;
    let mut section_header_table = parse_section_header_table(&mut reader, &header)?;

    let section_str_tab = parse_str_tab(
        &mut reader,
        &section_header_table[header.section_header_string_table_index as usize],
    )?;
    patch_section_names(&mut section_header_table, &section_str_tab)?;

    Ok(ElfBinary {
        header,
        program_header_table,
        section_header_table,
    })
}

pub fn parse_header(reader: &mut DataReader) -> Result<Header, String> {
    parse_magic_header(reader)?;

    let identification = parse_identification(reader)?;

    let elf_type = parse_elf_type(reader.read_u16())?;
    let machine = parse_machine(reader.read_u16())?;
    let version = reader.read_u32();
    let entry = reader.read_u64();
    let program_header_offset = reader.read_u64();
    let section_header_offset = reader.read_u64();
    let flags = reader.read_u32();
    let header_size = reader.read_u16();
    let program_header_entry_size = reader.read_u16();
    let program_header_num = reader.read_u16();
    let section_header_entry_size = reader.read_u16();
    let section_header_num = reader.read_u16();
    let section_header_string_table_index = reader.read_u16();

    Ok(Header {
        identification,
        elf_type,
        machine,
        version,
        entry,
        program_header_offset,
        section_header_offset,
        flags,
        header_size,
        program_header_entry_size,
        program_header_num,
        section_header_entry_size,
        section_header_num,
        section_header_string_table_index,
    })
}

fn parse_magic_header(reader: &mut DataReader) -> Result<(), String> {
    let magic = reader.read_u32();
    if magic != MAGIC_HEADER {
        return Err("not an elf file".to_string());
    }
    Ok(())
}

fn parse_identification(reader: &mut DataReader) -> Result<Identification, String> {
    let class = parse_class(reader.read_u8())?;
    let data_encoding = parse_data_encoding(reader.read_u8())?;
    let version = reader.read_u8();
    let os_abi = reader.read_u8();
    let abi_version = reader.read_u8();
    reader.skip(7); // reserved identification bytes

    Ok(Identification {
        class,
        data_encoding,
        version,
        os_abi,
        abi_version,
    })
}

fn parse_data_encoding(v: u8) -> Result<DataEncoding, String> {
    let data_encoding = match v {
        0 => DataEncoding::None,
        1 => DataEncoding::LSB,
        2 => DataEncoding::MSB,
        _ => return Err(format!("unknown data_encoding: 0x{:x}", v)),
    };
    Ok(data_encoding)
}

fn parse_class(v: u8) -> Result<Class, String> {
    let class = match v {
        0 => Class::None,
        1 => Class::Class32,
        2 => Class::Class64,
        _ => return Err(format!("unknown class: 0x{:x}", v)),
    };
    Ok(class)
}

fn parse_elf_type(v: u16) -> Result<ElfType, String> {
    let t = match v {
        0 => ElfType::None,
        1 => ElfType::Rel,
        2 => ElfType::Exec,
        3 => ElfType::Dyn,
        4 => ElfType::Core,
        0xFE00..=0xFEFF => ElfType::Os(v),
        0xFF00..=0xFFFF => ElfType::Proc(v),
        _ => return Err(format!("unknown e_type: 0x{:x}", v)),
    };
    Ok(t)
}

fn parse_machine(v: u16) -> Result<Machine, String> {
    let m = match v {
        0 => Machine::None,
        62 => Machine::X86_64,
        _ => return Err(format!("unknown e_machine: 0x{:x}", v)),
    };
    Ok(m)
}

fn parse_program_header_table(
    reader: &mut DataReader,
    header: &Header,
) -> Result<Vec<ProgramHeader>, String> {
    reader.reset_offset(header.program_header_offset as usize);

    let mut result = Vec::with_capacity(header.program_header_num as usize);
    for _ in 0..header.program_header_num {
        let p_type = parse_segment_type(reader.read_u32())?;
        let flags = reader.read_u32();
        let offset = reader.read_u64();
        let virtual_address = reader.read_u64();
        let physical_address = reader.read_u64();
        let file_size = reader.read_u64();
        let mem_size = reader.read_u64();
        let align = reader.read_u64();

        result.push(ProgramHeader {
            p_type,
            flags,
            offset,
            virtual_address,
            physical_address,
            file_size,
            mem_size,
            align,
        })
    }

    return Ok(result);
}

fn parse_segment_type(v: u32) -> Result<SegmentType, String> {
    let m = match v {
        0 => SegmentType::Null,
        1 => SegmentType::Load,
        2 => SegmentType::Dynamic,
        3 => SegmentType::Interp,
        4 => SegmentType::Note,
        5 => SegmentType::ShLib,
        6 => SegmentType::PhHdr,
        7 => SegmentType::Num,
        0x60000000..=0x6fffffff => SegmentType::Os(v),
        0x70000000..=0x7fffffff => SegmentType::Proc(v),
        _ => return Err(format!("unknown segment_type: 0x{:x}", v)),
    };
    Ok(m)
}

fn parse_section_header_table(
    reader: &mut DataReader,
    header: &Header,
) -> Result<Vec<SectionHeader>, String> {
    reader.reset_offset(header.section_header_offset as usize);

    let mut result = Vec::with_capacity(header.section_header_num as usize);
    for _ in 0..header.section_header_num {
        let name_index = reader.read_u32();
        let name = "".to_string();
        let s_type = parse_section_type(reader.read_u32())?;
        let flags = parse_section_flags(reader.read_u64())?;
        let virtual_address = reader.read_u64();
        let offset = reader.read_u64();
        let size = reader.read_u64();
        let link = reader.read_u32();
        let info = reader.read_u32();
        let address_align = reader.read_u64();
        let entry_size = reader.read_u64();

        result.push(SectionHeader {
            s_type,
            name_index,
            name,
            flags,
            virtual_address,
            offset,
            size,
            link,
            info,
            address_align,
            entry_size,
        });
    }

    Ok(result)
}

fn parse_section_flags(v: u64) -> Result<Vec<SectionFlag>, String> {
    let mut result = Vec::new();
    let r = &mut result;
    s_flag(v, SectionFlag::Write, r);
    s_flag(v, SectionFlag::Alloc, r);
    s_flag(v, SectionFlag::ExecInstr, r);
    s_flag(v, SectionFlag::Merge, r);
    s_flag(v, SectionFlag::Strings, r);
    s_flag(v, SectionFlag::InfoLink, r);
    s_flag(v, SectionFlag::LinkOrder, r);
    s_flag(v, SectionFlag::OsNonConforming, r);
    s_flag(v, SectionFlag::Group, r);
    s_flag(v, SectionFlag::TLS, r);
    s_flag(v, SectionFlag::Compressed, r);
    s_flag(v, SectionFlag::MaskOs(0), r);
    s_flag(v, SectionFlag::MaskProc(0), r);
    Ok(result)
}

fn s_flag(v: u64, flag: SectionFlag, result: &mut Vec<SectionFlag>) {
    if (v & section_flag_to_u64(flag)) != 0 {
        result.push(u64_to_section_flag(v, flag))
    }
}

// adds the proper tuple values
fn u64_to_section_flag(v: u64, flag: SectionFlag) -> SectionFlag {
    match flag {
        SectionFlag::MaskOs(_) => SectionFlag::MaskOs(v),
        SectionFlag::MaskProc(_) => SectionFlag::MaskProc(v),
        other => other,
    }
}

fn section_flag_to_u64(flag: SectionFlag) -> u64 {
    match flag {
        SectionFlag::Write => 0x01,
        SectionFlag::Alloc => 0x02,
        SectionFlag::ExecInstr => 0x04,
        SectionFlag::Merge => 0x10,
        SectionFlag::Strings => 0x20,
        SectionFlag::InfoLink => 0x40,
        SectionFlag::LinkOrder => 0x80,
        SectionFlag::OsNonConforming => 0x100,
        SectionFlag::Group => 0x200,
        SectionFlag::TLS => 0x400,
        SectionFlag::Compressed => 0x800,
        SectionFlag::MaskOs(_) => 0x0ff00000,
        SectionFlag::MaskProc(_) => 0xf0000000,
    }
}

fn parse_section_type(v: u32) -> Result<SectionType, String> {
    let m = match v {
        0 => SectionType::Null,
        1 => SectionType::ProgBits,
        2 => SectionType::SymTab,
        3 => SectionType::StrTab,
        4 => SectionType::Rela,
        5 => SectionType::Hash,
        6 => SectionType::Dynamic,
        7 => SectionType::Note,
        8 => SectionType::NoBits,
        9 => SectionType::Rel,
        10 => SectionType::ShLib,
        11 => SectionType::DynSym,
        14 => SectionType::InitArray,
        15 => SectionType::FiniArray,
        16 => SectionType::PreinitArray,
        17 => SectionType::Group,
        18 => SectionType::SymTabIndex,
        19 => SectionType::Relr,
        0x60000000..=0x6fffffff => SectionType::Os(v),
        0x70000000..=0x7fffffff => SectionType::Proc(v),
        0x80000000..=0x8fffffff => SectionType::User(v),
        _ => return Err(format!("unknown section_type: 0x{:x}", v)),
    };
    Ok(m)
}

fn parse_str_tab(
    reader: &mut DataReader,
    section: &SectionHeader,
) -> Result<HashMap<usize, String>, String> {
    if section.s_type != SectionType::StrTab {
        return Err("parse_str_tab of non StrTab section".to_string());
    }
    if reader.read_u8_at(section.offset as usize) != 0 {
        return Err("StrTab does not start with \\0 byte".to_string());
    }

    let str_tab_start = section.offset + 1; // +1 to skip the leading \0
    reader.reset_offset(str_tab_start as usize);

    let mut result = HashMap::new();
    let mut str_start = 1;
    for i in 1..section.size {
        let tab_offset = (section.offset + i) as usize;
        if reader.read_u8_at(tab_offset) == 0 {
            let str = reader.read_utf8_string(tab_offset - (section.offset as usize + str_start));
            reader.skip(1); // \0 byte
            result.insert(str_start, str);
            str_start = i as usize + 1;
        }
    }

    Ok(result)
}

fn patch_section_names(
    section_headers: &mut Vec<SectionHeader>,
    str_tab: &HashMap<usize, String>,
) -> Result<(), String> {
    for header in section_headers {
        if header.s_type == SectionType::Null {
            continue;
        }

        let may_section_name = str_tab.get(&(header.name_index as usize));
        if let Some(section_name) = may_section_name {
            header.name = section_name.clone();
        }
    }
    Ok(())
}

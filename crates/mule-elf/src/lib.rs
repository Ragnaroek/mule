use crate::reader::DataReader;
use serde::Serialize;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
mod reader;

#[derive(Serialize)]
pub struct Elf {
    header: Header,
    program_header_table: Vec<ProgramHeader>,
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

pub fn parse(data: &[u8]) -> Result<Elf, String> {
    let mut reader = DataReader::new(data);
    let header = parse_header(&mut reader)?;
    let program_header_table = parse_program_header_table(&mut reader, &header)?;
    Ok(Elf {
        header,
        program_header_table,
    })
}

pub fn parse_header(reader: &mut DataReader) -> Result<Header, String> {
    let magic = reader.read_u32();
    if magic != MAGIC_HEADER {
        return Err("not an elf file".to_string());
    }

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

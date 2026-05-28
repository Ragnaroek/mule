use crate::reader::DataReader;

#[cfg(test)]
#[path = "./lib_test.rs"]
mod lib_test;
mod reader;

pub struct Elf {
    header: Header,
}

pub const MAGIC_HEADER: u32 = 0x464c457F; // header is read as little-endian and because of this reversed

pub struct Header {}

pub fn parse(data: &[u8]) -> Result<Elf, String> {
    let mut reader = DataReader::new(data);
    let header = parse_header(&mut reader)?;
    Ok(Elf { header })
}

pub fn parse_header(reader: &mut DataReader) -> Result<Header, String> {
    let magic = reader.read_u32();
    println!("## magic = {:x}", magic);
    if magic != MAGIC_HEADER {
        return Err("not a elf file".to_string());
    }

    Ok(Header {})
}

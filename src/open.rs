use macho::{self, Macho};

use std::fs;
use std::io::Read;
use std::{fs::File, path::Path};

pub enum BinaryFile {
    Macho(Macho),
}

pub fn open_binary_file(path: &Path) -> Result<BinaryFile, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut magic = [0; 4];
    file.read(&mut magic).map_err(|e| e.to_string())?;
    let magic_u32 = u32::from_le_bytes(magic);
    if magic_u32 == macho::MAGIC_HEADER {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let macho_file = macho::load(&data)?;
        Ok(BinaryFile::Macho(macho_file))
    } else {
        Err("file not supported".to_string())
    }
}

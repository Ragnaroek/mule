use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::{fs::File, path::Path};

use mule_gb::GBBinary;
use mule_macho::Macho;

pub enum BinaryFile {
    Macho(Macho),
    GB(GBBinary),
}

pub fn open_binary_file(path: &Path) -> Result<BinaryFile, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut magic = [0; 4];
    file.read(&mut magic).map_err(|e| e.to_string())?;
    let magic_u32 = u32::from_le_bytes(magic);
    if magic_u32 == mule_macho::MAGIC_HEADER {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let macho_file = mule_macho::load(&data)?;
        return Ok(BinaryFile::Macho(macho_file));
    }

    let extension = path.extension().and_then(OsStr::to_str);

    if extension == Some("gb") || extension == Some("gbc") {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        let gb_file = mule_gb::load(&data)?;
        return Ok(BinaryFile::GB(gb_file));
    }

    Err("file not supported".to_string())
}

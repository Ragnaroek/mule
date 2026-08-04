use std::fs;

use crate::parse;

#[test]
fn test_parse() -> Result<(), String> {
    let test_binary = fs::read("testdata/kernel").expect("test binary");
    let elf = parse(&test_binary)?;
    Ok(())
}

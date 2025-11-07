use crate::DATA_PATH;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn read_file(file_name: impl AsRef<str>) -> anyhow::Result<Vec<u8>> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    // Read file content
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_file_str(file_name: impl AsRef<str>) -> anyhow::Result<String> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    // Read file content as string
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_file(file_name: impl AsRef<str>, content: &[u8]) -> anyhow::Result<()> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    let mut file = std::fs::File::create(path)?;
    file.write_all(content)?;
    Ok(())
}

pub fn write_file_str(file_name: impl AsRef<str>, content: Vec<String>) -> anyhow::Result<()> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    let mut out = std::fs::File::create(path)?;
    for line in content {
        out.write_all(line.as_bytes())?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

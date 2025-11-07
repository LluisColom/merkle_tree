use crate::DATA_PATH;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn read_file_str(file_name: impl AsRef<str>) -> anyhow::Result<String> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    // Read file content as string
    Ok(std::fs::read_to_string(path)?)
}

pub fn write_file(file_name: impl AsRef<str>, content: &[u8]) -> anyhow::Result<()> {
    let path = PathBuf::from(DATA_PATH).join(file_name.as_ref());
    let mut file = std::fs::File::create(path)?;
    file.write_all(content)
        .map_err(|e| anyhow::anyhow!("Error writing file: {}", e))?;
    Ok(())
}

pub fn read_doc(i: usize) -> anyhow::Result<Vec<u8>> {
    let path = PathBuf::from(DATA_PATH).join(format!("doc{}.dat", i));
    // Read file content
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| anyhow::anyhow!("Error reading file: {}", e))?;
    Ok(buf)
}

pub fn read_node(i: usize, j: usize) -> anyhow::Result<Option<Vec<u8>>> {
    let path = PathBuf::from(DATA_PATH).join(format!("node{}.{}.dat", i, j));
    // Read file content
    let mut file = std::fs::File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| anyhow::anyhow!("Error reading file: {}", e))?;
    Ok(Some(buf))
}

pub fn write_node(i: usize, j: usize, data: &[u8]) -> anyhow::Result<()> {
    let path = PathBuf::from(DATA_PATH).join(format!("node{}.{}.dat", i, j));
    // Write contents to the file
    let mut file = std::fs::File::create(path)?;
    file.write_all(data)
        .map_err(|e| anyhow::anyhow!("Error writing file: {}", e))?;
    Ok(())
}

pub fn write_summary(content: Vec<String>) -> anyhow::Result<()> {
    let path = PathBuf::from(DATA_PATH).join("summary.txt");
    let mut out = std::fs::File::create(path)?;
    for line in content {
        out.write_all(line.as_bytes())?;
        out.write_all(b"\n")?;
    }
    Ok(())
}

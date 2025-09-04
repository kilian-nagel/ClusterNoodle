use std::fs;
use std::io::Error;

pub fn path_exists(path: &str) -> Result<bool, Error> {
    let metadata = fs::metadata(path)?;
    assert!(metadata.is_file());
    Ok(true)
}

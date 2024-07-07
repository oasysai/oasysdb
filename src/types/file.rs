use super::err::{Error, ErrorCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// Reads a binary file and deserializes its contents to a type.
/// - `path`: Path to the binary file.
pub fn read_binary_file<T: DeserializeOwned>(
    path: impl AsRef<Path>,
) -> Result<T, Error> {
    let file = OpenOptions::new().read(true).open(path)?;
    let reader = BufReader::new(file);
    let value = bincode::deserialize_from(reader)?;
    Ok(value)
}

/// Serializes the data and writes it to a binary file.
/// - `path`: Path to the binary file.
/// - `data`: Data to write.
pub fn write_binary_file<T: Serialize>(
    path: impl AsRef<Path>,
    data: &T,
) -> Result<(), Error> {
    let file_name = parse_file_name(&path)?;
    let tmp_dir = get_tmp_dir()?;

    // To ensure that the target file is not corrupted if
    // the operation is interrupted or fails:
    // 1. Write the data to a temporary file.
    // 2. Rename the temporary file to the target file.

    let tmp_file = tmp_dir.join(file_name);
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_file)?;

    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, data)?;

    fs::rename(&tmp_file, &path)?;
    Ok(())
}

/// Returns the temporary directory path for OasysDB.
pub fn get_tmp_dir() -> Result<PathBuf, Error> {
    let tmp_dir = env::temp_dir().join("oasysdb");
    if !tmp_dir.try_exists()? {
        fs::create_dir_all(&tmp_dir)?;
    }

    Ok(tmp_dir)
}

/// Parses the file name from a path.
/// - `path`: Path to a file.
pub fn parse_file_name(path: impl AsRef<Path>) -> Result<String, Error> {
    let file_name = path.as_ref().file_name().ok_or_else(|| {
        let code = ErrorCode::FileError;
        let message = "Unable to parse the file name from the path.";
        Error::new(code, message)
    })?;

    Ok(file_name.to_string_lossy().to_string())
}

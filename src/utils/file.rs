use crate::types::err::{Error, ErrorCode};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::Path;

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
/// - `tmp_dir`: Temporary directory path.
/// - `path`: Path to the binary file.
/// - `data`: Data to write.
///
/// This function writes the data to a temporary file first and then renames
/// the temporary file to the target file. This ensures that the target file
/// is not corrupted if the operation is interrupted or fails.
pub fn write_binary_file<T: Serialize>(
    tmp_dir: impl AsRef<Path>,
    path: impl AsRef<Path>,
    data: &T,
) -> Result<(), Error> {
    let file_name = parse_file_name(&path)?;
    let tmp_file = tmp_dir.as_ref().join(file_name);
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

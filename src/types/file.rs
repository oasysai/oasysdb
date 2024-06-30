use super::error::{Error, ErrorCode};
use arrow::array::RecordBatch;
use arrow::ipc::reader::FileReader;
use arrow::ipc::writer::FileWriter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::cmp::min;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

/// A utility struct for reading and writing files.
pub struct FileOps {
    tmp_dir: PathBuf,
}

impl Default for FileOps {
    fn default() -> Self {
        let tmp_dir = env::temp_dir().join("oasysdb");
        Self::new(tmp_dir)
    }
}

impl FileOps {
    pub fn new(tmp_dir: PathBuf) -> Self {
        if !tmp_dir.exists() {
            fs::create_dir_all(&tmp_dir)
                .expect("Unable to create a temporary directory.")
        }

        Self { tmp_dir }
    }

    /// Reads a binary file and deserialize it into a type.
    pub fn read_binary_file<T: DeserializeOwned>(
        &self,
        path: &PathBuf,
    ) -> Result<T, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        bincode::deserialize_from(reader).map_err(Into::into)
    }

    /// Serializes a type and write it to a binary file.
    ///
    /// The file is written to a temporary file first, then renamed
    /// to make sure that the file is not corrupted if the operation fails.
    pub fn write_binary_file<T: Serialize>(
        &self,
        path: &PathBuf,
        data: &T,
    ) -> Result<(), Error> {
        let filename = self.parse_file_name(path)?;

        // Write the data to a temporary file first.
        // If this fails, the original file will not be overwritten.
        let tmp_path = self.tmp_dir.join(filename);
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&tmp_path)?;

        let writer = BufWriter::new(file);
        bincode::serialize_into(writer, data)?;

        // If the serialization is successful, rename the temporary file.
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    pub fn read_ipc_file(&self, path: &PathBuf) -> Result<RecordBatch, Error> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);
        let ipc_reader = FileReader::try_new(reader, None)?;
        let schema = ipc_reader.schema();

        // In OasyDB, there will be only one record batch per file.
        let record_batch = match ipc_reader.last() {
            Some(batch) => batch?,
            _ => RecordBatch::new_empty(schema),
        };

        Ok(record_batch)
    }

    pub fn write_ipc_files(
        &self,
        paths: &[PathBuf],
        data: &RecordBatch,
        batch_size: usize,
    ) -> Result<(), Error> {
        let create_tmp_path = |path: &PathBuf| {
            let filename = self.parse_file_name(path).unwrap();
            self.tmp_dir.join(filename)
        };

        let tmp_paths: Vec<PathBuf> =
            paths.iter().map(create_tmp_path).collect();

        let schema = data.schema();

        for (i, tmp_path) in tmp_paths.iter().enumerate() {
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(tmp_path)?;

            let writer = BufWriter::new(file);
            let mut ipc_writer = FileWriter::try_new(writer, &schema)?;

            // This attempts to write the record batch in chunks.
            // This is useful when the record batch is larger than
            // the predefined batch size.
            let batch = {
                let offset = i * batch_size;
                let length = min(batch_size, data.num_rows() - offset);
                data.slice(offset, length)
            };

            // Write the record batch to the file.
            ipc_writer.write(&batch)?;
            ipc_writer.finish()?;
        }

        // If the serialization is successful, rename the temporary file.
        for i in 0..tmp_paths.len() {
            fs::rename(&tmp_paths[i], &paths[i])?;
        }

        Ok(())
    }

    /// Parses a file name from a path.
    pub fn parse_file_name(&self, path: &PathBuf) -> Result<String, Error> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .ok_or_else(|| {
                let code = ErrorCode::FileError;
                let message = format!("Invalid file name from path: {path:?}");
                Error::new(&code, &message)
            })
    }
}

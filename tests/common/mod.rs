use byteorder::{LittleEndian, ReadBytesExt};
use curl::easy::Easy;
use flate2::read::GzDecoder;
use sqlx::any::install_default_drivers;
use sqlx::{AnyConnection, Connection, Executor, Row};
use std::env;
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tar::Archive;

/// Type of benchmark dataset to use.
/// - `SIFTSMALL`: SIFT small dataset (10k vectors of 128D).
/// - `SIFT`: SIFT dataset (1000k vectors of 128D).
/// - `GIST`: GIST dataset (1M vectors of 960D).
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub enum Dataset {
    #[default]
    SIFTSMALL,
    SIFT,
    GIST,
}

impl Dataset {
    /// Returns the name of the dataset in lowercase.
    pub fn name(&self) -> &str {
        match self {
            Dataset::SIFTSMALL => "siftsmall",
            Dataset::SIFT => "sift",
            Dataset::GIST => "gist",
        }
    }

    /// Returns the number of vectors in the dataset.
    pub fn size(&self) -> usize {
        match self {
            Dataset::SIFTSMALL => 10_000,
            Dataset::SIFT => 1_000_000,
            Dataset::GIST => 1_000_000,
        }
    }

    /// Returns OasysDB SQLite database URL for testing.
    pub fn database_url(&self) -> String {
        let path = self.tmp_dir().join("sqlite.db");
        format!("sqlite://{}?mode=rwc", path.display())
    }

    /// Populates the test SQL database with the benchmark dataset.
    pub async fn populate_database(&self) -> Result<(), Box<dyn Error>> {
        install_default_drivers();
        self.setup().await?;

        let db_url = self.database_url();
        let mut conn = AnyConnection::connect(&db_url).await?;

        let table_name = self.name();
        let tables = {
            let query = "SELECT name FROM sqlite_master WHERE type = 'table'";
            conn.fetch_all(query).await?
        };

        // If the dataset table already exists, return early since the next
        // operation is computationally expensive and not needed.
        if tables.iter().any(|row| row.get::<&str, usize>(0) == table_name) {
            return Ok(());
        }

        let create_table = format!(
            "CREATE TABLE IF NOT EXISTS {table_name} (
                id INTEGER PRIMARY KEY,
                vector JSON NOT NULL
            )"
        );

        conn.execute(create_table.as_ref()).await?;

        let dataset = self.base_dataset_file();
        let vectors = self.read_vectors(dataset)?;
        let mut insert_vector = format!(
            "INSERT INTO {table_name} (vector)
            VALUES"
        );

        for vector in vectors.iter() {
            let value = serde_json::to_string(vector)?;
            insert_vector.push_str(&format!("\n({value:?}),"));
        }

        insert_vector = insert_vector.trim_end_matches(',').to_string();
        conn.execute(insert_vector.as_ref()).await?;

        // Verify that the vectors were inserted correctly.
        let count = {
            let query = format!("SELECT COUNT(*) FROM {table_name}");
            conn.fetch_one(query.as_ref()).await?.get::<i64, usize>(0)
        };

        assert_eq!(count, self.size() as i64);
        Ok(())
    }

    /// Downloads and extracts the dataset to a directory.
    async fn setup(&self) -> Result<(), Box<dyn Error>> {
        if !self.compressed_file().try_exists()? {
            self.download().await?;
        }

        if !self.base_dataset_file().try_exists()? {
            self.extract()?;
        }

        Ok(())
    }

    /// Downloads the benchmark dataset from the server.
    async fn download(&self) -> Result<(), Box<dyn Error>> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.compressed_file())?;

        let mut easy = Easy::new();
        easy.url(&self.download_url())?;

        let mut writer = BufWriter::new(file);
        easy.write_function(move |data| {
            writer.write_all(data).unwrap();
            Ok(data.len())
        })?;

        easy.perform()?;
        Ok(())
    }

    /// Extracts the dataset from the compressed file.
    fn extract(&self) -> Result<(), Box<dyn Error>> {
        let path = self.compressed_file();
        let file = OpenOptions::new().read(true).open(path)?;
        let mut archive = Archive::new(GzDecoder::new(file));
        archive.unpack(self.tmp_dir())?;
        Ok(())
    }

    /// Reads the vectors from the dataset file.
    /// - `path`: Path to the fvecs file.
    pub fn read_vectors(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let file = OpenOptions::new().read(true).open(path)?;
        let mut reader = BufReader::new(file);

        let dimension = reader.read_i32::<LittleEndian>()? as usize;
        let vector_size = 4 + dimension * 4;

        let n = reader.seek(SeekFrom::End(0))? as usize / vector_size;
        reader.seek(SeekFrom::Start(((0) * vector_size) as u64))?;

        let mut vectors = vec![vec![0f32; n]; dimension];
        for i in 0..n {
            for j in 0..dimension {
                vectors[j][i] = reader.read_f32::<LittleEndian>()?;
            }
        }

        // Transpose the vector.
        let rows = vectors.len();
        let cols = vectors[0].len();
        let vectors = (0..cols)
            .map(|col| (0..rows).map(|row| vectors[row][col]).collect())
            .collect();

        Ok(vectors)
    }

    /// Returns the URL to download the dataset.
    fn download_url(&self) -> String {
        let base_url = "ftp://ftp.irisa.fr/local/texmex/corpus/";
        let file = format!("{}.tar.gz", self.name());
        format!("{base_url}/{file}")
    }

    /// Returns the path to the compressed file.
    fn compressed_file(&self) -> PathBuf {
        self.tmp_dir().join(format!("{}.tar.gz", self.name()))
    }

    /// Returns the path to the dataset file.
    pub fn base_dataset_file(&self) -> PathBuf {
        self.tmp_dir()
            .join(self.name())
            .join(format!("{}_base.fvecs", self.name()))
    }

    /// Returns the path to the query file.
    pub fn query_dataset_file(&self) -> PathBuf {
        self.tmp_dir()
            .join(self.name())
            .join(format!("{}_query.fvecs", self.name()))
    }

    /// Returns the temporary directory path for testing OasysDB.
    fn tmp_dir(&self) -> PathBuf {
        let dir = env::temp_dir().join("oasysdb");
        if !dir.exists() {
            fs::create_dir_all(&dir).unwrap();
        }

        dir
    }
}

use byteorder::{LittleEndian, ReadBytesExt};
use curl::easy::Easy;
use flate2::read::GzDecoder;
use oasysdb::collection::Record;
use rayon::iter::*;
use std::error::Error;
use std::fs::{create_dir_all, File};
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::path::Path;
use tar::Archive;
use tokio::runtime::Runtime;

async fn download_file(url: &str, to: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::create(to)?;

    let mut easy = Easy::new();
    easy.url(url)?;

    // Write the response to the file.
    easy.write_function(move |data| {
        file.write_all(data).unwrap();
        Ok(data.len())
    })?;

    easy.perform()?;
    Ok(())
}

fn extract_tar_gz(path: &str, to: &str) -> Result<(), Box<dyn Error>> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(to).unwrap();
    Ok(())
}

/// Downloads and extracts the dataset to `/data/siftsmall`.
pub fn download_siftsmall() -> Result<(), Box<dyn Error>> {
    // Check if the dataset exists.
    let source = "data/siftsmall/siftsmall_base.fvecs";
    if Path::new(source).exists() {
        return Ok(());
    }

    // Create the data directory if it does not exist.
    if !Path::new("data").exists() {
        create_dir_all("data")?;
    }

    // Download the dataset.
    let url = "ftp://ftp.irisa.fr/local/texmex/corpus/siftsmall.tar.gz";
    let to = "data/siftsmall.tar.gz";
    Runtime::new()?.block_on(download_file(url, to))?;

    // Extract the dataset.
    extract_tar_gz(to, "data")?;
    Ok(())
}

/// Reads the vectors from the dataset.
/// * `path`: Path to the dataset file.
pub fn read_vectors(path: &str) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
    let ext = path.split(".").last().unwrap();
    if ext != "fvecs" {
        return Err("Invalid file extension.".into());
    }

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Read the vector dimension and size.
    let dimension = reader.read_i32::<LittleEndian>()? as usize;
    let vector_size = 4 + dimension * 4;

    // Get the number of vectors.
    let n = reader.seek(SeekFrom::End(0))? as usize / vector_size;

    // Seek the starting position.
    reader.seek(SeekFrom::Start(((0) * vector_size) as u64))?;

    // Read the vectors.
    let mut _vectors = vec![vec![0f32; n]; dimension];
    for i in 0..n {
        for j in 0..dimension {
            _vectors[j][i] = reader.read_f32::<LittleEndian>()?;
        }
    }

    // Transpose the vector.
    let rows = _vectors.len();
    let cols = _vectors[0].len();
    let vectors = (0..cols)
        .map(|col| (0..rows).map(|row| _vectors[row][col]).collect())
        .collect();

    Ok(vectors)
}

/// Reads and converts the vectors from the dataset into records.
/// * `path`: Path to the dataset file.
pub fn get_records(path: &str) -> Result<Vec<Record>, Box<dyn Error>> {
    // Create records where the ID is the index.
    let records = read_vectors(path)?
        .par_iter()
        .enumerate()
        .map(|(id, vec)| Record::new(vec.into(), id.into()))
        .collect();

    Ok(records)
}

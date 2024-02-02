use byteorder::{LittleEndian, ReadBytesExt};
use oasysdb::collection::Record;
use oasysdb::vector::Vector;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};

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

pub fn get_records(
    path: &str,
) -> Result<Vec<Record<usize, 128>>, Box<dyn Error>> {
    let vectors = read_vectors(path)?;

    // Create records where the ID is the index.
    let records = vectors
        .iter()
        .enumerate()
        .map(|(id, vec)| {
            let vector: [f32; 128] = vec.as_slice().try_into().unwrap();
            Record { vector: Vector(vector), data: id }
        })
        .collect();

    Ok(records)
}

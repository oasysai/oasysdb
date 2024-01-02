use std::fs::File;
use std::io::{BufReader, Seek, SeekFrom};
use std::error::Error;
use byteorder::{ReadBytesExt, LittleEndian};
use oasysdb::db::database::Embedding;


pub enum FloatType {
    F32,
    F64,
}

pub fn vecs_read(filename: &str) -> Result<Vec<Embedding>, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);

    // if extensions is .ivecs read as f32, if .fvecs read as f64
    let ext = filename.split(".").last().unwrap();
    let float_type = match ext {
        "fvecs" => FloatType::F32,
        "ivecs" => FloatType::F64,
        _ => return Err("Invalid file extension".into()),
    };
    // Read the vector size (dimension)
    let d = reader.read_i32::<LittleEndian>()? as usize;

    // Calculate the size of each vector in bytes
    let vecsizeof = 4 + d * 4;


    // Get the number of vectors
    let n = reader.seek(SeekFrom::End(0))? as usize / vecsizeof;


    // Seek to the starting position
    reader.seek(SeekFrom::Start(((0) * vecsizeof) as u64))?;

    // Read n vectors
    let mut v = vec![vec![0f32; n]; d];
    for i in 0..n {
        // Check if the first value (dimension of the vectors) is correct
        let dim = reader.read_i32::<LittleEndian>()? as usize;
        if dim != d {
            println!("dim: {}", dim);
            println!("d: {}", d);
            return Err("Invalid vector dimension".into());
        }
        for j in 0..d {
            v[j][i] = match float_type {
                FloatType::F32 => reader.read_f32::<LittleEndian>()?,
                FloatType::F64 => reader.read_f64::<LittleEndian>()? as f32,
            };
        }
    }

    // transpose the vector
    let rows = v.len();
    let cols = v[0].len();

    let transposed: Vec<Embedding> = (0..cols).map(|col| {
        (0..rows)
            .map(|row| v[row][col])
            .collect()
    }).collect();

    Ok(transposed)
}

pub fn load_base_dataset() -> Vec<Embedding> {
    let filename = "data/siftsmall/siftsmall_base.fvecs";
    vecs_read(filename).unwrap()
}

pub fn load_query_dataset() -> Vec<Embedding> {
    let filename = "data/siftsmall/siftsmall_query.fvecs";
    vecs_read(filename).unwrap()
}
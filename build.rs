use std::error::Error;
use tonic_build::compile_protos;

fn main() -> Result<(), Box<dyn Error>> {
    compile_protos("protos/database.proto")?;
    Ok(())
}

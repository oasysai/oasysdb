use crate::db::database::Database;
use crate::types::error::Error;
use std::fs;
use std::path::PathBuf;

mod test_collection;
mod test_database;

const TEST_DIR: &str = "/tmp/oasysdb";

fn create_new_test_database() -> Result<Database, Error> {
    // Reset the database directory for testing.
    let path = PathBuf::from(TEST_DIR);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }

    // The database should have some subdirectories.
    let db = Database::open(path.clone())?;
    let subdirs = path.read_dir()?;
    assert!(subdirs.count() >= 3);

    Ok(db)
}

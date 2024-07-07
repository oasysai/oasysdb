use crate::prelude::*;
use crate::types::file;
use std::path::PathBuf;

mod test_database;

fn create_test_database() -> Result<Database, Error> {
    let path = PathBuf::from("odb_data");
    let source_url = {
        let db_path = file::get_tmp_dir()?.join("sqlite.db");
        Some(format!("sqlite://{}?mode=rwc", db_path.display()))
    };

    let db = Database::open(path, source_url)?;
    let state = db.state();
    assert_eq!(state.source_type(), "sqlite");
    Ok(db)
}

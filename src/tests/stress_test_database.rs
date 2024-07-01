use super::*;

const RECORDS_LEN: usize = 10_000;

#[test]
fn test_database_insert_many_records() -> Result<(), Error> {
    let path = PathBuf::from(TEST_DIR);
    if path.exists() {
        fs::remove_dir_all(&path)?;
    }

    let database = Database::open(path)?;

    let collection_name = "collection";
    database._create_collection(collection_name)?;

    let fields = vec!["vector".to_string()];
    let vectors = generate_random_vectors(128, RECORDS_LEN);
    let records = vec![
        Arc::new(array::ListArray::from_vectors(vectors)) as Arc<dyn Array>
    ];

    database._insert_records(collection_name, &fields, &records)?;

    let state = database.state()?;
    let collection_dir = &state.collection_refs[collection_name];
    let collection = Collection::open(collection_dir.clone())?;
    let state = collection.state()?;
    assert_eq!(state.count, RECORDS_LEN);

    Ok(())
}

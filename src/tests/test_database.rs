use super::*;

#[test]
fn test_database_create_collection() -> Result<(), Error> {
    let db = create_test_database()?;
    let name = "new_collection";
    db._create_collection(name)?;

    let state = db.state()?;
    assert!(state.collection_refs.contains_key(name));
    Ok(())
}

#[test]
fn test_database_delete_collection() -> Result<(), Error> {
    let db = create_test_database()?;
    db._delete_collection(TEST_COLLECTION)?;

    let state = db.state()?;
    assert!(!state.collection_refs.contains_key(TEST_COLLECTION));
    Ok(())
}

#[test]
fn test_database_add_fields() -> Result<(), Error> {
    let database = create_test_database()?;
    let state = database.state()?;
    let dir = &state.collection_refs[TEST_COLLECTION];

    let field = Field::new("id", DataType::Utf8, false);
    database._add_fields(TEST_COLLECTION, vec![field])?;

    let collection = Collection::open(dir.clone())?;
    let schema = collection.state()?.schema;
    assert!(schema.fields().find("id").is_some());

    Ok(())
}

#[test]
#[should_panic]
fn test_database_remove_default_fields() {
    let database = create_test_database().unwrap();
    let fields = ["internal_id".to_string()];
    database._remove_fields(TEST_COLLECTION, &fields).unwrap();
}

#[test]
fn test_database_remove_fields() -> Result<(), Error> {
    let database = create_test_database()?;
    let state = database.state()?;
    let dir = &state.collection_refs[TEST_COLLECTION];

    let fields = ["title".to_string()];
    database._remove_fields(TEST_COLLECTION, &fields)?;

    let collection = Collection::open(dir.clone())?;
    let schema = collection.state()?.schema;
    assert!(schema.fields().find("title").is_none());

    Ok(())
}

use super::*;

#[test]
fn test_create_collection() -> Result<(), Error> {
    let db = create_test_database()?;
    let name = "new_collection";
    db._create_collection(name)?;

    let state = db.state()?;
    assert!(state.collection_refs.contains_key(name));
    Ok(())
}

#[test]
fn test_delete_collection() -> Result<(), Error> {
    let db = create_test_database()?;
    db._delete_collection(TEST_COLLECTION)?;

    let state = db.state()?;
    assert!(!state.collection_refs.contains_key(TEST_COLLECTION));
    Ok(())
}

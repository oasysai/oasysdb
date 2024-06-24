use super::*;

#[test]
fn test_create_collection() -> Result<(), Error> {
    let db = create_new_test_database()?;
    let name = "collection";
    db._create_collection(name)?;

    let state = db.state()?;
    assert!(state.collection_refs.contains_key(name));
    Ok(())
}

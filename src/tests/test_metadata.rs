#[allow(unused_imports)]
use super::*;

#[cfg(feature = "json")]
use serde_json::{json, Value};

#[cfg(feature = "json")]
#[test]
fn json_value_to_metadata() {
    let map = HashMap::from([("key", "value")]);
    let value = json!(map);

    let metadata = Metadata::from(map);
    let metadata_from_value = Metadata::from(value);

    assert_eq!(metadata, metadata_from_value);
}

#[cfg(feature = "json")]
#[test]
fn metadata_to_json_value() {
    let map = HashMap::from([("key", "value")]);
    let value = json!(map);

    let metadata = Metadata::from(map);
    let value_from_metadata = Value::from(metadata);

    assert_eq!(value, value_from_metadata);
}

#[cfg(feature = "json")]
#[test]
fn insert_data_type_json() {
    let mut collection = create_collection();

    let data = json!({
        "number": 1,
        "boolean": true,
        "string": "text",
    });

    // Create a new record with JSON data.
    let vector = Vector::random(DIMENSION);
    let new_record = Record::new(&vector, &data.clone().into());
    let id = collection.insert(&new_record).unwrap();

    let metadata = Metadata::from(data);
    assert_eq!(collection.get(&id).unwrap().data, metadata);
}

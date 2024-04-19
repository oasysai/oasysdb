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

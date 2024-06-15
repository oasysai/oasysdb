use crate::prelude::*;
use serde_json::json;

const DIMENSION: usize = 128;

fn create_collection_multitype_metadata() -> Collection {
    let config = Config::default();
    let mut collection = Collection::new(&config);

    let vectors = vec![Vector::random(DIMENSION); 10];

    // Text metadata.
    let data = "This is awesome!";
    let record = Record::new(&vectors[0], &data.into());
    collection.insert(&record).unwrap();

    // Integer metadata.
    let data = 10;
    let record = Record::new(&vectors[2], &data.into());
    collection.insert(&record).unwrap();

    // Float metadata.
    let data = 20.0;
    let record = Record::new(&vectors[3], &data.into());
    collection.insert(&record).unwrap();

    // Boolean metadata.
    let data = true;
    let record = Record::new(&vectors[4], &data.into());
    collection.insert(&record).unwrap();

    // Array metadata.
    let data = vec![10, 20, 30];
    let record = Record::new(&vectors[5], &data.into());
    collection.insert(&record).unwrap();

    // Object metadata.
    let data = json!({
        "key": "value",
        "number": 10,
    });

    let record = Record::new(&vectors[6], &data.into());
    collection.insert(&record).unwrap();

    collection
}

#[test]
#[should_panic]
fn text_gt_filter() {
    let operator = FilterOperator::GreaterThan;
    Filter::new("text", &json!("value").into(), &operator);
}

#[test]
fn float_lt_filter() {
    let operator = FilterOperator::LessThan;
    let filter = Filter::new("float", &json!(10.5).into(), &operator);
    let filter_from_str = Filter::from("float < 10.5");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn boolean_neq_filter() {
    let operator = FilterOperator::NotEqual;
    let filter = Filter::new("boolean", &json!(true).into(), &operator);
    let filter_from_str = Filter::from("boolean != true");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn object_gteq_filter() {
    let operator = FilterOperator::GreaterThanOrEqual;
    let filter = Filter::new("object.id", &json!(10).into(), &operator);
    let filter_from_str = Filter::from("object.id >= 10");
    assert_eq!(filter, filter_from_str);
}

#[test]
#[should_panic]
fn object_as_value_filter() {
    let operator = FilterOperator::GreaterThan;
    let value = json!({ "key": "value" }).into();
    Filter::new("object", &value, &operator);
}

#[test]
fn and_filters() {
    let filters = Filters::AND(vec![
        Filter::new("text", &json!("value").into(), &FilterOperator::Equal),
        Filter::new("integer", &json!(10).into(), &FilterOperator::GreaterThan),
    ]);

    let filters_from_str = Filters::from("text = value AND integer > 10");
    assert_eq!(filters, filters_from_str);
}

#[test]
fn collection_text_integer_or_filters() {
    let collection = create_collection_multitype_metadata();
    let filters = Filters::from("text CONTAINS awesome OR integer > 5");
    let result = collection.filter(&filters).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn collection_array_filter() {
    let collection = create_collection_multitype_metadata();
    let filters = Filters::from("array CONTAINS 20");
    let result = collection.filter(&filters).unwrap();
    assert_eq!(result.len(), 1);

    let filters = Filters::from("array.0 >= 10");
    let result = collection.filter(&filters).unwrap();
    assert_eq!(result.len(), 1);
}

#[test]
fn collection_object_filter() {
    let collection = create_collection_multitype_metadata();
    let filters = Filters::from("object.key CONTAINS val OR object.number > 5");
    let result = collection.filter(&filters).unwrap();
    assert_eq!(result.len(), 1);

    // This should return an empty result since there is no way to store both
    // array and object at the same level at the same time.
    let filters = Filters::from("object.number = 10 AND array.0 <= 10");
    let result = collection.filter(&filters).unwrap();
    assert_eq!(result.len(), 0);
}

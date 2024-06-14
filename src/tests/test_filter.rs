use crate::filter::*;
use serde_json::json;

#[test]
fn text_eq_filter() {
    let operator = FilterOperator::Equal;
    let filter = Filter::new("text", json!("text value").into(), operator);
    let filter_from_str = Filter::from("text = text value");
    assert_eq!(filter, filter_from_str);
}

#[test]
#[should_panic]
fn text_gt_filter() {
    let operator = FilterOperator::GreaterThan;
    Filter::new("text", json!("value").into(), operator);
}

#[test]
fn integer_gt_filter() {
    let operator = FilterOperator::GreaterThan;
    let filter = Filter::new("integer", json!(10).into(), operator);
    let filter_from_str = Filter::from("integer > 10");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn float_lt_filter() {
    let operator = FilterOperator::LessThan;
    let filter = Filter::new("float", json!(10.5).into(), operator);
    let filter_from_str = Filter::from("float < 10.5");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn boolean_neq_filter() {
    let operator = FilterOperator::NotEqual;
    let filter = Filter::new("boolean", json!(true).into(), operator);
    let filter_from_str = Filter::from("boolean != true");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn object_eq_filter() {
    let operator = FilterOperator::Equal;
    let value = json!("text value").into();
    let filter = Filter::new("object.key", value, operator);
    let filter_from_str = Filter::from("object.key = text value");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn object_gteq_filter() {
    let operator = FilterOperator::GreaterThanOrEqual;
    let filter = Filter::new("object.id", json!(10).into(), operator);
    let filter_from_str = Filter::from("object.id >= 10");
    assert_eq!(filter, filter_from_str);
}

#[test]
#[should_panic]
fn object_as_value_filter() {
    let operator = FilterOperator::GreaterThan;
    let value = json!({ "key": "value" }).into();
    Filter::new("object", value, operator);
}

#[test]
fn array_contains_filter() {
    let operator = FilterOperator::Contains;
    let filter = Filter::new("array", json!("value").into(), operator);
    let filter_from_str = Filter::from("array CONTAINS value");
    assert_eq!(filter, filter_from_str);
}

#[test]
fn and_filters() {
    let filters = Filters::AND(vec![
        Filter::new("text", json!("value").into(), FilterOperator::Equal),
        Filter::new("integer", json!(10).into(), FilterOperator::GreaterThan),
    ]);

    let filters_from_str = Filters::from("text = value AND integer > 10");
    assert_eq!(filters, filters_from_str);
}

#[test]
fn or_filters_complex_type() {
    let query = "object.number >= 0 OR object.text CONTAINS value";
    let filters_from_str = Filters::from(query);

    let operator = FilterOperator::GreaterThanOrEqual;
    let filter_1 = Filter::new("object.number", json!(0).into(), operator);

    let operator = FilterOperator::Contains;
    let filter_2 = Filter::new("object.text", json!("value").into(), operator);

    let filters = Filters::OR(vec![filter_1, filter_2]);
    assert_eq!(filters, filters_from_str);
}

use super::*;

#[test]
fn test_set_value() {
    let client = create_test_client();

    // Create a serialized database value for body.
    let body = {
        let embedding = vec![0.0, 0.0];
        let value = Value { embedding, data: HashMap::new() };
        serde_json::to_string(&value).unwrap()
    };

    let response = client
        .post("/values/10")
        .body(body)
        .header(get_auth_header())
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_get_value() {
    let client = create_test_client();
    let header = get_auth_header();
    let response = client.get("/values/0").header(header).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_delete_value() {
    let client = create_test_client();
    let header = get_auth_header();
    let response = client.delete("/values/0").header(header).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

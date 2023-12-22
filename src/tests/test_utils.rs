use super::*;

#[test]
fn test_get_status() {
    let client = create_test_client("test_get_status");
    let response = client.get(uri!(get_status)).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_get_version() {
    let client = create_test_client("test_get_version");
    let response = client.get(uri!(get_version)).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

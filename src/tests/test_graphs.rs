use super::*;

#[test]
fn test_create_graph() {
    let client = create_test_client("test_create_graph");

    // Request body to create graph.
    let name = Some("all".to_string());
    let ef_construction = Some(10);
    let ef_search = Some(10);

    let data = CreateGraphBody {
        name,
        ef_construction,
        ef_search,
        ..Default::default()
    };

    // Send request to create graph.
    let header = get_auth_header();
    let body = serde_json::to_string(&data).unwrap();
    let response = client.post("/graphs").body(body).header(header).dispatch();

    assert_eq!(response.status(), Status::Created);
}

#[test]
fn test_query_graph() {
    let client = create_test_client("test_query_graph");

    // Request body for querying the graph.
    let embedding = vec![0.0, 0.0];
    let k = Some(5);
    let data = QueryGraphBody { embedding, k };
    let body = serde_json::to_string(&data).unwrap();

    // Query the default graph.
    let header = get_auth_header();
    let response = client
        .post("/graphs/default/query")
        .body(body)
        .header(header)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_delete_graph() {
    let client = create_test_client("test_delete_graph");
    let header = get_auth_header();
    let response = client.delete("/graphs/default").header(header).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

#[test]
fn test_reset_graphs() {
    let client = create_test_client("test_reset_graphs");
    let header = get_auth_header();
    let response = client.delete("/graphs").header(header).dispatch();
    assert_eq!(response.status(), Status::Ok);
}

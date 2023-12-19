mod common;

use common::{get_headers, run_server, stop_server};
use reqwest::Client;

// Test server host.
const HOST: &str = "http://127.0.0.1";

// Raw JSON strings for testing.
// This is needed to prevent overcomplicating the tests data.
// Use these with the client ...body() method.

const CREATE_VALUE: &str = r#"{
    "key": "key-10",
    "value": {"embedding": [0.0, 0.0], "data": {}}
}"#;

const QUERY_INDEX: &str = r#"{
    "embedding": [0.0, 0.0],
    "count": 5
}"#;

// Route testing.
// These are just a simple test to make sure that
// all of the core routes are working.
// Function name format: test_{method}_{route}.

#[tokio::test]
async fn test_get_root() {
    let port = String::from("31400");
    let runtime = run_server(port.clone()).await;

    // Make a get request to the root.
    let client = Client::new();
    let url = format!("{}:{}", HOST, port);
    let response = client.get(&url).send().await.unwrap();

    assert_eq!(response.status(), 200);
    stop_server(runtime, port).await;
}

#[tokio::test]
async fn test_post_values() {
    let port = String::from("31401");
    let runtime = run_server(port.clone()).await;

    // Create a key-value pair.
    let headers = get_headers();
    let url = format!("{}:{}/values", HOST, port);

    let client = Client::new();
    let response = client
        .post(&url)
        .headers(headers)
        .body(CREATE_VALUE)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), 201);
    stop_server(runtime, port).await;
}

#[tokio::test]
async fn test_get_values() {
    let port = String::from("31402");
    let runtime = run_server(port.clone()).await;

    // The key-0 is pre-populated for testing.
    let url = format!("{}:{}/values/key-0", HOST, port);

    // Call GET to get the value of the key.
    let headers = get_headers();
    let client = Client::new();
    let response = client.get(url).headers(headers).send().await.unwrap();

    assert_eq!(response.status(), 200);
    stop_server(runtime, port).await;
}

#[tokio::test]
async fn test_delete_values() {
    let port = String::from("31403");
    let runtime = run_server(port.clone()).await;

    // The key-5 is pre-populated when the server is started.
    let url = format!("{}:{}/values/key-5", HOST, port);

    // Use DELETE to delete the key-value pair.
    let headers = get_headers();
    let client = Client::new();
    let response = client.delete(&url).headers(headers).send().await.unwrap();

    // Assert for 204 status code.
    assert_eq!(response.status(), 204);
    stop_server(runtime, port).await;
}

#[tokio::test]
async fn test_post_index() {
    let port = String::from("31404");
    let runtime = run_server(port.clone()).await;

    // This is a POST request with no body as for this endpoint,
    // the body is optional: ef_search and ef_construction.
    let url = format!("{}:{}/index", HOST, port);
    let headers = get_headers();

    let client = Client::new();
    let res = client.post(&url).headers(headers).send().await.unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime, port).await;
}

#[tokio::test]
async fn test_post_index_query() {
    let port = String::from("31405");
    let runtime = run_server(port.clone()).await;

    // The body embedding is required and the dimension
    // must match the dimension specified in the config.
    let headers = get_headers();
    let url = format!("{}:{}/index/query", HOST, port);

    let client = Client::new();
    let res = client
        .post(&url)
        .headers(headers)
        .body(QUERY_INDEX)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime, port).await;
}

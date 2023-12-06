mod common;

use common::{run_server, stop_server};
use reqwest::{get, Client};

// Test server host.
const HOST: &str = "http://127.0.0.1";

// Raw JSON strings for testing.
// This is needed to prevent overcomplicating the tests data.
// Use these with the client ...body() method.

const CREATE_KVS: &str = r#"{
    "key": "key-10",
    "value": {"embedding": [0.0, 0.0], "data": {}}
}"#;

const SEARCH: &str = r#"{
    "embedding": [0.0, 0.0],
    "count": 5
}"#;

// Route testing.
// These are just a simple test to make sure that
// all of the core routes are working.
// Function name format: test_{method}_{route}.

#[tokio::test]
async fn test_get_root() {
    let (runtime, port) = run_server().await;

    let url = format!("{}:{}", HOST, port);
    let res = get(url).await.unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_kvs() {
    let (runtime, port) = run_server().await;

    // Make a post request to create key-value store.
    let url = format!("{}:{}/kvs", HOST, port);
    let client = Client::new();
    let res = client.post(&url).body(CREATE_KVS).send().await.unwrap();

    assert_eq!(res.status(), 201);
    stop_server(runtime).await;
}

#[tokio::test]
async fn test_get_kvs() {
    let (runtime, port) = run_server().await;

    // Get the key-value store.
    // This key is pre-populated in the server.
    let url = format!("{}:{}/kvs/key-0", HOST, port);
    let res = get(url).await.unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime).await;
}

#[tokio::test]
async fn test_delete_kvs() {
    let (runtime, port) = run_server().await;

    // Delete the existing key-value store.
    let url = format!("{}:{}/kvs/key-0", HOST, port);
    let client = Client::new();
    let res = client.delete(&url).send().await.unwrap();

    // Assert for 204 status code.
    assert_eq!(res.status(), 204);
    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_build() {
    let (runtime, port) = run_server().await;

    // Build the index.
    let url = format!("{}:{}/build", HOST, port);
    let client = Client::new();
    let res = client.post(&url).send().await.unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_search() {
    let (runtime, port) = run_server().await;

    // Make a post request to search for nearest neighbors.
    let url = format!("{}:{}/search", HOST, port);
    let client = Client::new();
    let res = client.post(&url).body(SEARCH).send().await.unwrap();

    assert_eq!(res.status(), 200);
    stop_server(runtime).await;
}

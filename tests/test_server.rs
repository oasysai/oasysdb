mod common;

use common::{run_server, stop_server};
use reqwest::{get, Client};

// Test server host.
const HOST: &str = "http://127.0.0.1";

// JSON body to create a new key-value store.
const CREATE_KVS: &str = r#"{
    "key": "key-10",
    "value": {"embedding": [0.0, 0.0], "data": {}}
}"#;

#[tokio::test]
async fn test_get_root() {
    let (runtime, port) = run_server().await;

    // Make a GET request to the root.
    let url = format!("{}:{}", HOST, port);
    let res = get(url).await.unwrap();

    // Assert the response.
    assert_eq!(res.status(), 200);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_kvs() {
    let (runtime, port) = run_server().await;

    // Make a post request.
    let url = format!("{}:{}/kvs", HOST, port);
    let client = Client::new();
    let res = client.post(&url).body(CREATE_KVS).send().await.unwrap();

    // Assert the response code.
    assert_eq!(res.status(), 201);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_get_kvs() {
    let (runtime, port) = run_server().await;

    // Get the key-value store.
    let url = format!("{}:{}/kvs/key-0", HOST, port);
    let res = get(url).await.unwrap();

    // Assert the response code.
    assert_eq!(res.status(), 200);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_delete_kvs() {
    let (runtime, port) = run_server().await;

    // Delete the key-value store.
    let url = format!("{}:{}/kvs/key-0", HOST, port);
    let client = Client::new();
    let res = client.delete(&url).send().await.unwrap();

    // Assert the response code.
    assert_eq!(res.status(), 204);

    stop_server(runtime).await;
}

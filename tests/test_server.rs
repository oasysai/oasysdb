mod common;

use common::*;

// Test server host.
const HOST: &str = "http://127.0.0.1";

// JSON body to create a new key-value store.
const CREATE_KVS: &str = r#"{
    "key": "test_key",
    "value": {"embedding": [0.0], "data": {}}
}"#;

#[tokio::test]
async fn test_get_root() {
    let (runtime, port) = run_server().await;

    // Make a GET request to the root.
    let url = format!("{}:{}", HOST, port);
    let res = reqwest::get(url).await.unwrap();

    // Assert the response.
    assert_eq!(res.status(), 200);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_kvs() {
    let (runtime, port) = run_server().await;

    // Make a post request.
    let url = format!("{}:{}/kvs", HOST, port);
    let client = reqwest::Client::new();
    let res = client.post(&url).body(CREATE_KVS).send().await;

    // Get the response code.
    let code = if res.is_ok() {
        res.unwrap().status().as_u16()
    } else {
        500
    };

    // Assert the response code.
    assert_eq!(code, 201);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_get_kvs() {
    let (runtime, port) = run_server().await;

    // Get the key-value store.
    let url = format!("{}:{}/kvs/initial_key", HOST, port);
    let res = reqwest::get(url).await.unwrap();

    // Assert the response code.
    assert_eq!(res.status(), 200);

    stop_server(runtime).await;
}

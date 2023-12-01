mod common;

use common::*;

#[tokio::test]
async fn test_get_root() {
    let runtime = run_server().await;

    // Make a GET request to the root.
    let url = format!("http://{}:{}", HOST, PORT);
    let res = reqwest::get(url).await.unwrap();

    // Assert the response.
    assert_eq!(res.status(), 200);

    stop_server(runtime).await;
}

#[tokio::test]
async fn test_post_kvs() {
    let runtime = run_server().await;

    let url = format!("http://{}:{}/kvs", HOST, PORT);

    // Create raw JSON string for the body.
    let json = r#"{
        "key": "test_key",
        "value": {"embedding": [0.0], "data": {}}
    }"#;

    // Make a post request.
    let client = reqwest::Client::new();
    let res = client.post(&url).body(json).send().await;

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

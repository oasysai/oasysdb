use oasysdb::db::client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::new("127.0.0.1", "3141").await;

    // Testing the server functions.
    client.set("key_1", "value_1").await;
    client.set("key_2", "value_2").await;
    client.get("key").await;
}

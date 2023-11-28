use serde_json::to_string;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub async fn new(host: &str, port: &str) -> Client {
        let addr: SocketAddr = format!("{}:{}", host, port).parse().unwrap();
        let stream = TcpStream::connect(addr).await.unwrap();
        Client { stream }
    }

    pub async fn set(&mut self, key: &str, value: &str) {
        let mut map = HashMap::new();
        map.insert("command", "set");
        map.insert("key", key);
        map.insert("value", value);

        let data = to_string(&map).unwrap();
        self.send(&data).await;
    }

    pub async fn get(&mut self, key: &str) {
        let mut map = HashMap::new();
        map.insert("command", "get");
        map.insert("key", key);

        let data = to_string(&map).unwrap();
        self.send(&data).await;
    }

    async fn send(&mut self, data: &str) {
        // Write data to the server.
        self.stream.write_all(data.as_bytes()).await.unwrap();

        // Read the server's response.
        let mut buf = vec![0; 1024];
        let n = self.stream.read(&mut buf).await.unwrap();
        let response = String::from_utf8_lossy(&buf[0..n]);

        // Print the server's response.
        println!("oasysdb > {}", response);
    }
}

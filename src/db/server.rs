use serde_json::{from_str, to_string};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::spawn;

// Set up some type aliases for convenience.
type RequestBody = HashMap<String, String>;
type KeyValue = Arc<Mutex<HashMap<String, String>>>;

pub struct Server {
    addr: SocketAddr,
    kv: KeyValue,
}

impl Server {
    pub async fn new(host: &str, port: &str) -> Server {
        let addr = format!("{}:{}", host, port).parse().unwrap();
        let kv = Arc::new(Mutex::new(HashMap::new()));
        Server { addr, kv }
    }

    pub async fn serve(&self) {
        // Bind a listener to the socket address.
        let listener = TcpListener::bind(self.addr).await.unwrap();

        // Accept and handle connections from clients.
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            let handler = self.handle_connection(stream).await;
            spawn(async move { handler });
        }
    }

    async fn handle_connection(&self, mut stream: TcpStream) {
        loop {
            // Read data from the client.
            let mut buf = vec![0; 1024];
            let n = match stream.read(&mut buf).await {
                Ok(n) => n,
                Err(e) => {
                    println!("ReadError: {}", e);
                    return;
                }
            };

            // When the client disconnects.
            if n == 0 {
                break;
            }

            // Parse the request body.
            let data = String::from_utf8_lossy(&buf[0..n]);
            let json: RequestBody = from_str(&data).unwrap();

            // TODO: Add error handling for missing keys.
            let command = json.get("command").unwrap().to_lowercase();

            // Create unrecognized command error.
            let mut _err = HashMap::new();
            _err.insert("status", "unrecognized_command");
            let error = to_string(&_err).unwrap();

            // Handle the command for the response.
            let response = match command.as_str() {
                "set" => self.handle_set(json).await,
                "get" => self.handle_get(json).await,
                _ => error,
            };

            // Write the data back to the client.
            match stream.write_all(response.as_bytes()).await {
                Ok(_) => (),
                Err(e) => {
                    println!("WriteError: {}", e);
                    return;
                }
            }
        }
    }

    async fn handle_set(&self, json: RequestBody) -> String {
        // Get the key and value from the request body.
        let key = json.get("key").unwrap().to_string();
        let value = json.get("value").unwrap().to_string();

        // Create a map for the response.
        let mut map = HashMap::new();
        map.insert(key.clone(), value.clone());

        // Insert the key and value into the database.
        self.kv.lock().unwrap().insert(key, value);

        to_string(&map).unwrap()
    }

    async fn handle_get(&self, json: RequestBody) -> String {
        // TODO: Add error handling for missing keys.
        let key = json.get("key").unwrap().to_string();

        // Get the value from the database.
        // TODO: Add error handling when value not found.
        let value = self.kv.lock().unwrap().get(&key).unwrap().to_string();

        // Create a map for the response.
        let mut map = HashMap::new();
        map.insert(key, value);

        to_string(&map).unwrap()
    }
}

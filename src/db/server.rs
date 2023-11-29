use http::Response;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

type RequestHeaders = HashMap<String, String>;

// The request body still need to be deserialized
// to the proper format, likely a HashMap or Vec.
struct Request {
    pub method: String,
    pub route: String,
    pub headers: RequestHeaders,
    pub body: String,
}

// Use Arc and Mutex to share the key-value store
// across threads while ensuring exclusive access.
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
            let handler = self._handle_connection(stream).await;
            tokio::spawn(async move { handler });
        }
    }

    async fn _handle_connection(&self, mut stream: TcpStream) {
        loop {
            // Read request from the client.
            let _req = self._read(&mut stream).await;

            // Handle disconnection.
            if _req.is_none() {
                break;
            }

            // Unwrap the data.
            let req: &Request = _req.as_ref().unwrap();
            let method = req.method.clone();
            let route = req.route.clone();

            let response = match route.as_str() {
                "/" => self._handle_root(&method),
                "/version" => self._handle_version(&method),
                _ => self._get_not_found_res(),
            };

            // Write the data back to the client.
            self._write(&mut stream, response).await;
        }
    }

    // Route handlers.
    // These are the functions that handle certain routes.
    // They are called by the handle_connection method.
    // Use format: _handle_<route> for naming.

    fn _handle_root(&self, method: &str) -> Response<String> {
        match method {
            "get" => self._get_root(),
            _ => self._get_not_allowed_res(),
        }
    }

    fn _handle_version(&self, method: &str) -> Response<String> {
        match method {
            "get" => self._get_version(),
            _ => self._get_not_allowed_res(),
        }
    }

    // Route functions.
    // These are the functions that handle the route functionality
    // and is used by the handle_connection method.
    // Use format: _<method>_<route> for naming.

    fn _get_root(&self) -> Response<String> {
        // Create a HashMap to store the status.
        let mut map = HashMap::new();
        map.insert("status", "ok");

        Response::builder()
            .status(200)
            .body(serde_json::to_string(&map).unwrap())
            .unwrap()
    }

    fn _get_version(&self) -> Response<String> {
        // Get the version from the Cargo.toml file.
        let ver = env!("CARGO_PKG_VERSION");

        // Create a HashMap to store the version.
        let mut map = HashMap::new();
        map.insert("version", ver);

        Response::builder()
            .status(200)
            .body(serde_json::to_string(&map).unwrap())
            .unwrap()
    }

    // Stream utilities.
    // These are the private methods that help us read and
    // write data from and to the stream.

    async fn _read(&self, stream: &mut TcpStream) -> Option<Request> {
        // Prepare the request for parsing.
        let mut _headers = [httparse::EMPTY_HEADER; 16];
        let mut _req = httparse::Request::new(&mut _headers);

        // Read data from the stream.
        let mut buf = vec![0; 1024];
        let n = stream.read(&mut buf).await.unwrap();

        // Disconnection handler.
        if n == 0 {
            return None;
        }

        // Parse the request.
        let _ = _req.parse(&buf).unwrap();

        // Parse request headers.
        let headers: RequestHeaders = HashMap::from_iter(_req.headers.iter().map(|header| {
            let key = header.name.to_lowercase();
            let val = String::from_utf8_lossy(header.value).to_string();
            (key, val)
        }));

        // If content length is present or more than 0, read the body.
        let _content_len = headers
            .get("content-length")
            .unwrap_or(&"0".to_string())
            .parse::<usize>()
            .unwrap_or(0);

        // Parse the request body.
        let body = if _content_len > 0 {
            let _buf = String::from_utf8_lossy(&buf);
            let _parts = _buf.split_once("\r\n\r\n").unwrap();
            _parts.1.replace("\0", "").clone()
        } else {
            String::new()
        };

        // Return request data.
        let data = Some(Request {
            method: _req.method.unwrap().to_lowercase(),
            route: _req.path.unwrap().to_string(),
            headers,
            body,
        });

        data
    }

    async fn _write(&self, stream: &mut TcpStream, response: Response<String>) {
        let (parts, body) = response.into_parts();

        // Get the status code and canonical reason.
        let status = parts.status.as_str();
        let reason = parts.status.canonical_reason().unwrap();

        // HTTP response tag and header.
        let tag = format!("HTTP/1.1 {} {}", status, reason);
        let header = format!("content-length: {}", body.len());

        // Format the response as a string.
        let data = format!("{}\r\n{}\r\n\r\n{}", tag, header, body);

        // Write the response to the stream.
        stream.write_all(data.as_bytes()).await.unwrap();
    }

    // Response helpers.
    // Write any code that helps us create a response below.
    // Prefix the function name _get or _create.
    // Suffixed the function name with _res.

    fn _create_blank_res(&self, code: u16) -> Response<String> {
        let code = http::StatusCode::from_u16(code).unwrap();
        Response::builder()
            .status(code)
            .body("{}".to_string())
            .unwrap()
    }

    fn _get_not_allowed_res(&self) -> Response<String> {
        self._create_blank_res(405)
    }

    fn _get_not_found_res(&self) -> Response<String> {
        self._create_blank_res(404)
    }
}

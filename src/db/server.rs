use http::Response;
use serde_json::Value as RequestBody;
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
    pub body: RequestBody,
}

// This type will be used to serialize the response body.
type ResponseBody = HashMap<&'static str, &'static str>;

// This is the data structure that will be stored in
// the key-value store as the value.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub embedding: Vec<f32>,
    pub data: HashMap<String, String>,
}

// Use Arc and Mutex to share the key-value store
// across threads while ensuring exclusive access.
type KeyValue = Arc<Mutex<HashMap<String, Value>>>;

pub struct Server {
    addr: SocketAddr,
    kvs: KeyValue,
}

impl Server {
    pub async fn new(host: &str, port: &str) -> Server {
        let addr = format!("{}:{}", host, port).parse().unwrap();
        let kvs = Arc::new(Mutex::new(HashMap::new()));
        Server { addr, kvs }
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

            // Handle disconnection or invalid request.
            // Return invalid request response.
            if _req.is_none() {
                let mut _res_body = HashMap::new();
                _res_body.insert("error", "Invalid request.");
                let res: Response<String> = self._create_res(400, Some(_res_body));
                self._write(&mut stream, res).await;
                break;
            }

            // Unwrap the data.
            let req: &Request = _req.as_ref().unwrap();
            let route = req.route.clone();

            // Get response based on different routes and methods.
            let response = match route.as_str() {
                "/" => self._handle_root(req),
                "/version" => self._handle_version(req),
                _ if route.starts_with("/kvs") => self._handle_kvs(req),
                _ => self._get_not_found_res(),
            };

            // Write the data back to the client.
            self._write(&mut stream, response).await;
        }
    }

    // Native functionality handler.
    // These are the functions that handle the native
    // functionality of the database.
    // Example: get, set, delete, etc.

    pub fn get(&self, key: String) -> Option<Value> {
        let kvs = self.kvs.lock().unwrap();
        kvs.get(&key).cloned()
    }

    pub fn set(&self, key: String, value: Value) {
        let mut kvs = self.kvs.lock().unwrap();
        kvs.insert(key, value);
    }

    // Route handlers.
    // These are the functions that handle certain routes.
    // They are called by the handle_connection method.
    // Use format: _handle_<route> for naming.

    fn _handle_root(&self, request: &Request) -> Response<String> {
        match request.method.as_str() {
            "get" => self._get_root(),
            _ => self._get_not_allowed_res(),
        }
    }

    fn _handle_version(&self, request: &Request) -> Response<String> {
        match request.method.as_str() {
            "get" => self._get_version(),
            _ => self._get_not_allowed_res(),
        }
    }

    fn _handle_kvs(&self, request: &Request) -> Response<String> {
        match request.method.as_str() {
            "get" => self._get_kvs_key(request.route.clone()),
            "post" => self._post_kvs(request.body.clone()),
            _ => self._get_not_allowed_res(),
        }
    }

    // Route functions.
    // These are the functions that handle the route functionality
    // and is used by the handle_connection method.
    // Use format: _<method>_<route> for naming.

    fn _get_root(&self) -> Response<String> {
        let mut map = HashMap::new();
        map.insert("status", "ok");
        self._create_res(200, Some(map))
    }

    fn _get_version(&self) -> Response<String> {
        // Get the version from the Cargo.toml file.
        let ver = env!("CARGO_PKG_VERSION");

        // Create a HashMap to store the version.
        let mut map = HashMap::new();
        map.insert("version", ver);

        self._create_res(200, Some(map))
    }

    fn _get_kvs_key(&self, route: String) -> Response<String> {
        // Get the key from the route.
        let route_parts: Vec<&str> = route.split("/").collect();
        let key = route_parts.last().unwrap().to_string();

        // If key is empty, return 400 with error message.
        if key.is_empty() || route_parts.len() < 3 {
            let mut _map = HashMap::new();
            _map.insert("error", "The key is required.");
            return self._create_res(400, Some(_map));
        }

        // Get the value from the key-value store.
        let value = self.get(key.clone());

        // If value is None, return 404 with error message.
        if value.is_none() {
            let mut _map = HashMap::new();
            let msg = "The value is not found.";
            _map.insert("error", msg);
            return self._create_res(404, Some(_map));
        }

        // Serialize value as string for the response.
        let body = {
            let _val: Value = value.unwrap();
            serde_json::to_string(&_val).unwrap()
        };

        Response::builder().status(200).body(body).unwrap()
    }

    fn _post_kvs(&self, request_body: RequestBody) -> Response<String> {
        // If request body is missing key or value.
        if request_body.get("key").is_none() || request_body.get("value").is_none() {
            let mut _map = HashMap::new();
            _map.insert("error", "Both key and value are required.");
            return self._create_res(400, Some(_map));
        }

        // Get the key from request body.
        // Validate that key is string.
        let key: String = match request_body["key"].as_str() {
            Some(key) => key.to_string(),
            None => {
                let mut _map = HashMap::new();
                _map.insert("error", "The key must be a string.");
                return self._create_res(400, Some(_map));
            }
        };

        // Get the value from request body.
        // Validate that value is a Value struct.
        let value: Value = match serde_json::from_value(request_body["value"].clone()) {
            Ok(value) => value,
            Err(_) => {
                let mut _map = HashMap::new();
                let msg = "The value provided is invalid.";
                _map.insert("error", msg);
                return self._create_res(400, Some(_map));
            }
        };

        // Insert the key-value pair into the key-value store.
        self.set(key, value);

        // Serialize value as string for the response.
        let body = {
            let _val: Value = serde_json::from_value(request_body["value"].clone()).unwrap();
            serde_json::to_string(&_val).unwrap()
        };

        Response::builder().status(201).body(body).unwrap()
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
        // By default, the body is an empty map, not None.
        let _body = if _content_len > 0 {
            let _buf = String::from_utf8_lossy(&buf);
            let _parts = _buf.split_once("\r\n\r\n").unwrap();
            _parts.1.replace("\0", "").clone()
        } else {
            // Create an empty body.
            "{}".to_string()
        };

        // Try to parse the body. If fail, return None.
        // This will guard against invalid JSON.
        let body: Option<RequestBody> = match serde_json::from_str(&_body) {
            Ok(body) => body,
            Err(_) => None,
        };

        // Returning None will cause the connection to close.
        if body.is_none() {
            return None;
        }

        // Return request data.
        let data = Some(Request {
            method: _req.method.unwrap().to_lowercase(),
            route: _req.path.unwrap().to_string(),
            headers: headers,
            body: body.unwrap(),
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

    fn _create_res(&self, code: u16, body: Option<ResponseBody>) -> Response<String> {
        // Check MDN for a list of status codes.
        // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status
        let code = http::StatusCode::from_u16(code).unwrap();

        // Serialize the body if provided.
        let _body = if !body.is_none() {
            serde_json::to_string(&body.unwrap()).unwrap()
        } else {
            // Default to an empty object.
            "{}".to_string()
        };

        // Return the response.
        Response::builder().status(code).body(_body).unwrap()
    }

    fn _get_not_allowed_res(&self) -> Response<String> {
        self._create_res(405, None)
    }

    fn _get_not_found_res(&self) -> Response<String> {
        self._create_res(404, None)
    }
}

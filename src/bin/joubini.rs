use nanoid::nanoid;
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tracing::{debug, error, info, warn};
use tracing_subscriber::{
    filter::targets::Targets, layer::SubscriberExt, util::SubscriberInitExt,
};

#[derive(Debug)]
struct Proxy {
    local_path: String,
    remote_path: String,
    remote_port: u16,
}

#[derive(Debug)]
struct ProxyParseError;

impl FromStr for Proxy {
    type Err = ProxyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((local, remote)) = s.split_once(':') {
            let (port, path) = if let Some((p1, p2)) = remote.split_once('/') {
                (p1, p2)
            } else {
                (remote, "")
            };

            return Ok(Proxy {
                local_path: ["/", local].join(""),
                remote_path: ["/", path].join(""),
                remote_port: port.parse::<u16>().unwrap(),
            });
        }

        Err(ProxyParseError)
    }
}

impl Proxy {
    fn new(local_path: String, remote_path: String, remote_port: u16) -> Self {
        Proxy {
            local_path,
            remote_path,
            remote_port,
        }
    }
}

#[tokio::main]
async fn main() {
    // let p1 = Proxy::new(String::from(":3000"));
    // let p2 = Proxy::new(String::from("api:3001/api"));
    // let p3 = Proxy::new(String::from("admin:3002/dashboard"));
    // let p4 = Proxy::new(String::from("db:3003"));
    // let p5 = Proxy::new(String::from("deep:3004/deep/nested/path"));

    let filter_layer = Targets::from_str(
        std::env::var("RUST_LOG").as_deref().unwrap_or("info"),
    )
    .unwrap();

    let format_layer = tracing_subscriber::fmt::layer();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(format_layer)
        .init();

    info!("Starting...");

    let addr = "127.0.0.1:7878";

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => {
            info!("Bound to address");
            l
        }
        Err(_) => {
            error!("Unable to bind to port");
            panic!();
        }
    };

    loop {
        let uid = nanoid!(5);

        if let Ok(socket) = listener.accept().await {
            info!("[{uid}] Established TCP connection");

            tokio::spawn(async move {
                info!("[{uid}] Handling request");

                let mut proxies = vec![];

                proxies.push(
                    Proxy::from_str("zero:3000/profile")
                        .expect("should parse proxy from string"),
                );

                proxies.push(
                    Proxy::from_str("one:3001/comments")
                        .expect("should parse proxy from string"),
                );

                let (stream, _) = socket;

                if handle_connection(stream, proxies, &uid).await.is_err() {
                    error!("[{uid}] Unable to handle connection");
                }
            });
        } else {
            error!("[{uid}] Unable to establish a TCP connection");
        }
    }
}

async fn handle_connection(
    stream: TcpStream,
    proxies: Vec<Proxy>,
    uid: &String,
) -> Result<(), Box<dyn Error>> {
    info!("[{uid}] Parsing incoming request...");
    let parsed_request = parse_incoming_request(stream).await?;

    info!("[{uid}] Mapping request to proxy request...");
    let proxy_request = map_proxy_request(parsed_request, proxies).await?;

    let request_str = build_proxy_request(proxy_request)?;

    println!("request_str:\n{:#?}", request_str);

    Ok(())
}

#[derive(Debug)]
struct Request {
    http_method: String,
    http_version: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    fn new() -> Request {
        Request {
            http_method: String::from(""),
            http_version: String::from(""),
            path: String::from(""),
            headers: HashMap::new(),
            body: None,
        }
    }
}

async fn parse_incoming_request(
    mut incoming_request_stream: TcpStream,
) -> Result<Request, Box<dyn Error>> {
    let mut request = Request::new();

    let mut reader = BufReader::new(&mut incoming_request_stream);

    let mut request_line = String::new();

    reader.read_line(&mut request_line).await?;

    let request_line_parts: Vec<&str> = request_line.split(' ').collect();

    request.http_method = String::from(request_line_parts[0].trim());
    request.path = String::from(request_line_parts[1].trim());
    request.http_version = String::from(request_line_parts[2].trim());

    loop {
        let mut header_line = String::new();

        reader.read_line(&mut header_line).await?;

        if header_line.trim().is_empty() {
            break;
        }

        if let Some((header_name, header_value)) = header_line.split_once(':') {
            request.headers.insert(
                String::from(header_name.trim()),
                String::from(header_value.trim()),
            );
        }
    }

    if let Some(content_length) = request.headers.get("Content-Length") {
        let mut body_buffer = vec![
            0u8;
            content_length.parse::<usize>().expect(
                "content length should be parsable to an int"
            )
        ];

        reader.read_exact(&mut body_buffer).await?;

        request.body = Some(String::from(
            std::str::from_utf8(&body_buffer)
                .expect("body should be parsable to string"),
        ));
    }

    Ok(request)
}

async fn map_proxy_request(
    mut request: Request,
    proxies: Vec<Proxy>,
) -> Result<Request, Box<dyn Error>> {
    let proxy = proxies
        .iter()
        .find(|x| request.path.starts_with(&x.local_path))
        .expect("should be a matching proxy for the request");

    request.headers.insert(
        String::from("Host"),
        format!("localhost:{}", proxy.remote_port),
    );

    request.path = request.path.replace(&proxy.local_path, &proxy.remote_path);

    Ok(request)
}

fn build_proxy_request(request: Request) -> Result<String, Box<dyn Error>> {
    let mut request_str = String::new();

    let request_line = format!(
        "{} {} {}",
        request.http_method, request.path, request.http_version
    );
    request_str.push_str(&request_line);
    request_str.push_str("\r\n");

    let headers = request
        .headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\r\n");

    request_str.push_str(&headers);
    request_str.push_str("\r\n\r\n");

    if let Some(body) = request.body {
        request_str.push_str(&body);
    }

    Ok(request_str)
}

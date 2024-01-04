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

impl Proxy {
    fn new(proxy_config: String) -> Self {
        if let Some((local, remote)) = proxy_config.split_once(':') {
            let (port, path) = if let Some((p1, p2)) = remote.split_once('/') {
                (p1, p2)
            } else {
                (remote, "")
            };

            return Proxy {
                local_path: ["/", local].join(""),
                remote_path: ["/", path].join(""),
                remote_port: port.parse::<u16>().unwrap(),
            };
        }

        error!("Invalid configuration: '{proxy_config}'");
        panic!();
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

                let mut proxies = HashMap::new();

                proxies.insert("zero", (3000, ""));
                proxies.insert("one", (3001, ""));

                let (stream, _) = socket;

                if handle_connection(stream, &proxies, &uid).await.is_err() {
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
    proxies: &HashMap<&str, (i32, &str)>,
    uid: &String,
) -> Result<(), Box<dyn Error>> {
    let parsed_request = parse_incoming_request(stream);

    println!("parsed_request:\n{:#?}", parsed_request.await?);

    Ok(())
}

#[derive(Debug)]
struct Request {
    http_method: String,
    http_version: String,
    path: String,
    request_line: String,
    request_headers: HashMap<String, String>,
    request_body: Option<String>,
}

impl Request {
    fn new() -> Request {
        Request {
            http_method: String::from(""),
            http_version: String::from(""),
            path: String::from(""),
            request_line: String::from(""),
            request_headers: HashMap::new(),
            request_body: None,
        }
    }
}

async fn parse_incoming_request(
    mut incoming_request_stream: TcpStream,
) -> Result<Request, Box<dyn Error>> {
    let mut request = Request::new();

    let mut reader = BufReader::new(&mut incoming_request_stream);

    reader.read_line(&mut request.request_line).await?;

    println!("request_line: {}", request.request_line);

    loop {
        let mut header_line = String::new();

        reader.read_line(&mut header_line).await?;

        if header_line.trim().is_empty() {
            break;
        }

        if let Some((header_name, header_value)) = header_line.split_once(':') {
            request.request_headers.insert(
                String::from(header_name.trim()),
                String::from(header_value.trim()),
            );
        }
    }

    if let Some(content_length) = request.request_headers.get("Content-Length")
    {
        println!("length: {content_length}");
        let mut body_buffer = vec![
            0u8;
            content_length.parse::<usize>().expect(
                "content length should be parsable to an int"
            )
        ];

        reader.read_exact(&mut body_buffer).await?;

        request.request_body = Some(String::from(
            std::str::from_utf8(&body_buffer)
                .expect("body should be parsable to string"),
        ));
    }

    Ok(request)
}

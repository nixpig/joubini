use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use nanoid::nanoid;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};
use tracing::{debug, debug_span, error, info};

use crate::proxy::Proxy;

pub async fn run(
    listener: &TcpListener,
    proxies: Arc<Vec<Proxy>>,
) -> Result<JoinHandle<()>, Box<dyn Error>> {
    let socket = listener
        .accept()
        .await
        .expect("Unable to establish TCP connection");

    let handle = tokio::spawn({
        let uid = nanoid!(5);
        let proxies = proxies.clone();
        let (stream, addr) = socket;

        async move {
            if handle_connection(stream, addr, proxies, &uid)
                .await
                .is_err()
            {
                error!("[{uid}] Unable to handle connection");
            }
        }
    });

    Ok(handle)
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    proxies: Arc<Vec<Proxy>>,
    uid: &String,
) -> Result<(), Box<dyn Error>> {
    debug!("[{uid}] Parsing incoming request...");
    let (parsed_request, mut stream) = parse_incoming_request(stream).await?;

    let local_path = parsed_request.path.clone();
    let method = parsed_request.http_method.clone();

    debug!("[{uid}] Mapping request to proxy request...");
    let proxy_request = map_proxy_request(parsed_request, proxies).await?;

    let remote_port = proxy_request.port;
    let remote_path = proxy_request.path.clone();

    let port = proxy_request.port;
    let request_str = build_proxy_request(proxy_request)?;

    let address = format!("{}:{}", addr.ip(), port);
    debug!("address: {address}");

    if let Ok(mut remote_stream) = TcpStream::connect(address).await {
        debug!("[{uid}] Connected to remote server");

        debug!("[{uid}] Request string:\n{:#?}", request_str);

        debug_span!("request:", method = %method, local_path = %local_path, remote_port = %remote_port, remote_path = %remote_path, id = %uid);

        info!(
            "[{uid}] {} {} => :{}{}",
            method, local_path, remote_port, remote_path
        );

        remote_stream.write_all(request_str.as_bytes()).await?;

        let mut response = vec![];

        debug!("[{uid}] Reading from remote stream");
        remote_stream.read_to_end(&mut response).await?;

        debug!("[{uid}] Writing to local stream");
        stream.write_all(&response).await?;
    } else {
        error!("[{uid}] Unable to connect to remote server");
    }

    Ok(())
}

#[derive(Debug)]
struct Request {
    http_method: String,
    http_version: String,
    path: String,
    port: u16,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Request {
    fn new() -> Request {
        Request {
            http_method: String::from(""),
            http_version: String::from(""),
            path: String::from(""),
            port: 80,
            headers: HashMap::new(),
            body: None,
        }
    }
}

async fn parse_incoming_request(
    mut incoming_request_stream: TcpStream,
) -> Result<(Request, TcpStream), Box<dyn Error>> {
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

    Ok((request, incoming_request_stream))
}

async fn map_proxy_request(
    mut request: Request,
    proxies: Arc<Vec<Proxy>>,
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

    request.port = proxy.remote_port;

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

    request_str.push_str("\r\nConnection: close\r\n\r\n");

    if let Some(body) = request.body {
        request_str.push_str(&body);
    }

    Ok(request_str)
}

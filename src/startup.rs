use std::{error::Error, net::SocketAddr, sync::Arc};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    task::JoinHandle,
};

use crate::{
    build::build_proxy_request, map::map_proxy_request,
    parse::parse_incoming_request, proxy::Proxy,
};

pub async fn run(
    listener: Arc<TcpListener>,
    proxies: Arc<Vec<Proxy>>,
) -> Result<JoinHandle<()>, std::io::Error> {
    // TODO: run(proxies, listener, handler(parser, mappper, builder));

    let handle = tokio::spawn({
        let (stream, addr) = listener
            .accept()
            .await
            .expect("Unable to establish TCP connection");

        let proxies = proxies.clone();

        async move {
            if handle_connection(stream, addr, proxies).await.is_err() {
                eprintln!("Unable to handle connection");
            }
        }
    });

    Ok(handle)
}

async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    proxies: Arc<Vec<Proxy>>,
) -> Result<(), Box<dyn Error>> {
    let (parsed_request, mut stream) = parse_incoming_request(stream).await?;

    let local_path = parsed_request.path.clone();
    let method = parsed_request.http_method.clone();

    let proxy_request = map_proxy_request(parsed_request, proxies)?;

    let remote_port = proxy_request.port;
    let remote_path = proxy_request.path.clone();

    let port = proxy_request.port;
    let request_str = build_proxy_request(proxy_request)?;

    let address = format!("{}:{}", addr.ip(), port);

    if let Ok(mut remote_stream) = TcpStream::connect(address).await {
        let log = format!(
            "{} {} => :{}{}",
            method, local_path, remote_port, remote_path
        );
        println!("{}", log);

        remote_stream.write_all(request_str.as_bytes()).await?;

        let mut response = String::new();

        remote_stream
            .read_to_string(&mut response)
            .await
            .expect("Unable to read from remote stream");

        stream.write_all(response.as_bytes()).await?;
    } else {
        eprintln!("Unable to connect to remote server");
    }

    Ok(())
}

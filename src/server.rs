use crate::{
    error::Error,
    settings::{ProxyConfig, Settings},
};
use hyper::{
    client::conn::http1::SendRequest,
    header::{HeaderName, HeaderValue},
    server, HeaderMap, Uri,
};
use hyper_util::rt::{TokioExecutor, TokioIo};
use lazy_static::lazy_static;
use native_tls::Identity;
use std::{fs, sync::Arc};
use tokio_native_tls::TlsAcceptor;

lazy_static! {
    static ref HOST_HEADER_NAME: HeaderName = HeaderName::from_static("host");
    static ref X_FORWARDED_FOR_HEADER_NAME: HeaderName =
        HeaderName::from_static("x-forwarded-for");
}

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::Incoming, service::service_fn, Request, Response};
use tokio::net::{TcpListener, TcpStream};

pub async fn start(
    listener: Arc<TcpListener>,
    settings: Arc<Settings>,
) -> Result<(), Error> {
    println!("Listening on: {}", listener.local_addr()?);
    println!("{}", settings);

    let pem = fs::read("localhost.crt").unwrap();
    let key = fs::read("localhost.key").unwrap();

    let cert = Identity::from_pkcs8(&pem, &key).unwrap();
    let tls_acceptor = native_tls::TlsAcceptor::builder(cert).build().unwrap();
    let tls_acceptor = tokio_native_tls::TlsAcceptor::from(tls_acceptor);

    loop {
        let settings = settings.clone();

        let (stream, _) = listener.clone().accept().await?;

        match settings.tls {
            true => spawn_tls_server(tls_acceptor.clone(), stream, settings),
            false => spawn_regular_server(stream, settings),
        }
    }
}

fn spawn_tls_server(
    tls_acceptor: TlsAcceptor,
    stream: TcpStream,
    settings: Arc<Settings>,
) {
    tokio::task::spawn(async move {
        let tls_stream =
            tls_acceptor.accept(stream).await.expect("accept error");

        let io = hyper_util::rt::TokioIo::new(tls_stream);

        if let Err(e) =
            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io,
                    service_fn(move |req| handle(req, settings.clone())),
                )
                .await
        {
            eprintln!("\x1b[31mERR\x1b[0m Error serving connection: {}", e);
        }
    });
}

fn spawn_regular_server(stream: TcpStream, settings: Arc<Settings>) {
    tokio::task::spawn(async move {
        let io = TokioIo::new(stream);

        if let Err(e) = server::conn::http1::Builder::new()
            .serve_connection(
                io,
                service_fn(move |req| handle(req, settings.clone())),
            )
            .await
        {
            eprintln!("\x1b[31mERR\x1b[0m Error serving connection: {}", e);
        }
    });
}

async fn handle(
    req: Request<Incoming>,
    settings: Arc<Settings>,
) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, Error> {
    let proxy = get_proxy(req.uri().path().to_string(), &settings.proxies);

    let addr = build_addr(&settings.host, proxy.remote_port);

    let stream = TcpStream::connect(addr).await?;

    let io = hyper_util::rt::TokioIo::new(stream);

    if let Some(upgrade) = req.headers().get(hyper::header::UPGRADE) {
        println!("upgrade header: {:#?}", upgrade);
    }

    let (client, connection) = hyper::client::conn::http1::Builder::new()
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!(
                "\x1b[31mERR\x1b[0m Unable to establish connection: {:?}",
                e
            );
        }
    });

    let request_uri = req.uri().clone();
    let request_method = req.method().clone();

    let proxy_request =
        build_request(req, &settings.host, settings.local_port, proxy)?;

    let proxy_uri = proxy_request.uri().clone();

    println!("sending request");
    let res = send_request(client, proxy_request).await?;
    println!("got response");
    let status = res.status().as_u16();

    println!(
        "{} {} {} \x1b[94m➡\x1b[0m :{}{}",
        colourise_status(status),
        request_method,
        request_uri.path(),
        proxy.remote_port,
        proxy_uri.path(),
    );

    Ok(res.map(|b| b.boxed()))
}

fn colourise_status(status_code: u16) -> String {
    match status_code {
        200..=399 => format!("\x1b[92m{}\x1b[0m", status_code),
        400..=499 => format!("\x1b[93m{}\x1b[0m", status_code),
        500..=599 => format!("\x1b[91m{}\x1b[0m", status_code),
        _ => status_code.to_string(),
    }
}

pub fn build_request(
    mut req: Request<Incoming>,
    host: &str,
    local_port: u16,
    proxy: &ProxyConfig,
) -> Result<Request<Incoming>, Error> {
    let local_addr = build_addr(host, local_port);
    let remote_addr = build_addr(host, proxy.remote_port);

    strip_hop_by_hop_headers(req.headers_mut());
    add_x_forwarded_for_header(req.headers_mut(), &local_addr)?;
    add_host_header(req.headers_mut(), &remote_addr)?;

    let mapped_uri = map_proxy_uri(req.uri(), proxy)?;
    *req.uri_mut() = mapped_uri;

    Ok(req)
}

pub async fn send_request(
    mut client: SendRequest<Incoming>,
    proxy_request: Request<Incoming>,
) -> Result<Response<Incoming>, hyper::Error> {
    let res = client.send_request(proxy_request).await?;

    Ok(res)
}

fn build_addr(hostname: &str, port: u16) -> String {
    format!("{}:{}", hostname, port)
}

fn strip_hop_by_hop_headers(headers: &mut HeaderMap) {
    headers.remove(hyper::header::CONNECTION);
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(hyper::header::PROXY_AUTHENTICATE);
    headers.remove(hyper::header::PROXY_AUTHORIZATION);
    headers.remove(hyper::header::TE);
    headers.remove(hyper::header::TRAILER);
    headers.remove(hyper::header::TRANSFER_ENCODING);
    headers.remove(hyper::header::UPGRADE);
}

fn add_x_forwarded_for_header(
    headers: &mut HeaderMap,
    local_addr: &str,
) -> Result<(), Error> {
    match headers.entry(&*X_FORWARDED_FOR_HEADER_NAME) {
        hyper::header::Entry::Vacant(v) => {
            v.insert(
                HeaderValue::from_str(local_addr)
                    .expect("`local_addr` should be valid as header value."),
            );
        }
        hyper::header::Entry::Occupied(mut v) => {
            v.insert(HeaderValue::from_str(
                &[
                    v.get()
                        .to_str()
                        .expect("Header valud to be parsable to string."),
                    local_addr,
                ]
                .join(", "),
            ).expect("Strings concatenated with a ', ' should be a valid header value."));
        }
    };

    Ok(())
}

fn add_host_header(
    headers: &mut HeaderMap,
    remote_addr: &str,
) -> Result<(), Error> {
    let host = HeaderValue::from_str(remote_addr)
        .expect("`remote_addr` should be valid as header value.");

    headers.insert(&*HOST_HEADER_NAME, host);

    Ok(())
}

fn get_proxy(req_uri: String, proxies: &[ProxyConfig]) -> &ProxyConfig {
    proxies
        .iter()
        .rfind(|x| req_uri.starts_with(&x.local_path))
        .unwrap()
}

pub fn map_proxy_uri(req_uri: &Uri, proxy: &ProxyConfig) -> Result<Uri, Error> {
    Ok(req_uri
        .to_string()
        .replace(&proxy.local_path, &proxy.remote_path)
        .parse::<hyper::Uri>()
        .unwrap())
}

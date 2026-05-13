use crate::settings::{ProxyConfig, Settings};
use anyhow::{Error, Result, anyhow};
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::header;
use hyper::header::Entry::{Occupied, Vacant};
use hyper::rt::{Read, Write};
use hyper::{
    HeaderMap, Uri,
    header::{HeaderName, HeaderValue},
};
use hyper::{Request, Response, body::Incoming, service::service_fn};
use hyper_util::rt::{TokioExecutor, TokioIo};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::marker::{Send, Unpin};
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;

static HOST_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("host"));

static X_FORWARDED_FOR_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("x-forwarded-for"));

pub async fn start(
    listener: Arc<TcpListener>,
    settings: Arc<Settings>,
) -> Result<(), Error> {
    println!("Listening on: {}", listener.local_addr()?);
    println!("{}", settings);

    match settings.tls {
        true => {
            let certs: Vec<CertificateDer<'static>> =
                CertificateDer::pem_file_iter(settings.pem.as_ref().unwrap())
                    .unwrap()
                    .collect::<Result<Vec<_>, _>>()
                    .unwrap();

            let private_key =
                PrivateKeyDer::from_pem_file(settings.key.as_ref().unwrap())
                    .unwrap();

            let config = ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(certs, private_key)?;

            let tls_acceptor = TlsAcceptor::from(Arc::new(config));

            loop {
                let settings = settings.clone();
                let (stream, _) = listener.accept().await?;

                match tls_acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);
                        spawn_server(io, settings)
                    }
                    Err(e) => eprintln!(
                        "\x1b[31mERR\x1b[0m TLS handshake failed: {}",
                        e
                    ),
                }
            }
        }
        false => loop {
            let settings = settings.clone();
            let (stream, _) = listener.accept().await?;
            let io = TokioIo::new(stream);

            spawn_server(io, settings);
        },
    }
}

fn spawn_server(
    io_stream: impl Read + Write + Unpin + Send + 'static,
    settings: Arc<Settings>,
) {
    tokio::task::spawn(async move {
        if let Err(e) =
            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io_stream,
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
    let Some(proxy) = get_proxy(req.uri().path(), &settings.proxies) else {
        return Ok(Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .body(
                http_body_util::Empty::<hyper::body::Bytes>::new()
                    .map_err(|e| match e {})
                    .boxed(),
            )
            .unwrap());
    };

    let stream = TcpStream::connect(&proxy.remote_addr).await?;

    let io = hyper_util::rt::TokioIo::new(stream);

    let (mut client, connection) = hyper::client::conn::http1::Builder::new()
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

    let proxy_request = build_request(req, &settings.local_addr, proxy)?;

    let proxy_uri = proxy_request.uri().clone();

    let res = client.send_request(proxy_request).await?;
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
    local_addr: &str,
    proxy: &ProxyConfig,
) -> Result<Request<Incoming>, Error> {
    strip_hop_by_hop_headers(req.headers_mut());
    add_x_forwarded_for_header(req.headers_mut(), local_addr);
    add_host_header(req.headers_mut(), &proxy.remote_addr);

    let mapped_uri = map_proxy_uri(req.uri(), proxy)?;
    *req.uri_mut() = mapped_uri;

    Ok(req)
}

fn strip_hop_by_hop_headers(headers: &mut HeaderMap) {
    headers.remove(header::CONNECTION);
    headers.remove(HeaderName::from_static("keep-alive"));
    headers.remove(header::PROXY_AUTHENTICATE);
    headers.remove(header::PROXY_AUTHORIZATION);
    headers.remove(header::TE);
    headers.remove(header::TRAILER);
    headers.remove(header::TRANSFER_ENCODING);
    headers.remove(header::UPGRADE);
}

fn add_x_forwarded_for_header(headers: &mut HeaderMap, local_addr: &str) {
    match headers.entry(&*X_FORWARDED_FOR_HEADER_NAME) {
        Vacant(v) => {
            v.insert(
                HeaderValue::from_str(local_addr)
                    .expect("`local_addr` should be valid as header value."),
            );
        }
        Occupied(mut v) => {
            v.insert(HeaderValue::from_str(
                &[
                    v.get()
                        .to_str()
                        .expect("Header value to be parsable to string."),
                    local_addr,
                ]
                .join(", "),
            ).expect("Strings concatenated with a ', ' should be a valid header value."));
        }
    };
}

fn add_host_header(headers: &mut HeaderMap, remote_addr: &str) {
    let host = HeaderValue::from_str(remote_addr)
        .expect("`remote_addr` should be valid as header value.");

    headers.insert(&*HOST_HEADER_NAME, host);
}

fn get_proxy<'a>(
    req_path: &str,
    proxies: &'a [ProxyConfig],
) -> Option<&'a ProxyConfig> {
    let req_segments = req_path
        .split('/')
        .filter(|p| !p.is_empty())
        .collect::<Vec<&str>>();

    proxies.iter().rfind(|p| {
        req_segments.starts_with(
            &p.local_path
                .split('/')
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>(),
        )
    })
}

pub fn map_proxy_uri(req_uri: &Uri, proxy: &ProxyConfig) -> Result<Uri, Error> {
    let local_path = proxy.local_path.trim_end_matches('/');
    let remote_path = proxy.remote_path.trim_end_matches('/');

    req_uri
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/")
        .strip_prefix(local_path)
        .map(|rest| {
            let path = format!("{}{}", remote_path, rest);
            if path.is_empty() {
                "/".to_string()
            } else {
                path
            }
        })
        .unwrap_or_else(|| format!("{}/", remote_path))
        .parse::<Uri>()
        .map_err(|e| anyhow!(e))
}

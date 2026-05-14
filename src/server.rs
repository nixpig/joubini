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

#[tracing::instrument]
pub async fn start(
    listener: Arc<TcpListener>,
    settings: Arc<Settings>,
) -> Result<(), Error> {
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
                let (stream, client_addr) = listener.accept().await?;

                match tls_acceptor.accept(stream).await {
                    Ok(tls_stream) => {
                        let io = TokioIo::new(tls_stream);
                        spawn_server(io, client_addr.ip().to_string(), settings)
                    }
                    Err(e) => {
                        tracing::error! { %e, "failed to complete TLS handshake" }
                    }
                }
            }
        }
        false => loop {
            let settings = settings.clone();
            let (stream, client_addr) = listener.accept().await?;
            let io = TokioIo::new(stream);

            spawn_server(io, client_addr.ip().to_string(), settings);
        },
    }
}

fn spawn_server(
    io_stream: impl Read + Write + Unpin + Send + 'static,
    client_addr: String,
    settings: Arc<Settings>,
) {
    tokio::task::spawn(async move {
        if let Err(e) =
            hyper_util::server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(
                    io_stream,
                    service_fn(move |req| {
                        handle(req, client_addr.clone(), settings.clone())
                    }),
                )
                .await
        {
            tracing::error! { %e, "failed to serve connection" };
        }
    });
}

#[tracing::instrument]
async fn handle(
    req: Request<Incoming>,
    client_addr: String,
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
            tracing::error! { %e, "failed to establish connection" };
        }
    });

    let request_uri = req.uri().clone();
    let request_method = req.method().clone();

    let proxy_request = build_request(req, &client_addr, proxy)?;

    let proxy_uri = proxy_request.uri().clone();

    let res = client.send_request(proxy_request).await?;
    let status = res.status().as_u16();
    let request_path = request_uri.path();
    let remote_port = proxy.remote_port;
    let proxy_path = proxy_uri.path();

    tracing::info! { %status, %request_method, %request_path, %remote_port, %proxy_path };

    Ok(res.map(|b| b.boxed()))
}

pub fn build_request(
    mut req: Request<Incoming>,
    client_addr: &str,
    proxy: &ProxyConfig,
) -> Result<Request<Incoming>, Error> {
    strip_hop_by_hop_headers(req.headers_mut());
    add_x_forwarded_for_header(req.headers_mut(), client_addr);
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

fn add_x_forwarded_for_header(headers: &mut HeaderMap, client_addr: &str) {
    match headers.entry(&*X_FORWARDED_FOR_HEADER_NAME) {
        Vacant(v) => {
            v.insert(
                HeaderValue::from_str(client_addr)
                    .expect("`client_addr` should be valid as header value."),
            );
        }
        Occupied(mut v) => {
            v.insert(HeaderValue::from_str(
                &[
                    v.get()
                        .to_str()
                        .expect("Header value to be parsable to string."),
                    client_addr,
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

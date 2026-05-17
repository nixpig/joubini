use crate::settings::{Proxy, Settings};
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
use tracing::Instrument;
use uuid::Uuid;

static HOST_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("host"));

static X_FORWARDED_FOR_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("x-forwarded-for"));

static KEEP_ALIVE_HEADER_NAME: LazyLock<HeaderName> =
    LazyLock::new(|| HeaderName::from_static("keep-alive"));

pub async fn start(
    listener: Arc<TcpListener>,
    settings: Arc<Settings>,
) -> Result<(), Error> {
    let id = Uuid::new_v4().to_string();

    match settings.tls.as_ref() {
        Some(tls) => {
            let certs: Vec<CertificateDer<'static>> =
                CertificateDer::pem_file_iter(&tls.pem)?
                    .collect::<Result<Vec<_>, _>>()?;

            let private_key = PrivateKeyDer::from_pem_file(&tls.private_key)?;

            let config = Arc::new(
                ServerConfig::builder()
                    .with_no_client_auth()
                    .with_single_cert(certs, private_key)?,
            );

            let tls_acceptor = TlsAcceptor::from(config);

            loop {
                let (stream, client_addr) = listener.accept().await?;
                let span = tracing::info_span!("connection", %id);

                match tls_acceptor.accept(stream).await {
                    Ok(tls_stream) => span.in_scope(|| {
                        tracing::info!("accepted");

                        spawn_server(
                            TokioIo::new(tls_stream),
                            client_addr.ip().to_string(),
                            Arc::clone(&settings),
                        )
                    }),
                    Err(e) => span.in_scope(|| {
                        tracing::error!(%e, "failed to complete TLS handshake")
                    })
                }
            }
        }
        None => loop {
            let (stream, client_addr) = listener.accept().await?;
            let span = tracing::info_span!("connection", %id);

            span.in_scope(|| {
                tracing::info!("accepted");

                spawn_server(
                    TokioIo::new(stream),
                    client_addr.ip().to_string(),
                    Arc::clone(&settings),
                );
            });
        },
    };
}

fn spawn_server(
    io_stream: impl Read + Write + Unpin + Send + 'static,
    client_addr: String,
    settings: Arc<Settings>,
) {
    let span = tracing::Span::current();

    tokio::task::spawn(
        async move {
            if let Err(e) = hyper_util::server::conn::auto::Builder::new(
                TokioExecutor::new(),
            )
            .serve_connection(
                io_stream,
                service_fn(move |req| {
                    handle(req, client_addr.clone(), Arc::clone(&settings))
                }),
            )
            .await
            {
                tracing::error!(%e, "failed to handle request");
            }
        }
        .instrument(span),
    );
}

async fn handle(
    req: Request<Incoming>,
    client_addr: String,
    settings: Arc<Settings>,
) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, Error> {
    let Some(proxy) = get_proxy(req.uri().path(), &settings.proxies) else {
        tracing::warn!(path = req.uri().path(), "no proxy configured for path");

        return Ok(Response::builder()
            .status(hyper::StatusCode::NOT_FOUND)
            .body(
                http_body_util::Empty::<hyper::body::Bytes>::new()
                    .map_err(|e| match e {})
                    .boxed(),
            )?);
    };

    let stream =
        TcpStream::connect(&proxy.remote_addr)
            .await
            .inspect_err(|e| {
                tracing::error!(%e, "failed to connect to upstream");
            })?;

    let io = hyper_util::rt::TokioIo::new(stream);

    let (mut client, connection) = hyper::client::conn::http1::Builder::new()
        .handshake(io)
        .await
        .inspect_err(|e| {
            tracing::error!(%e, "failed to complete handshake");
        })?;

    tokio::task::spawn(
        async move {
            if let Err(e) = connection.await {
                tracing::error!(%e, "failed to establish connection");
            }
        }
        .in_current_span(),
    );

    let request_uri = req.uri().clone();
    let request_method = req.method().clone();

    let proxy_request =
        build_request(req, &client_addr, proxy).inspect_err(|e| {
            tracing::error!(%e, "failed to build request");
        })?;

    let proxy_uri = proxy_request.uri().clone();

    let res = client.send_request(proxy_request).await.inspect_err(|e| {
        tracing::error!(%e, "failed to send request");
    })?;

    tracing::info!(
        status = %res.status(),
        %request_method,
        request_path = request_uri.path(),
        remote_addr = %proxy.remote_addr,
        proxy_path = %proxy_uri.path(),
    );

    Ok(res.map(|b| b.boxed()))
}

pub fn build_request(
    mut req: Request<Incoming>,
    client_addr: &str,
    proxy: &Proxy,
) -> Result<Request<Incoming>, Error> {
    strip_hop_by_hop_headers(req.headers_mut());
    add_x_forwarded_for_header(req.headers_mut(), client_addr);
    add_host_header(req.headers_mut(), &proxy.remote_addr);

    let mapped_uri = map_proxy_uri(req.uri(), proxy)?;
    *req.uri_mut() = mapped_uri;

    Ok(req)
}

fn strip_hop_by_hop_headers(headers: &mut HeaderMap) {
    let connection_headers: Vec<String> = headers
        .get(header::CONNECTION)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.split(',').map(|h| h.trim().to_owned()).collect())
        .unwrap_or_default();

    connection_headers.iter().for_each(|h| {
        headers.remove(h);
    });

    headers.remove(header::CONNECTION);
    headers.remove(&*KEEP_ALIVE_HEADER_NAME);
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
            let combined = v
                .get()
                .to_str()
                .map(|existing| format!("{existing}, {client_addr}"))
                .unwrap_or(client_addr.to_string());

            v.insert(
                HeaderValue::from_str(&combined)
                    .expect("should be valid as header value."),
            );
        }
    };
}

fn add_host_header(headers: &mut HeaderMap, remote_addr: &str) {
    let host = HeaderValue::from_str(remote_addr)
        .expect("`remote_addr` should be valid as header value.");

    headers.insert(&*HOST_HEADER_NAME, host);
}

fn get_proxy<'a>(req_path: &str, proxies: &'a [Proxy]) -> Option<&'a Proxy> {
    let req_segments = req_path
        .split('/')
        .filter(|p| !p.is_empty())
        .map(str::to_owned)
        .collect::<Vec<String>>();

    proxies
        .iter()
        .rfind(|p| req_segments.starts_with(&p.normalised_local_path))
}

pub fn map_proxy_uri(req_uri: &Uri, proxy: &Proxy) -> Result<Uri, Error> {
    let remote_path = proxy.remote_path.trim_end_matches('/');

    let remaining = req_uri
        .path()
        .split('/')
        .filter(|s| !s.is_empty())
        .skip(proxy.normalised_local_path.len())
        .collect::<Vec<_>>()
        .join("/");

    let query = req_uri.query().map(|q| format!("?{q}")).unwrap_or_default();

    let path = if remaining.is_empty() {
        format!("{remote_path}{query}")
    } else {
        format!("{remote_path}/{remaining}{query}")
    };

    let path = if path.is_empty() {
        "/".to_string()
    } else {
        path
    };

    path.parse::<Uri>().map_err(|e| anyhow!(e))
}

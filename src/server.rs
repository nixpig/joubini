use crate::{
    error::Error,
    settings::{ProxyConfig, Settings},
};
use hyper::{
    client::conn::http1::SendRequest,
    header::{HeaderName, HeaderValue},
    HeaderMap, Uri,
};
use lazy_static::lazy_static;
use std::sync::Arc;
use tracing::{error, info};

lazy_static! {
    static ref HOST_HEADER_NAME: HeaderName = HeaderName::from_static("host");
    static ref X_FORWARDED_FOR_HEADER_NAME: HeaderName =
        HeaderName::from_static("x-forwarded-for");
}

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{body::Incoming, server, service::service_fn, Request, Response};
use tokio::net::{TcpListener, TcpStream};

pub async fn start(
    listener: Arc<TcpListener>,
    settings: Arc<Settings>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("Listening on: {}", listener.local_addr().unwrap());

    loop {
        let (stream, _) = listener.clone().accept().await?;

        let io = hyper_util::rt::TokioIo::new(stream);

        let settings = settings.clone();

        tokio::task::spawn(async move {
            if let Err(err) = server::conn::http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| handle(req, settings.clone())),
                )
                .await
            {
                error!("Error serving connection: {}", err);
            }
        });
    }
}

async fn handle(
    req: Request<Incoming>,
    settings: Arc<Settings>,
) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, hyper::Error> {
    let proxy = get_proxy(req.uri().to_string(), &settings.proxies);

    let addr = build_addr(&settings.host, proxy.remote_port);

    let stream = TcpStream::connect(addr).await.unwrap();

    let io = hyper_util::rt::TokioIo::new(stream);

    let (client, connection) = hyper::client::conn::http1::Builder::new()
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = connection.await {
            error!("Unable to establish connection: {:?}", err);
        }
    });

    let request_uri = req.uri().clone();
    let request_method = req.method().clone();

    let proxy_request =
        build_request(req, &settings.host, settings.local_port, proxy).unwrap();

    let proxy_uri = proxy_request.uri().clone();

    let res = send_request(client, proxy_request).await?;

    info!(
        "{} {} {} => :{}{}",
        res.status().as_u16(),
        request_method,
        request_uri,
        proxy.remote_port,
        proxy_uri
    );

    Ok(res.map(|b| b.boxed()))
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
    add_x_forwarded_for_header(req.headers_mut(), &local_addr);
    add_host_header(req.headers_mut(), &remote_addr);

    match map_proxy_uri(req.uri(), proxy) {
        Ok(v) => {
            *req.uri_mut() = v;
            Ok(req)
        }
        Err(e) => Err(Error::RequestBuildError(format!(
            "URI mapping failed: {}",
            e
        ))),
    }
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

fn add_x_forwarded_for_header(headers: &mut HeaderMap, local_addr: &str) {
    match headers.entry(&*X_FORWARDED_FOR_HEADER_NAME) {
        hyper::header::Entry::Vacant(v) => {
            v.insert(HeaderValue::from_str(local_addr).unwrap());
        }
        hyper::header::Entry::Occupied(mut v) => {
            v.insert(
                HeaderValue::from_str(
                    &[v.get().to_str().unwrap(), local_addr].join(", "),
                )
                .unwrap(),
            );
        }
    };
}

fn add_host_header(headers: &mut HeaderMap, remote_addr: &str) {
    headers.insert(
        &*HOST_HEADER_NAME,
        HeaderValue::from_str(remote_addr).unwrap(),
    );
}

fn get_proxy(req_uri: String, proxies: &[ProxyConfig]) -> &ProxyConfig {
    proxies
        .iter()
        .rfind(|x| req_uri.starts_with(&x.local_path))
        .expect("Unable to find proxy config")
}

fn map_proxy_uri(req_uri: &Uri, proxy: &ProxyConfig) -> Result<Uri, Error> {
    match req_uri
        .to_string()
        .replace(&proxy.local_path, &proxy.remote_path)
        .parse()
    {
        Ok(v) => Ok(v),
        Err(e) => Err(Error::ProxyUriMapError(e)),
    }
}

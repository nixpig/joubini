use std::{net::SocketAddr, sync::Arc};

use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::Incoming,
    client::conn::http1::SendRequest,
    header::{HeaderName, HeaderValue},
    Request, Response,
};
use lazy_static::lazy_static;
use tokio::net::TcpStream;

lazy_static! {
    static ref HOST_HEADER_NAME: HeaderName = HeaderName::from_static("host");
    static ref X_FORWARDED_FOR_HEADER_NAME: HeaderName =
        HeaderName::from_static("x-forwarded-for");
}

pub async fn handle(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<hyper::body::Bytes, hyper::Error>>, hyper::Error> {
    println!("incoming req: {:#?}", req);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let stream = TcpStream::connect(addr).await.unwrap();

    let io = hyper_util::rt::TokioIo::new(stream);

    let (client, connection) = hyper::client::conn::http1::Builder::new()
        .handshake(io)
        .await?;

    tokio::task::spawn(async move {
        if let Err(err) = connection.await {
            println!("Unable to establish connection: {:?}", err);
        }
    });

    let proxy_request = build_request(req).unwrap();

    let res = send_request(client, proxy_request).await?;

    Ok(res.map(|b| b.boxed()))
}

fn build_request(
    mut req: Request<Incoming>,
) -> Result<Request<Incoming>, hyper::Error> {
    match req.headers_mut().entry(&*X_FORWARDED_FOR_HEADER_NAME) {
        hyper::header::Entry::Vacant(v) => {
            v.insert(HeaderValue::from_str("127.0.0.1:3000").unwrap());
        }
        hyper::header::Entry::Occupied(mut v) => {
            v.insert(
                HeaderValue::from_str(
                    &[v.get().to_str().unwrap(), "127.0.0.1:3000"].join(", "),
                )
                .unwrap(),
            );
        }
    };

    req.headers_mut().insert(
        &*HOST_HEADER_NAME,
        HeaderValue::from_static("127.0.0.1:3000"),
    );

    Ok(req)
}

async fn send_request(
    mut client: SendRequest<Incoming>,
    proxy_request: Request<Incoming>,
) -> Result<Response<Incoming>, hyper::Error> {
    println!("proxy req: {:#?}", proxy_request);
    let res = client.send_request(proxy_request).await?;
    Ok(res)
}

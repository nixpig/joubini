use std::{error::Error, net::SocketAddr};

use hyper::{server, service::service_fn};
use tokio::net::TcpListener;

use crate::{proxy::handle, settings::Settings};

const LOCAL_PORT: u16 = 7878;

pub async fn start(
    settings: Settings,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], LOCAL_PORT));

    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;

        let io = hyper_util::rt::TokioIo::new(stream);

        tokio::task::spawn(async move {
            if let Err(err) = server::conn::http1::Builder::new()
                .serve_connection(io, service_fn(move |req| handle(req)))
                .await
            {
                println!("Error serving connection: {}", err);
            }
        });
    }
}

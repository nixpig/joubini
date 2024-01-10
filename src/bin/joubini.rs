use std::{error::Error, net::SocketAddr, sync::Arc};

use joubini::{server::start, settings::get_settings};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let settings = Arc::new(get_settings());

    let addr = SocketAddr::from(([127, 0, 0, 1], settings.local_port.unwrap()));

    let listener = Arc::new(TcpListener::bind(addr).await?);

    start(listener, settings).await
}

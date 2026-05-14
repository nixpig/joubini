use std::sync::Arc;

use anyhow::Error;
use joubini::{server::start, settings::get_settings};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_level(true)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let settings = Arc::new(get_settings(std::env::args_os().collect())?);

    let bind_addr = format!("{}:{}", settings.host, settings.local_port);
    let listener = Arc::new(TcpListener::bind(bind_addr).await?);

    start(listener.clone(), settings.clone()).await
}

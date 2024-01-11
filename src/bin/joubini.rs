use std::{error::Error, sync::Arc};

use joubini::{server::start, settings::get_settings};
use tokio::net::TcpListener;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let log_filter = std::env::var("RUST_LOG").unwrap_or("info".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let settings = Arc::new(get_settings());

    let listener = Arc::new(
        TcpListener::bind(format!("{}:{}", settings.host, settings.local_port))
            .await?,
    );

    start(listener.clone(), settings.clone()).await
}

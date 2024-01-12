use std::sync::Arc;

use joubini::{
    error::{Error, IoError},
    server::start,
    settings::get_settings,
};
use tokio::net::TcpListener;
use tracing::error;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let log_filter = std::env::var("RUST_LOG").unwrap_or("info".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    match get_settings() {
        Ok(settings) => {
            let settings = Arc::new(settings);

            match TcpListener::bind(format!(
                "{}:{}",
                settings.host, settings.local_port
            ))
            .await
            {
                Ok(listener) => {
                    let listener = Arc::new(listener);
                    start(listener.clone(), settings.clone()).await
                }
                Err(e) => {
                    error!("Unable to bind to local port");
                    Err(Error::IoError(IoError::StdIo(e)))
                }
            }
        }
        Err(e) => {
            error!("Unable to get settings");
            Err(e)
        }
    }
}

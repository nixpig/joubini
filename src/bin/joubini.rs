use joubini::settings::get_settings;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    let log_filter = std::env::var("RUST_LOG").unwrap_or("info".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let settings = get_settings();

    debug!("{:#?}", settings);

    let proxies = Arc::new(settings.proxies);

    let listener = Arc::new(
        TcpListener::bind(format!(
            "{}:{}",
            settings.hostname, settings.local_port
        ))
        .await
        .expect("Could not bind to port"),
    );

    loop {
        joubini::startup::run(listener.clone(), proxies.clone())
            .await
            .expect("Should be able to start app loop");
    }
}

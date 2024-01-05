use joubini::settings::get_settings;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::fmt::format::FmtSpan;

#[tokio::main]
async fn main() {
    // let p1 = Proxy::new(String::from(":3000"));
    // let p2 = Proxy::new(String::from("api:3001/api"));
    // let p3 = Proxy::new(String::from("admin:3002/dashboard"));
    // let p4 = Proxy::new(String::from("db:3003"));
    // let p5 = Proxy::new(String::from("deep:3004/deep/nested/path"));

    let log_filter = std::env::var("RUST_LOG").unwrap_or("info".to_owned());

    tracing_subscriber::fmt()
        .with_env_filter(log_filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let settings = get_settings();

    debug!("{:#?}", settings);

    let proxies = Arc::new(settings.proxies);

    let listener = TcpListener::bind(format!(
        "{}:{}",
        settings.hostname, settings.local_port
    ))
    .await
    .expect("Could not bind to port");

    loop {
        joubini::startup::run(&listener, proxies.clone())
            .await
            .expect("Should be able to start app loop");
    }
}

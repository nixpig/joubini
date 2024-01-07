use joubini::settings::get_settings;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let settings = get_settings();

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

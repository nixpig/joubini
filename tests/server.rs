use joubini::server::start;
use joubini::settings::{ProxyConfig, Settings};
use serial_test::serial;
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;

#[serial]
#[tokio::test]
async fn test_fail_when_no_remote_server() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str(":5000").unwrap()],
    };

    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client.get("http://127.0.0.1:7878").send().await;

    assert!(response.is_err());

    Ok(())
}

async fn start_joubini(settings: Settings) {
    let listener = Arc::new(
        TcpListener::bind(format!(
            "127.0.0.1:{}",
            settings.local_port.unwrap()
        ))
        .await
        .expect("Unable to bind to local port"),
    );

    let settings = Arc::new(settings);

    tokio::spawn(async move {
        loop {
            start(listener.clone(), settings.clone())
                .await
                .expect("Unable to start server");
        }
    });
}

use std::{error::Error, str::FromStr, sync::Arc};

use actix_web::{web, App, HttpResponse, HttpServer};
use joubini::{proxy::Proxy, settings::Settings};
use reqwest::StatusCode;
use serial_test::serial;
use tokio::net::TcpListener;

#[serial]
#[tokio::test]
async fn test_only_port_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![
            Proxy::from_str(":3000").expect("Unable to parse proxy string")
        ],
    };

    start_server(3000, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878")
        .send()
        .await
        .expect("Request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_path_to_port_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![
            Proxy::from_str("foo:3001").expect("Unable to parse proxy string")
        ],
    };

    start_server(3001, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/foo")
        .send()
        .await
        .expect("Request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_path_to_path_mapping() {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![Proxy::from_str("bar:3002/bar")
            .expect("Unable to parse proxy string")],
    };

    start_server(3002, "/bar").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/bar")
        .send()
        .await
        .expect("HTTP request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");
}

#[serial]
#[tokio::test]
async fn test_rename_path_mapping() {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![Proxy::from_str("baz:3003/qux")
            .expect("Unable to parse proxy string")],
    };

    start_server(3003, "/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/baz")
        .send()
        .await
        .expect("HTTP request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");
}

#[serial]
#[tokio::test]
async fn test_shallow_to_deep_path_mapping() {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![Proxy::from_str("foo:3004/bar/baz/qux")
            .expect("Unable to parse proxy config from string")],
    };

    start_server(3004, "/bar/baz/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/foo")
        .send()
        .await
        .expect("Unable to make HTTP request");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");
}

#[serial]
#[tokio::test]
async fn test_deep_to_shallow_path_mapping() {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![Proxy::from_str("foo/bar/baz:3005/qux")
            .expect("Unable to parse proxy settings from provided string")],
    };

    start_server(3005, "/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/foo/bar/baz")
        .send()
        .await
        .expect("Failed to send HTTP request");

    let status = response.status();

    let body = response.text().await.expect("Could not get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get ok");
}

#[serial]
#[tokio::test]
async fn test_nested_matching_path_mappings() {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![
            Proxy::from_str("foo/bar:3006/baz")
                .expect("unable to parse proxy string"),
            Proxy::from_str("foo/qux:3007/thud")
                .expect("unable to parse proxy string"),
            Proxy::from_str("foo:3008/fred")
                .expect("unable to parse proxy string"),
        ],
    };

    start_server(3006, "/baz").await;
    start_server(3007, "/thud").await;
    start_server(3008, "/fred").await;

    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response_foo = client
        .get("http://localhost:7878/foo")
        .send()
        .await
        .expect("HTTP request failed");

    let response_foo_bar = client
        .get("http://localhost:7878/foo/bar")
        .send()
        .await
        .expect("HTTP request failed");

    let response_foo_qux = client
        .get("http://localhost:7878/foo/qux")
        .send()
        .await
        .expect("HTTP request failed");

    let status_foo = response_foo.status();
    let status_foo_bar = response_foo_bar.status();
    let status_foo_qux = response_foo_qux.status();

    let body_foo = response_foo.text().await.expect("Unable to get body");
    let body_foo_bar =
        response_foo_bar.text().await.expect("Unable to get body");
    let body_foo_qux =
        response_foo_qux.text().await.expect("Unable to get body");

    assert_eq!(status_foo, StatusCode::OK);
    assert_eq!(status_foo_bar, StatusCode::OK);
    assert_eq!(status_foo_qux, StatusCode::OK);

    assert_eq!(body_foo, "get ok");
    assert_eq!(body_foo_bar, "get ok");
    assert_eq!(body_foo_qux, "get ok");
}

// #[tokio::test]
// async fn test_multiple_mappings() {
//     Proxy::from_str(":3000").expect("unable to parse proxy string");
//
//     todo!()
// }
//

// TODO: test different content types
// TODO: test different HTTP verbs - POST, PUT, DELETE, etc...
// TODO: test different responses - 200, 201, 302, 404, 500, etc...

async fn start_joubini(settings: Settings) {
    let listener = Arc::new(
        TcpListener::bind(format!(
            "{}:{}",
            settings.hostname, settings.local_port
        ))
        .await
        .expect("Could not bind to port"),
    );

    let proxies = Arc::new(settings.proxies);

    tokio::spawn(async move {
        loop {
            joubini::startup::run(listener.clone(), proxies.clone())
                .await
                .expect("should be able to start the app");
        }
    });
}

async fn start_server(port: u16, path: &'static str) {
    let listener = std::net::TcpListener::bind(format!("localhost:{}", port))
        .expect("Unable to bind to port");

    let data = actix_web::web::Data::new(path);

    let server = HttpServer::new(move || {
        App::new()
            .route(path, web::get().to(get_ok))
            .app_data(data.clone())
    })
    .listen(listener)
    .expect("Unable to bind to listener")
    .run();

    tokio::spawn(server);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct Response {
    message: String,
}

async fn get_ok(data: actix_web::web::Data<&str>) -> HttpResponse {
    HttpResponse::Ok().body("get ok")
}

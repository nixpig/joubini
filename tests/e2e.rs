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
            Proxy::from_str("api:3000").expect("unable to parse proxy string")
        ],
    };

    start_server(3000, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:7878/api")
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
            Proxy::from_str("foo:3001").expect("unable to parse proxy string")
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

// #[tokio::test]
// async fn test_rename_path_mapping() {
//     Proxy::from_str("admin:3002/dashboard")
//         .expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_port_to_path_mapping() {
//     Proxy::from_str("db:3003").expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_shallow_to_deep_path_mapping() {
//     Proxy::from_str("shallow:3004/some/deep/path")
//         .expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_deep_to_shallow_path_mapping() {
//     Proxy::from_str("some/deep/path:3005")
//         .expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_multiple_matching_path_mappings() {
//     Proxy::from_str("myapp/api:3001/api")
//         .expect("unable to parse proxy string");
//     Proxy::from_str("myapp:3000/ui").expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_multiple_mappings() {
//     Proxy::from_str(":3000").expect("unable to parse proxy string");
//
//     todo!()
// }
//
// #[tokio::test]
// async fn test_app_start() {
//     let settings = Settings {
//         hostname: String::from("localhost"),
//         local_port: 7878,
//         proxies: vec![],
//     };
//
//     start_joubini(settings).await;
//
//     let client = reqwest::Client::new();
//
//     let response = client
//         .get("http://localhost:7878/zero")
//         .send()
//         .await
//         .expect("Request did not complete successfully.");
//
//     println!("response: {:#?}", response);
//
//     assert_eq!(1, 2);
// }

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
    println!("WEB DATA: {:?}", data);

    HttpResponse::Ok().body("get ok")
}

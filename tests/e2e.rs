use std::{collections::HashMap, error::Error, str::FromStr, sync::Arc};

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
async fn test_path_to_path_mapping() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_rename_path_mapping() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_shallow_to_deep_path_mapping() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_deep_to_shallow_path_mapping() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_nested_matching_path_mappings() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_post_json() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![
            Proxy::from_str(":3009").expect("Unable to parse proxy mapping")
        ],
    };

    start_server(3009, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    #[derive(serde::Serialize)]
    struct PostData {
        data: String,
    }

    let post_data = PostData {
        data: String::from("Some post data"),
    };

    let response = client
        .post("http://localhost:7878/json-post")
        .json(&post_data)
        .send()
        .await
        .expect("HTTP POST request failed");

    let status = response.status();

    let body: Response =
        response.json().await.expect("Unable to parse response");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        Response {
            message: String::from("POST JSON OK")
        }
    );

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_post_form() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        hostname: String::from("localhost"),
        local_port: 7878,
        proxies: vec![
            Proxy::from_str(":3010").expect("Unable to parse proxy config")
        ],
    };

    start_server(3010, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let mut form_data = HashMap::new();
    form_data.insert("form_key", "form_value");

    let response = client
        .post("http://localhost:7878/form-post")
        .form(&form_data)
        .send()
        .await
        .expect("Unable to post form data");

    let status = response.status();

    let body: Response =
        response.json().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        Response {
            message: String::from("POST FORM OK")
        }
    );

    Ok(())
}

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

    let server = HttpServer::new(move || {
        App::new()
            .route(path, web::get().to(get_ok))
            .route("/json-post", web::post().to(post_json_ok))
            .route("/form-post", web::post().to(post_form_ok))
    })
    .listen(listener)
    .expect("Unable to bind to listener")
    .run();

    tokio::spawn(server);
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
struct Response {
    message: String,
}

async fn get_ok() -> HttpResponse {
    HttpResponse::Ok().body("get ok")
}

async fn post_json_ok() -> HttpResponse {
    let json_ok = Response {
        message: String::from("POST JSON OK"),
    };

    HttpResponse::Ok().json(json_ok)
}

async fn post_form_ok() -> HttpResponse {
    let form_ok = Response {
        message: String::from("POST FORM OK"),
    };

    HttpResponse::Ok().json(&form_ok)
}

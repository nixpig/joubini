use actix_web::{web, App, HttpResponse, HttpServer};
use joubini::server::start;
use joubini::settings::{ProxyConfig, Settings};
use reqwest::StatusCode;
use serial_test::serial;
use std::collections::HashMap;
use std::error::Error;
use std::net::TcpListener;
use std::str::FromStr;
use std::sync::Arc;

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
struct ResponseData {
    message: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct PostData {
    data: String,
}

#[derive(PartialEq, Debug, serde::Serialize, serde::Deserialize)]
struct FormData {
    form_key: String,
}

#[serial]
#[tokio::test]
async fn test_post_json() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str(":3009").unwrap()],
    };

    start_remote(3009, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let post_data = PostData {
        data: String::from("post_data"),
    };

    let response = client
        .post("http://127.0.0.1:7878/json-post")
        .json(&post_data)
        .send()
        .await
        .expect("HTTP POST request failed");

    let status = response.status();
    assert_eq!(status, StatusCode::OK);

    let body: ResponseData =
        response.json().await.expect("Unable to parse response");

    assert_eq!(
        body,
        ResponseData {
            message: String::from("post_json_ok")
        }
    );

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_post_form() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str(":3010").unwrap()],
    };

    start_remote(3010, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let mut form_data = HashMap::new();
    form_data.insert("form_key", "form_value");

    let response = client
        .post("http://127.0.0.1:7878/form-post")
        .form(&form_data)
        .send()
        .await
        .expect("Unable to post form data");

    let status = response.status();

    let body: ResponseData =
        response.json().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        ResponseData {
            message: String::from("post_form_ok")
        }
    );

    Ok(())
}
// FLAKY WHEN RUN QUICKLY IN SUCCESSION!!
// #[serial]
// #[should_panic]
// #[tokio::test]
// async fn test_fail_when_no_remote_server() {
//     let settings = Settings {
//         local_port: Some(7878),
//         proxies: vec![ProxyConfig::from_str(":3000").unwrap()],
//     };
//
//     start_joubini(settings).await;
//
//     let client = reqwest::Client::new();
//
//     let _ = client.get("http://127.0.0.1:7878").send().await;
// }

#[serial]
#[tokio::test]
async fn test_only_port_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str(":3000").unwrap()],
    };

    start_remote(3000, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let res = client.get("http://127.0.0.1:7878").send().await?;

    let status = res.status();
    assert_eq!(status, StatusCode::OK);

    let body = res.text().await?;
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_path_to_port_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str("foo:3001")
            .expect("Unable to parse proxy string")],
    };

    start_remote(3001, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:7878/foo")
        .send()
        .await
        .expect("Request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_path_to_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str("bar:3002/bar")
            .expect("Unable to parse proxy string")],
    };

    start_remote(3002, "/bar").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:7878/bar")
        .send()
        .await
        .expect("HTTP request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_rename_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str("baz:3003/qux")
            .expect("Unable to parse proxy string")],
    };

    start_remote(3003, "/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:7878/baz")
        .send()
        .await
        .expect("HTTP request failed");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_shallow_to_deep_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str("foo:3004/bar/baz/qux")
            .expect("Unable to parse proxy config from string")],
    };

    start_remote(3004, "/bar/baz/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:7878/foo")
        .send()
        .await
        .expect("Unable to make HTTP request");

    let status = response.status();

    let body = response.text().await.expect("Unable to get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_deep_to_shallow_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![ProxyConfig::from_str("foo/bar/baz:3005/qux")
            .expect("Unable to parse proxy settings from provided string")],
    };

    start_remote(3005, "/qux").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client
        .get("http://127.0.0.1:7878/foo/bar/baz")
        .send()
        .await
        .expect("Failed to send HTTP request");

    let status = response.status();

    let body = response.text().await.expect("Could not get response body");

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_nested_matching_path_mappings() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        local_port: Some(7878),
        proxies: vec![
            ProxyConfig::from_str("foo:3008/fred")
                .expect("unable to parse proxy string"),
            ProxyConfig::from_str("foo/qux:3007/thud")
                .expect("unable to parse proxy string"),
            ProxyConfig::from_str("foo/bar:3006/baz")
                .expect("unable to parse proxy string"),
        ],
    };

    start_remote(3006, "/baz").await;
    start_remote(3007, "/thud").await;
    start_remote(3008, "/fred").await;

    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response_foo = client
        .get("http://localhost:7878/foo")
        .send()
        .await
        .expect("HTTP request failed");

    println!("response_foo: {:#?}", response_foo);

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

    assert_eq!(body_foo, "get_ok");
    assert_eq!(body_foo_bar, "get_ok");
    assert_eq!(body_foo_qux, "get_ok");

    Ok(())
}

async fn start_joubini(settings: Settings) {
    println!("starting joubini");
    let listener = Arc::new(
        tokio::net::TcpListener::bind(format!(
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

async fn start_remote(port: u16, path: &'static str) {
    println!("starting remote");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .expect("Unable to listen on port");

    println!("starting remote on 127.0.0.1:{}", port);

    let server = HttpServer::new(move || {
        App::new()
            .route(path, web::get().to(get_ok))
            .route("/json-post", web::post().to(post_json_ok))
            .route("/form-post", web::post().to(post_form_ok))
    })
    .listen(listener)
    .expect("Unable to start remote server")
    .run();

    tokio::spawn(server);
}

async fn get_ok() -> HttpResponse {
    HttpResponse::Ok().body("get_ok")
}

async fn post_json_ok(body: web::Json<PostData>) -> HttpResponse {
    if body.data == "post_data" {
        let json_ok = ResponseData {
            message: String::from("post_json_ok"),
        };

        HttpResponse::Ok().json(json_ok)
    } else {
        println!("Forwarded body data does not match provided body data");

        HttpResponse::InternalServerError().into()
    }
}

async fn post_form_ok(form: web::Form<FormData>) -> HttpResponse {
    let expected_form_data = web::Form(FormData {
        form_key: String::from("form_value"),
    });

    if form == expected_form_data {
        let form_ok = ResponseData {
            message: String::from("post_form_ok"),
        };

        HttpResponse::Ok().json(&form_ok)
    } else {
        HttpResponse::InternalServerError().into()
    }
}

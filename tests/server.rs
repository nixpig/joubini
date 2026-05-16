use actix_web::http::header::{self, HeaderMap};
use actix_web::{App, HttpRequest, HttpResponse, HttpServer, web};
use hyper::Uri;
use joubini::server::start;
use joubini::settings::{Proxy, Settings, TlsConfig};
use reqwest::StatusCode;
use serial_test::serial;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::net::TcpListener;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

static HOP_HEADERS: [hyper::header::HeaderName; 7] = [
    hyper::header::HeaderName::from_static("keep-alive"),
    hyper::header::PROXY_AUTHENTICATE,
    hyper::header::PROXY_AUTHORIZATION,
    hyper::header::TE,
    hyper::header::TRAILER,
    hyper::header::TRANSFER_ENCODING,
    hyper::header::UPGRADE,
];

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

#[test]
fn test_map_proxy_uri() -> Result<(), Box<dyn Error>> {
    let proxy = Proxy::new("foo:3000/baz", "localhost")?;

    let req_uri = Uri::from_static("/foo/bar");
    let mapped_uri = joubini::server::map_proxy_uri(&req_uri, &proxy)?;
    assert_eq!(mapped_uri, Uri::from_static("/baz/bar"));

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_headers_updated() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![Proxy::new(":3015", "localhost").unwrap()],
    };

    start_remote(3015, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let res = client
        .get("http://localhost:7878/headers")
        .header(hyper::header::HOST, "http://localhost")
        .header("keep-alive", "true")
        .header(hyper::header::PROXY_AUTHENTICATE, "")
        .header(hyper::header::PROXY_AUTHORIZATION, "")
        .header(hyper::header::TE, "")
        .header(hyper::header::TRAILER, "")
        .header(hyper::header::TRANSFER_ENCODING, "")
        .header(hyper::header::UPGRADE, "")
        .header("x-custom-header", "custom_header_value")
        .send()
        .await?;

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_fail_when_no_remote_server() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![Proxy::new(":3011", "localhost").unwrap()],
    };

    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let res = client.get("http://localhost:7878").send().await;

    assert!(res.is_err());

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_fail_when_no_proxy_specified() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![],
    };

    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let response = client.get("http://localhost:7878").send().await?;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_post_json() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![Proxy::new(":3009", "localhost").unwrap()],
    };

    start_remote(3009, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let post_data = PostData {
        data: String::from("post_data"),
    };

    let response = client
        .post("http://localhost:7878/json-post")
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
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![Proxy::new(":3010", "localhost").unwrap()],
    };

    start_remote(3010, "/").await;
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

#[serial]
#[tokio::test]
async fn test_only_port_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![Proxy::new(":3000", "localhost").unwrap()],
    };

    start_remote(3000, "/").await;
    start_joubini(settings).await;

    let client = reqwest::Client::new();

    let res = client.get("http://localhost:7878").send().await?;

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
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("foo:3001", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_remote(3001, "/").await;
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
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_path_to_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("bar:3002/bar", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_remote(3002, "/bar").await;
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
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_rename_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("baz:3003/qux", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_remote(3003, "/qux").await;
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
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_shallow_to_deep_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("foo:3004/bar/baz/qux", "localhost")
                .expect("Unable to parse proxy config from string"),
        ],
    };

    start_remote(3004, "/bar/baz/qux").await;
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
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_deep_to_shallow_path_mapping() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("foo/bar/baz:3005/qux", "localhost")
                .expect("Unable to parse proxy settings from provided string"),
        ],
    };

    start_remote(3005, "/qux").await;
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
    assert_eq!(body, "get_ok");

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_nested_matching_path_mappings() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new("foo:3008/fred", "localhost")
                .expect("unable to parse proxy string"),
            Proxy::new("foo/qux:3007/thud", "localhost")
                .expect("unable to parse proxy string"),
            Proxy::new("foo/bar:3006/baz", "localhost")
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

#[serial]
#[tokio::test]
async fn test_add_x_forwarded_for_header() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new(":3012", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_joubini(settings).await;
    start_remote(3012, "/").await;

    let res = reqwest::get("http://localhost:7878/add-forwarded")
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_append_x_forwarded_for_header() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,

        proxies: vec![
            Proxy::new(":3013", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_joubini(settings).await;
    start_remote(3013, "/").await;

    let client = reqwest::Client::new();

    let res = client
        .get("http://localhost:7878/append-forwarded")
        .header("x-forwarded-for", "127.0.0.1")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_response_codes() -> Result<(), Box<dyn Error>> {
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        tls: None,
        proxies: vec![
            Proxy::new(":3014", "localhost")
                .expect("Unable to parse proxy string"),
        ],
    };

    start_joubini(settings).await;
    start_remote(3014, "/").await;

    let client = reqwest::Client::new();

    let res = client
        .get("http://localhost:7878/301")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::MOVED_PERMANENTLY);

    let res = client
        .get("http://localhost:7878/500")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let res = client
        .get("http://localhost:7878/404")
        .send()
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    Ok(())
}

#[serial]
#[tokio::test]
async fn test_tls_server() -> Result<(), Box<dyn Error>> {
    // TODO: generate TLS cert on the fly
    let settings = Settings {
        host: String::from("localhost"),
        local_port: 7878,
        proxies: vec![Proxy::new(":3016", "localhost").unwrap()],
        tls: Some(TlsConfig {
            pem: PathBuf::from_str("/tmp/localhost.crt").unwrap(),
            private_key: PathBuf::from_str("/tmp/localhost.key").unwrap(),
        }),
    };

    start_remote(3016, "/").await;
    start_joubini(settings).await;

    let pem = fs::read("/tmp/localhost.crt").unwrap();
    let key = fs::read("/tmp/localhost.key").unwrap();
    let root = fs::read("/tmp/localCA.pem").unwrap();

    let mut combined = pem.clone();
    combined.extend_from_slice(&key);
    let cert = reqwest::Identity::from_pem(&combined)?;
    let ca = reqwest::Certificate::from_pem(&root)?;

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .identity(cert)
        .add_root_certificate(ca)
        .pool_max_idle_per_host(0)
        .build()?;

    let res = client.get("https://localhost:7878").send().await?;

    let status = res.status();
    assert_eq!(status, reqwest::StatusCode::OK);

    let body = res.text().await?;
    assert_eq!(body, "get_ok");

    Ok(())
}

// TODO: http2
// let client = reqwest::Client::builder()
//     .use_native_tls()
//     .identity(cert)
//     .add_root_certificate(ca)
//     .pool_max_idle_per_host(0)
// .http2_prior_knowledge()
//     .build()?;

async fn start_joubini(settings: Settings) {
    let listener = Arc::new(
        tokio::net::TcpListener::bind(format!(
            "localhost:{}",
            settings.local_port
        ))
        .await
        .expect("Unable to bind to local port"),
    );

    let settings = Arc::new(settings);

    tokio::spawn(async move {
        start(listener.clone(), settings.clone())
            .await
            .expect("Unable to start server");
    });
}

async fn start_remote(port: u16, path: &'static str) {
    let listener = TcpListener::bind(format!("localhost:{}", port))
        .expect("Unable to listen on port");

    let server = HttpServer::new(move || {
        App::new()
            .route(path, web::get().to(get_ok))
            .route("/json-post", web::post().to(post_json_ok))
            .route("/form-post", web::post().to(post_form_ok))
            .route("/add-forwarded", web::get().to(add_forwarded_ok))
            .route("/append-forwarded", web::get().to(append_forwarded_ok))
            .route("/301", web::get().to(handler_301))
            .route("/404", web::get().to(handler_404))
            .route("/500", web::get().to(handler_500))
            .route("/headers", web::get().to(headers_ok))
    })
    .listen(listener)
    .expect("Unable to start remote server")
    .run();

    tokio::spawn(server);
}

async fn handler_301() -> HttpResponse {
    HttpResponse::MovedPermanently().finish()
}

async fn handler_404() -> HttpResponse {
    HttpResponse::NotFound().finish()
}

async fn handler_500() -> HttpResponse {
    HttpResponse::InternalServerError().finish()
}

async fn get_ok() -> HttpResponse {
    HttpResponse::Ok().body("get_ok")
}

async fn headers_ok(req: HttpRequest) -> HttpResponse {
    let headers: &HeaderMap = req.headers();

    let hop_headers: Vec<&hyper::header::HeaderName> = HOP_HEADERS
        .iter()
        .filter(|h| headers.get(h.to_string()).is_some())
        .collect();

    if !hop_headers.is_empty() {
        return HttpResponse::InternalServerError().finish();
    }

    if headers.get("x-custom-header").unwrap() != "custom_header_value" {
        return HttpResponse::InternalServerError().finish();
    }

    if headers.get(header::HOST).unwrap() != "localhost:3015" {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

async fn post_json_ok(body: web::Json<PostData>) -> HttpResponse {
    if body.data == "post_data" {
        let json_ok = ResponseData {
            message: String::from("post_json_ok"),
        };

        HttpResponse::Ok().json(json_ok)
    } else {
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

async fn add_forwarded_ok(req: HttpRequest) -> HttpResponse {
    let x_forwarded_for_header = req.headers().get(header::X_FORWARDED_FOR);

    if x_forwarded_for_header.is_some_and(|h| h == "::1") {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::ImATeapot().finish()
    }
}

async fn append_forwarded_ok(req: HttpRequest) -> HttpResponse {
    let x_forwarded_for_header = req.headers().get(header::X_FORWARDED_FOR);

    if x_forwarded_for_header.is_some_and(|h| h == "127.0.0.1, ::1") {
        HttpResponse::Ok().finish()
    } else {
        HttpResponse::ImATeapot().finish()
    }
}

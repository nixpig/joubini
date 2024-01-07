use crate::proxy::Request;
use std::error::Error;

pub fn build_proxy_request(request: Request) -> Result<String, Box<dyn Error>> {
    let mut request_str = String::new();

    let request_line = format!(
        "{} {} {}",
        request.http_method, request.path, request.http_version
    );
    request_str.push_str(&request_line);
    request_str.push_str("\r\n");

    let headers = request
        .headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\r\n");

    request_str.push_str(&headers);

    request_str.push_str("\r\nConnection: close\r\n\r\n");

    if let Some(body) = request.body {
        request_str.push_str(&body);
    }

    Ok(request_str)
}

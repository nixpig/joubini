use std::error::Error;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    net::TcpStream,
};

use crate::proxy::Request;

pub async fn parse_incoming_request(
    mut incoming_request_stream: TcpStream,
) -> Result<(Request, TcpStream), Box<dyn Error>> {
    let mut request = Request::new();

    let mut reader = BufReader::new(&mut incoming_request_stream);

    let mut request_line = String::new();

    reader.read_line(&mut request_line).await?;

    let request_line_parts: Vec<&str> = request_line.split(' ').collect();

    request.http_method = String::from(request_line_parts[0].trim());
    request.path = String::from(request_line_parts[1].trim());
    request.http_version = String::from(request_line_parts[2].trim());

    loop {
        let mut header_line = String::new();

        reader.read_line(&mut header_line).await?;

        if header_line.trim().is_empty() {
            break;
        }

        if let Some((header_name, header_value)) = header_line.split_once(':') {
            request.headers.insert(
                String::from(header_name.to_lowercase().trim()),
                String::from(header_value.trim()),
            );
        }

        request
            .headers
            .insert(String::from("host"), String::from("localhost:3000"));
    }

    if let Some(content_length) = request.headers.get("content-length") {
        let mut body_buffer =
            vec![0u8; content_length.parse::<usize>().unwrap()];

        reader.read_exact(&mut body_buffer).await?;

        request.body =
            Some(String::from(std::str::from_utf8(&body_buffer).unwrap()));
    }

    Ok((request, incoming_request_stream))
}

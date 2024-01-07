use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub struct Request {
    pub http_method: String,
    pub http_version: String,
    pub path: String,
    pub port: u16,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn new() -> Request {
        Request {
            http_method: String::from(""),
            http_version: String::from(""),
            path: String::from(""),
            port: 80,
            headers: HashMap::new(),
            body: None,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Proxy {
    pub local_path: String,
    pub remote_path: String,
    pub remote_port: u16,
}

#[derive(Debug)]
pub struct ProxyParseError;

impl FromStr for Proxy {
    type Err = ProxyParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((local, remote)) = s.split_once(':') {
            let (port, path) = if let Some((p1, p2)) = remote.split_once('/') {
                (p1, p2)
            } else {
                (remote, "")
            };

            return Ok(Proxy {
                local_path: ["/", local].join(""),
                remote_path: ["/", path].join(""),
                remote_port: port.parse::<u16>().unwrap(),
            });
        }

        Err(ProxyParseError)
    }
}

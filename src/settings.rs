use crate::cli::Cli;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use serde::Deserialize;
use std::ffi::OsString;
use std::{fs, path::PathBuf};

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq, Deserialize)]
pub struct TlsConfig {
    pub pem: PathBuf,
    pub private_key: PathBuf,
}

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq, Deserialize)]
pub struct Settings {
    pub host: String,
    pub local_port: u16,
    pub proxies: Vec<Proxy>,
    pub tls: Option<TlsConfig>,
}

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq, Deserialize)]
pub struct Proxy {
    pub local_path: String,
    pub remote_port: u16,
    pub remote_path: String,
    pub remote_addr: String,
}

impl Proxy {
    pub fn new(s: &str, host: &str) -> Result<Self, Error> {
        let Some((local_path, remote)) = s.split_once(':') else {
            return Err(anyhow!("Unable to parse proxy definition."));
        };

        let (remote_port, remote_path) = match remote.split_once('/') {
            Some((port, path)) => (port, path),
            None => (remote, ""),
        };

        let remote_port = remote_port.parse::<u16>()?;

        Ok(Proxy {
            local_path: format!("/{}", local_path),
            remote_port,
            remote_path: format!("/{}", remote_path),
            remote_addr: format!("{}:{}", host, remote_port),
        })
    }
}

#[derive(Default, Debug, Deserialize)]
struct Config {
    host: Option<String>,
    #[serde(rename = "port")]
    local_port: Option<u16>,
    proxies: Vec<String>,
    tls: Option<bool>,
    pem: Option<PathBuf>,
    key: Option<PathBuf>,
}

impl TryFrom<PathBuf> for Config {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let config_str = fs::read_to_string(&path)?;
        let config_yaml = serde_yaml::from_str(&config_str)?;
        Ok(config_yaml)
    }
}

pub fn get_settings(cli_args: Vec<OsString>) -> Result<Settings, Error> {
    let cli_config = Cli::parse_from(cli_args);

    let file_config = cli_config
        .config
        .as_ref()
        .map(|p| Config::try_from(PathBuf::from(p)))
        .transpose()?
        .unwrap_or_default();

    let host = cli_config
        .host
        .or(file_config.host)
        .unwrap_or(String::from("127.0.0.1"));

    let local_port = cli_config
        .local_port
        .or(file_config.local_port)
        .unwrap_or(if cli_config.tls { 443 } else { 80 });

    let tls = cli_config.tls || file_config.tls.unwrap_or(false);
    let pem = cli_config.pem.or(file_config.pem);
    let private_key = cli_config.key.or(file_config.key);

    let tls_config = match (tls, pem, private_key) {
        (false, _, _) => None,
        (true, Some(pem), Some(private_key)) => {
            Some(TlsConfig { pem, private_key })
        }
        (true, None, _) => {
            return Err(anyhow!("TLS enabled but missing required pem"));
        }
        (true, _, None) => {
            return Err(anyhow!(
                "TLS enabled but missing required private key"
            ));
        }
    };

    let proxies = file_config
        .proxies
        .into_iter()
        .chain(cli_config.proxies)
        .map(|p| Proxy::new(&p, &host))
        .collect::<Result<Vec<Proxy>, _>>()?;

    Ok(Settings {
        host,
        local_port,
        proxies,
        tls: tls_config,
    })
}

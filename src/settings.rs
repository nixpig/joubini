use crate::cli::Cli;
use anyhow::Error;
use anyhow::Result;
use anyhow::anyhow;
use clap::Parser;
use std::ffi::OsString;
use std::{fmt::Display, fs, path::PathBuf};

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct Settings {
    pub host: String,
    pub local_port: u16,
    pub local_addr: String,
    pub proxies: Vec<ProxyConfig>,
    pub config: Option<PathBuf>,
    pub tls: bool,
    pub pem: Option<PathBuf>,
    pub key: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        let host = default_host();
        let local_port = default_port();
        let local_addr = format!("{}:{}", host, local_port);

        Settings {
            host,
            local_port,
            local_addr,
            proxies: vec![],
            config: None,
            tls: false,
            pem: None,
            key: None,
        }
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}\n",
            self.proxies
                .iter()
                .map(|x| format!(
                    "\x1b[95mᴥ\x1b[0m {}:{}{} \x1b[94m➡\x1b[0m :{}{}",
                    self.host,
                    self.local_port,
                    x.local_path,
                    x.remote_port,
                    x.remote_path
                ))
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct ProxyConfig {
    pub local_path: String,
    pub remote_port: u16,
    pub remote_path: String,
    pub remote_addr: String,
}

impl ProxyConfig {
    pub fn new(s: &str, host: &str) -> Result<Self, Error> {
        let Some((local_path, remote)) = s.split_once(':') else {
            return Err(anyhow!("Unable to parse proxy definition."));
        };

        let (remote_port, remote_path) = match remote.split_once('/') {
            Some((port, path)) => (port, path),
            None => (remote, ""),
        };

        let remote_port = remote_port.parse::<u16>()?;

        Ok(ProxyConfig {
            local_path: format!("/{}", local_path),
            remote_port,
            remote_path: format!("/{}", remote_path),
            remote_addr: format!("{}:{}", host, remote_port),
        })
    }
}

fn default_host() -> String {
    String::from("127.0.0.1")
}

fn default_port() -> u16 {
    80
}

#[derive(Debug, serde::Deserialize)]
struct ConfigFileProxies {
    #[serde(default = "default_host")]
    host: String,

    #[serde(rename = "port", default = "default_port")]
    local_port: u16,

    proxies: Vec<String>,

    tls: Option<bool>,
    pem: Option<PathBuf>,
    key: Option<PathBuf>,
}

impl TryFrom<PathBuf> for Settings {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let config_str = fs::read_to_string(&path)?;
        let config_yaml: ConfigFileProxies = serde_yaml::from_str(&config_str)?;

        let proxies = config_yaml
            .proxies
            .iter()
            .map(|p| ProxyConfig::new(p, &config_yaml.host))
            .collect::<Result<Vec<ProxyConfig>, Error>>()?;

        let tls = config_yaml.tls.unwrap_or(false);

        let host = config_yaml.host;
        let local_port = config_yaml.local_port;
        let local_addr = format!("{}:{}", host, local_port);

        Ok(Settings {
            host,
            local_port,
            local_addr,
            proxies,
            config: Some(path),
            tls,
            pem: config_yaml.pem,
            key: config_yaml.key,
        })
    }
}

pub fn get_settings(cli_args: Vec<OsString>) -> Result<Settings, Error> {
    let cli = Cli::parse_from(cli_args);

    let file = cli
        .config
        .as_ref()
        .map(|p| Settings::try_from(PathBuf::from(p)))
        .transpose()?
        .unwrap_or_default();

    let host = cli.host.as_deref().unwrap_or(&file.host);

    let mut proxies = file
        .proxies
        .into_iter()
        .map(|p| ProxyConfig {
            remote_addr: format!("{}:{}", host, p.remote_port),
            ..p
        })
        .collect::<Vec<ProxyConfig>>();

    let mut cli_proxies = cli
        .proxies
        .iter()
        .map(|p| ProxyConfig::new(p, host))
        .collect::<Result<Vec<ProxyConfig>, _>>()?;

    proxies.append(&mut cli_proxies);

    let host = cli.host.unwrap_or(file.host);
    let local_port = cli.local_port.unwrap_or(file.local_port);
    let local_addr = format!("{}:{}", host, local_port);

    Ok(Settings {
        host,
        local_port,
        local_addr,
        proxies,
        config: cli.config.or(file.config),
        tls: cli.tls || file.tls,
        pem: cli.pem.or(file.pem),
        key: cli.key.or(file.key),
    })
}

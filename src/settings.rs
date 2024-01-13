use crate::error::Error;
use crate::{cli::Cli, error::ParseError};
use clap::Parser;
use std::ffi::OsString;
use std::{fmt::Display, fs, path::PathBuf, str::FromStr};
use tracing::debug;

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct Settings {
    pub host: String,
    pub local_port: u16,
    pub proxies: Vec<ProxyConfig>,
    pub config: Option<PathBuf>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            host: String::from("localhost"),
            local_port: 80,
            proxies: vec![],
            config: None,
        }
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\n{}",
            self.proxies
                .iter()
                .map(|x| format!(
                    "Proxy: {}:{}{} => :{}{}",
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

impl Settings {
    pub fn new() -> Settings {
        Settings::default()
    }

    pub fn merge(&mut self, other: &mut Settings) -> Settings {
        let mut proxies: Vec<ProxyConfig> = vec![];

        proxies.append(&mut self.proxies);
        proxies.append(&mut other.proxies);

        Settings {
            host: other.host.clone(),
            local_port: other.local_port,
            proxies,
            config: other.config.clone(),
        }
    }
}

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct ProxyConfig {
    pub local_path: String,
    pub remote_port: u16,
    pub remote_path: String,
}

impl FromStr for ProxyConfig {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((local_path, remote)) = s.split_once(':') {
            let (remote_port, remote_path) = if let Some((
                remote_port,
                remote_path,
            )) = remote.split_once('/')
            {
                (remote_port, remote_path)
            } else {
                (remote, "")
            };

            Ok(ProxyConfig {
                local_path: ["/", local_path].join(""),
                remote_port: remote_port.parse::<u16>()?,
                remote_path: ["/", remote_path].join(""),
            })
        } else {
            Err(Error::ParseError(ParseError::ProxyDefinition))
        }
    }
}

impl TryFrom<Cli> for Settings {
    type Error = Error;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let proxies = value
            .proxies
            .iter()
            .map(|p| ProxyConfig::from_str(p))
            .collect::<Result<Vec<ProxyConfig>, Error>>()?;

        Ok(Settings {
            host: value.host,
            local_port: value.local_port,
            proxies,
            config: value.config,
        })
    }
}

fn default_host() -> String {
    String::from("localhost")
}

fn default_port() -> u16 {
    80
}

#[derive(Debug, serde::Deserialize)]
struct ConfigFileProxies {
    #[serde(default = "default_host")]
    host: String,

    #[serde(default = "default_port")]
    local_port: u16,

    proxies: Vec<String>,
}

impl TryFrom<PathBuf> for Settings {
    type Error = Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let config_str = fs::read_to_string(&path)?;
        let config_yaml: ConfigFileProxies = serde_yaml::from_str(&config_str)?;

        let proxies = config_yaml
            .proxies
            .iter()
            .map(|p| ProxyConfig::from_str(p))
            .collect::<Result<Vec<ProxyConfig>, Error>>()?;

        Ok(Settings {
            host: config_yaml.host,
            local_port: config_yaml.local_port,
            proxies,
            config: Some(path),
        })
    }
}

pub fn get_settings(cli_args: Vec<OsString>) -> Result<Settings, Error> {
    let mut cli_settings: Settings = Cli::parse_from(cli_args).try_into()?;
    debug!("cli_settings: {}", cli_settings);

    if let Some(config_file) = &cli_settings.config {
        let mut file_settings = Settings::try_from(PathBuf::from(config_file))?;
        debug!("file_settings: {}", file_settings);

        Ok(Settings::new()
            .merge(&mut cli_settings)
            .merge(&mut file_settings))
    } else {
        Ok(cli_settings)
    }
}

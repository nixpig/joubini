use crate::cli::Cli;
use clap::Parser;
use std::{fs, path::PathBuf, str::FromStr};

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct Settings {
    pub host: String,
    pub local_port: u16,
    pub proxies: Vec<ProxyConfig>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            host: String::from("localhost"),
            local_port: 80,
            proxies: vec![],
        }
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
        }
    }
}

#[derive(Ord, Eq, PartialOrd, Debug, PartialEq)]
pub struct ProxyConfig {
    pub local_path: String,
    pub remote_port: u16,
    pub remote_path: String,
}

#[derive(Debug, PartialEq)]
pub struct ProxyConfigParseError;

impl FromStr for ProxyConfig {
    type Err = ProxyConfigParseError;

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
                remote_port: remote_port.parse::<u16>().unwrap(),
                remote_path: ["/", remote_path].join(""),
            })
        } else {
            Err(ProxyConfigParseError)
        }
    }
}

#[derive(Debug)]
pub struct SettingsParseError;

impl TryFrom<Cli> for Settings {
    type Error = SettingsParseError;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let proxies = value
            .proxies
            .iter()
            .map(|p| ProxyConfig::from_str(p).unwrap())
            .collect();

        Ok(Settings {
            host: value.host,
            local_port: value.local_port,
            proxies,
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
    type Error = SettingsParseError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let config_str = fs::read_to_string(path).unwrap();
        let config_yaml: ConfigFileProxies =
            serde_yaml::from_str(&config_str).unwrap();

        let proxies = config_yaml
            .proxies
            .iter()
            .map(|p| ProxyConfig::from_str(p).unwrap())
            .collect();

        Ok(Settings {
            host: config_yaml.host,
            local_port: config_yaml.local_port,
            proxies,
        })
    }
}

pub fn get_settings() -> Settings {
    let mut cli_settings = Cli::parse().try_into().unwrap();
    let mut file_settings =
        Settings::try_from(PathBuf::from_str("config.yml").unwrap()).unwrap();

    Settings::new()
        .merge(&mut file_settings)
        .merge(&mut cli_settings)
}

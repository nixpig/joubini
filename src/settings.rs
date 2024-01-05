use crate::{cli::Cli, proxy::Proxy};
use clap::Parser;
use std::str::FromStr;

#[derive(Debug)]
pub struct Settings {
    pub local_port: u16,
    pub proxies: Vec<Proxy>,
}

#[derive(Debug)]
pub struct SettingsParseError;

impl TryFrom<Cli> for Settings {
    type Error = SettingsParseError;

    fn try_from(value: Cli) -> Result<Self, Self::Error> {
        let proxies = value
            .proxies
            .iter()
            .map(|p| Proxy::from_str(p).expect("Unable to parse proxy config"))
            .collect();

        Ok(Settings {
            local_port: value.port,
            proxies,
        })
    }
}

pub fn get_settings() -> Settings {
    Cli::parse().try_into().expect("Unable to parse arguments")
}

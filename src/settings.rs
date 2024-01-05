use std::str::FromStr;

use crate::{cli::Cli, proxy::Proxy};

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

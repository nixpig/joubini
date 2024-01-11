use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ConfigFileReadError(std::io::Error),
    ConfigFileParseError(serde_yaml::Error),
    ParsePortError(std::num::ParseIntError),

    ProxyConfigParseError(String),
    SettingsParseError(String),
    ProxyUriMapError(hyper::http::uri::InvalidUri),
    RequestBuildError(String),
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParsePortError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::ConfigFileReadError(value)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::ConfigFileParseError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ConfigFileReadError(ref e) => {
                write!(f, "Could not read config file: {}", e)
            }
            Error::ConfigFileParseError(ref e) => {
                write!(f, "Could not parse config file: {}", e)
            }
            Error::ParsePortError(ref e) => {
                write!(f, "Could not parse port number: {}", e)
            }

            Error::ProxyConfigParseError(ref e) => {
                write!(f, "Could not parse proxy config: {}", e)
            }
            Error::SettingsParseError(ref e) => {
                write!(f, "Could not parse settings: {}", e)
            }
            Error::ProxyUriMapError(ref e) => {
                write!(f, "Could not map proxy URI: {}", e)
            }
            Error::RequestBuildError(ref e) => {
                write!(f, "Could not build request: {}", e)
            }
        }
    }
}

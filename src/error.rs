use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    ProxyConfigParseError(String),
    SettingsParseError(String),
    ProxyUriMapError(hyper::http::uri::InvalidUri),
    RequestBuildError(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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

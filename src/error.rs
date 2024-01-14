use std::fmt::Display;

#[derive(Debug)]
pub enum ProxyError {
    RequestFailed(hyper::Error),
}

impl Display for ProxyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyError::RequestFailed(ref e) => {
                write!(f, "Request failed: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum IoError {
    StdIo(std::io::Error),
}

impl Display for IoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IoError::StdIo(ref e) => {
                write!(f, "Standard IO error: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    ParseInt(std::num::ParseIntError),
    ProxyDefinition,
    FileConfig(serde_yaml::Error),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseInt(ref e) => {
                write!(f, "Parse int error: {}", e)
            }
            ParseError::ProxyDefinition => {
                write!(f, "Unable to parse proxy definition.")
            }
            ParseError::FileConfig(ref e) => {
                write!(f, "Unable to parse config from config file: {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    IoError(IoError),
    ParseError(ParseError),
    ProxyError(ProxyError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::IoError(ref e) => {
                write!(f, "IO error: {}", e)
            }

            Error::ParseError(ref e) => {
                write!(f, "Parse error: {}", e)
            }

            Error::ProxyError(ref e) => {
                write!(f, "Proxy error: {}", e)
            }
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(value: std::num::ParseIntError) -> Self {
        Error::ParseError(ParseError::ParseInt(value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(IoError::StdIo(value))
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(value: serde_yaml::Error) -> Self {
        Error::ParseError(ParseError::FileConfig(value))
    }
}

impl From<hyper::Error> for Error {
    fn from(value: hyper::Error) -> Self {
        Error::ProxyError(ProxyError::RequestFailed(value))
    }
}

impl std::error::Error for Error {}

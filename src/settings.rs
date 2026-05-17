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

#[derive(Clone, Ord, Eq, PartialOrd, Debug, PartialEq, Deserialize)]
pub struct Proxy {
    pub local_path: String,
    pub(crate) normalised_local_path: Vec<String>,
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

        let local_path = format!("/{}", local_path);
        let normalised_local_path = local_path
            .split('/')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect::<Vec<String>>();

        Ok(Proxy {
            local_path,
            normalised_local_path,
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

    let tls = cli_config.tls || file_config.tls.unwrap_or(false);

    let local_port = cli_config
        .local_port
        .or(file_config.local_port)
        .unwrap_or(if tls { 443 } else { 80 });

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

#[test]
fn test_parse_proxy_config_from_str() -> Result<(), Box<dyn std::error::Error>>
{
    let p1 = ":3000"; // :remote_port
    let o1 = Proxy::new(p1, "localhost");

    assert_eq!(
        o1.unwrap(),
        Proxy {
            local_path: String::from("/"),
            normalised_local_path: vec![],
            remote_path: String::from("/"),
            remote_addr: String::from("localhost:3000"),
        },
    );

    let p2 = ":3000/api"; // :remote_port/remote_path
    let o2 = Proxy::new(p2, "localhost");

    assert_eq!(
        o2.unwrap(),
        Proxy {
            local_path: String::from("/"),
            normalised_local_path: vec![],
            remote_path: String::from("/api"),
            remote_addr: String::from("localhost:3000"),
        }
    );

    let p3 = "api:3000"; // local_path:remote_port
    let o3 = Proxy::new(p3, "localhost");

    assert_eq!(
        o3.unwrap(),
        Proxy {
            local_path: String::from("/api"),
            normalised_local_path: vec![String::from("api")],
            remote_path: String::from("/"),
            remote_addr: String::from("localhost:3000"),
        }
    );

    let p4 = "api:3000/api"; // local_path:remote_port/remote_path
    let o4 = Proxy::new(p4, "localhost");

    assert_eq!(
        o4.unwrap(),
        Proxy {
            local_path: String::from("/api"),
            normalised_local_path: vec![String::from("api")],
            remote_path: String::from("/api"),
            remote_addr: String::from("localhost:3000"),
        }
    );

    let p5 = "local/v1:3000/api/v1"; // nested_local_path:remote_port/nested_remote_path
    let o5 = Proxy::new(p5, "localhost");

    assert_eq!(
        o5.unwrap(),
        Proxy {
            local_path: String::from("/local/v1"),
            normalised_local_path: vec![
                String::from("local"),
                String::from("v1")
            ],
            remote_path: String::from("/api/v1"),
            remote_addr: String::from("localhost:3000"),
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_with_optional_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let settings_file_path = PathBuf::from("tests/config.yml");

    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from(format!("--config={}", settings_file_path.display())),
    ]);

    assert_eq!(
        settings.unwrap(),
        Settings {
            host: String::from("localhost"),
            local_port: 7878,
            tls: Some(TlsConfig {
                pem: PathBuf::from("/tmp/localhost.crt"),
                private_key: PathBuf::from("/tmp/localhost.key"),
            }),
            proxies: vec![
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    normalised_local_path: vec![
                        String::from("local"),
                        String::from("v1")
                    ],
                    remote_path: String::from("/remote/v1"),
                    remote_addr: String::from("localhost:3000"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_without_optional_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let settings_file_path = PathBuf::from("tests/config-without-options.yml");

    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from(format!("--config={}", settings_file_path.display())),
    ]);

    assert_eq!(
        settings.unwrap(),
        Settings {
            host: String::from("127.0.0.1"),
            local_port: 80,
            tls: None,
            proxies: vec![
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    normalised_local_path: vec![
                        String::from("local"),
                        String::from("v1")
                    ],
                    remote_path: String::from("/remote/v1"),
                    remote_addr: String::from("127.0.0.1:3000"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_parse_empty_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let config_file = PathBuf::from("tests/empty-config.yml");

    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from(format!("--config={}", config_file.display())),
    ]);

    assert!(settings.is_err());

    Ok(())
}

#[test]
fn test_tls_missing_pem() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = vec![
        OsString::from(""),
        OsString::from("--tls"),
        OsString::from("--key=/tmp/localhost.key"),
    ];

    let settings = get_settings(cli_args);

    assert!(settings.is_err());

    Ok(())
}

#[test]
fn test_invalid_proxy_config() -> Result<(), Box<dyn std::error::Error>> {
    let p = "invalid proxy config";
    let o = Proxy::new(p, "localhost").unwrap_err().to_string();

    assert_eq!(o, String::from("Unable to parse proxy definition."));

    Ok(())
}

#[test]
fn test_fail_invalid_port() -> Result<(), Box<dyn std::error::Error>> {
    let p1 = ":foo";
    let p2 = ":bar/baz";
    let p3 = "qux:thud/fred";

    let c1 = Proxy::new(p1, "localhost").unwrap_err().to_string();
    let c2 = Proxy::new(p2, "localhost").unwrap_err().to_string();
    let c3 = Proxy::new(p3, "localhost").unwrap_err().to_string();

    assert_eq!(c1, String::from("invalid digit found in string"));
    assert_eq!(c2, String::from("invalid digit found in string"));
    assert_eq!(c3, String::from("invalid digit found in string"));

    Ok(())
}

#[test]
fn test_missing_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from("--config=tests/missing.yml"),
    ]);

    let err = settings.unwrap_err().to_string();

    assert_eq!(err, String::from("No such file or directory (os error 2)"));

    Ok(())
}

#[test]
fn test_get_settings_without_config_file()
-> Result<(), Box<dyn std::error::Error>> {
    let cli_args = vec![
        OsString::from("empty first value to discard"),
        OsString::from("--port=7878"),
        OsString::from("--host=127.0.0.1"),
        OsString::from("--proxy=:3000"),
        OsString::from("--tls"),
        OsString::from("--pem=foo/bar.pem"),
        OsString::from("--key=bar/baz.key"),
    ];

    let settings = get_settings(cli_args)
        .expect("Should be able to parse cli args to settings");

    assert_eq!(
        settings,
        Settings {
            host: String::from("127.0.0.1"),
            local_port: 7878,
            tls: Some(TlsConfig {
                pem: PathBuf::from("foo/bar.pem"),
                private_key: PathBuf::from("bar/baz.key"),
            }),
            proxies: vec![Proxy {
                local_path: String::from("/"),
                normalised_local_path: vec![],
                remote_path: String::from("/"),
                remote_addr: String::from("127.0.0.1:3000")
            }]
        }
    );

    Ok(())
}

#[test]
fn test_cli_overrides_config_file() -> Result<(), Box<dyn std::error::Error>> {
    let cli_args = vec![
        OsString::from("joubini"),
        OsString::from("--config=tests/config.yml"),
        OsString::from("--host=127.0.0.1"),
        OsString::from("--pem=override/localhost.pem"),
        OsString::from("--key=override/localhost.key"),
    ];

    let settings = get_settings(cli_args)?;

    for p in &settings.proxies {
        assert!(p.remote_addr.starts_with("127.0.0.1:"));
    }

    let tls = settings.tls.unwrap();

    assert_eq!(tls.pem, "override/localhost.pem".to_string());
    assert_eq!(tls.private_key, "override/localhost.key".to_string());

    Ok(())
}

#[test]
fn test_cli_proxies_merge_config_file_proxies()
-> Result<(), Box<dyn std::error::Error>> {
    let settings_file_path = PathBuf::from("tests/config.yml");

    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from(format!("--config={}", settings_file_path.display())),
        OsString::from("--proxy=api/v2:3000/v2"),
        OsString::from("--proxy=api/v3:3000/v3"),
    ]);

    assert_eq!(
        settings.unwrap(),
        Settings {
            host: String::from("localhost"),
            local_port: 7878,
            tls: Some(TlsConfig {
                pem: PathBuf::from("/tmp/localhost.crt"),
                private_key: PathBuf::from("/tmp/localhost.key"),
            }),
            proxies: vec![
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    normalised_local_path: vec![],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    normalised_local_path: vec![String::from("api")],
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    normalised_local_path: vec![
                        String::from("local"),
                        String::from("v1")
                    ],
                    remote_path: String::from("/remote/v1"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api/v2"),
                    normalised_local_path: vec![
                        String::from("api"),
                        String::from("v2")
                    ],
                    remote_path: String::from("/v2"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api/v3"),
                    normalised_local_path: vec![
                        String::from("api"),
                        String::from("v3")
                    ],
                    remote_path: String::from("/v3"),
                    remote_addr: String::from("localhost:3000"),
                },
            ]
        }
    );

    Ok(())
}

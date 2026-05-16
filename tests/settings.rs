use std::{error::Error, ffi::OsString, path::PathBuf, str::FromStr};

use joubini::settings::{Proxy, Settings, TlsConfig, get_settings};

#[test]
fn test_parse_proxy_config_from_str() -> Result<(), Box<dyn Error>> {
    let p1 = ":3000"; // :remote_port
    let o1 = Proxy::new(p1, "localhost");

    assert_eq!(
        o1.unwrap(),
        Proxy {
            local_path: String::from("/"),
            remote_port: 3000,
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
            remote_port: 3000,
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
            remote_port: 3000,
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
            remote_port: 3000,
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
            remote_port: 3000,
            remote_path: String::from("/api/v1"),
            remote_addr: String::from("localhost:3000"),
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_with_optional_fields()
-> Result<(), Box<dyn Error>> {
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
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
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
-> Result<(), Box<dyn Error>> {
    let settings_file_path =
        PathBuf::from_str("tests/config-without-options.yml").unwrap();

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
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("127.0.0.1:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                    remote_addr: String::from("127.0.0.1:3000"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_parse_empty_config_file() -> Result<(), Box<dyn Error>> {
    let config_file = PathBuf::from_str("tests/empty-config.yml")?;

    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from(format!("--config={}", config_file.display())),
    ]);

    assert!(settings.is_err());

    Ok(())
}

#[test]
fn test_tls_missing_pem() -> Result<(), Box<dyn Error>> {
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
fn test_invalid_proxy_config() -> Result<(), Box<dyn Error>> {
    let p = "invalid proxy config";
    let o = Proxy::new(p, "localhost").unwrap_err().to_string();

    assert_eq!(o, String::from("Unable to parse proxy definition."));

    Ok(())
}

#[test]
fn test_fail_invalid_port() -> Result<(), Box<dyn Error>> {
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
fn test_missing_config_file() -> Result<(), Box<dyn Error>> {
    let settings = get_settings(vec![
        OsString::from(""),
        OsString::from("--config=tests/missing.yml"),
    ]);

    let err = settings.unwrap_err().to_string();

    assert_eq!(err, String::from("No such file or directory (os error 2)"));

    Ok(())
}

#[test]
fn test_get_settings_without_config_file() -> Result<(), Box<dyn Error>> {
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
                pem: PathBuf::from_str("foo/bar.pem").unwrap(),
                private_key: PathBuf::from_str("bar/baz.key").unwrap(),
            }),
            proxies: vec![Proxy {
                local_path: String::from("/"),
                remote_port: 3000,
                remote_path: String::from("/"),
                remote_addr: String::from("127.0.0.1:3000")
            }]
        }
    );

    Ok(())
}

#[test]
fn test_cli_overrides_config_file() -> Result<(), Box<dyn Error>> {
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
fn test_cli_proxies_merge_config_file_proxies() -> Result<(), Box<dyn Error>> {
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
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api/v2"),
                    remote_port: 3000,
                    remote_path: String::from("/v2"),
                    remote_addr: String::from("localhost:3000"),
                },
                Proxy {
                    local_path: String::from("/api/v3"),
                    remote_port: 3000,
                    remote_path: String::from("/v3"),
                    remote_addr: String::from("localhost:3000"),
                },
            ]
        }
    );

    Ok(())
}

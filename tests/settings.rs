use std::{error::Error, ffi::OsString, path::PathBuf, str::FromStr};

use joubini::{
    cli::Cli,
    settings::{get_settings, ProxyConfig, Settings},
};

#[test]
fn test_parse_proxy_config_from_str() -> Result<(), Box<dyn Error>> {
    let p1 = ":3000"; // :remote_port
    let o1 = ProxyConfig::from_str(p1);

    assert_eq!(
        o1.unwrap(),
        ProxyConfig {
            local_path: String::from("/"),
            remote_port: 3000,
            remote_path: String::from("/"),
        },
    );

    let p2 = ":3000/api"; // :remote_port/remote_path
    let o2 = ProxyConfig::from_str(p2);

    assert_eq!(
        o2.unwrap(),
        ProxyConfig {
            local_path: String::from("/"),
            remote_port: 3000,
            remote_path: String::from("/api"),
        }
    );

    let p3 = "api:3000"; // local_path:remote_port
    let o3 = ProxyConfig::from_str(p3);

    assert_eq!(
        o3.unwrap(),
        ProxyConfig {
            local_path: String::from("/api"),
            remote_port: 3000,
            remote_path: String::from("/"),
        }
    );

    let p4 = "api:3000/api"; // local_path:remote_port/remote_path
    let o4 = ProxyConfig::from_str(p4);

    assert_eq!(
        o4.unwrap(),
        ProxyConfig {
            local_path: String::from("/api"),
            remote_port: 3000,
            remote_path: String::from("/api"),
        }
    );

    let p5 = "local/v1:3000/api/v1"; // nested_local_path:remote_port/nested_remote_path
    let o5 = ProxyConfig::from_str(p5);

    assert_eq!(
        o5.unwrap(),
        ProxyConfig {
            local_path: String::from("/local/v1"),
            remote_port: 3000,
            remote_path: String::from("/api/v1"),
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_with_optional_fields(
) -> Result<(), Box<dyn Error>> {
    let settings_file_path = PathBuf::from("tests/config.yml");

    let settings = Settings::try_from(settings_file_path).unwrap();

    assert_eq!(
        settings,
        Settings {
            config: Some(PathBuf::from("tests/config.yml")),
            host: String::from("localhost"),
            local_port: 7878,
            tls: true,
            pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
            key: Some(PathBuf::from("tests/ssl/localhost.key")),
            proxies: vec![
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_without_optional_fields(
) -> Result<(), Box<dyn Error>> {
    let settings_file_path =
        PathBuf::from_str("tests/config-without-options.yml").unwrap();

    let settings = Settings::try_from(settings_file_path).unwrap();

    assert_eq!(
        settings,
        Settings {
            config: Some(PathBuf::from("tests/config-without-options.yml")),
            host: String::from("127.0.0.1"),
            local_port: 80,
            tls: false,
            pem: None,
            key: None,
            proxies: vec![
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_parse_settings_from_cli() -> Result<(), Box<dyn Error>> {
    let config = Cli {
        config: Some(PathBuf::from("tests/config.yml")),
        host: String::from("127.0.0.1"),
        local_port: 7878,
        tls: true,
        pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
        key: Some(PathBuf::from("tests/ssl/localhost.key")),
        proxies: vec![
            String::from(":3000"),
            String::from(":3000/api"),
            String::from("api:3000"),
            String::from("api:3000/api"),
            String::from("local/v1:3000/remote/v1"),
        ],
    };

    let settings: Settings = config.try_into().unwrap();

    assert_eq!(
        settings,
        Settings {
            config: Some(PathBuf::from("tests/config.yml")),
            host: String::from("127.0.0.1"),
            local_port: 7878,
            tls: true,
            pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
            key: Some(PathBuf::from("tests/ssl/localhost.key")),
            proxies: vec![
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                }
            ]
        }
    );

    Ok(())
}

#[test]
fn test_merge_settings_structs() -> Result<(), Box<dyn Error>> {
    let mut settings_1 = Settings {
        config: None,
        host: String::from("localhost_1"),
        local_port: 7878,
        tls: false,
        pem: None,
        key: None,
        proxies: vec![ProxyConfig {
            local_path: String::from("/local_one"),
            remote_port: 3001,
            remote_path: String::from("/remote_one"),
        }],
    };

    let mut settings_2 = Settings {
        config: None,
        host: String::from("localhost_2"),
        local_port: 7879,
        tls: true,
        pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
        key: Some(PathBuf::from("tests/ssl/localhost.key")),
        proxies: vec![ProxyConfig {
            local_path: String::from("/local_two"),
            remote_port: 3002,
            remote_path: String::from("/remote_two"),
        }],
    };

    let merged_settings = settings_1.merge(&mut settings_2);

    assert_eq!(
        merged_settings,
        Settings {
            config: None,
            host: String::from("localhost_2"),
            local_port: 7879,
            tls: true,
            pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
            key: Some(PathBuf::from("tests/ssl/localhost.key")),
            proxies: vec![
                ProxyConfig {
                    local_path: String::from("/local_one"),
                    remote_port: 3001,
                    remote_path: String::from("/remote_one"),
                },
                ProxyConfig {
                    local_path: String::from("/local_two"),
                    remote_port: 3002,
                    remote_path: String::from("/remote_two"),
                },
            ]
        }
    );

    Ok(())
}

#[test]
fn test_create_new_settings() -> Result<(), Box<dyn Error>> {
    let new_settings = Settings::new();
    assert_eq!(
        new_settings,
        Settings {
            config: None,
            host: String::from("127.0.0.1"),
            local_port: 80,
            proxies: vec![],
            tls: false,
            pem: None,
            key: None,
        }
    );

    Ok(())
}

#[test]
fn test_create_default_settings() -> Result<(), Box<dyn Error>> {
    let default_settings = Settings::default();
    assert_eq!(
        default_settings,
        Settings {
            config: None,
            host: String::from("127.0.0.1"),
            local_port: 80,
            proxies: vec![],
            tls: false,
            pem: None,
            key: None,
        }
    );

    Ok(())
}

#[test]
fn test_fail_parsing_empty_config_file() -> Result<(), Box<dyn Error>> {
    let config_file = PathBuf::from_str("tests/empty-config.yml")?;

    let settings = Settings::try_from(config_file);

    assert!(settings.is_err());

    Ok(())
}

#[test]
fn test_print_settings() -> Result<(), Box<dyn Error>> {
    let mut settings = Settings::default();

    settings
        .proxies
        .push(ProxyConfig::from_str("foo:3000/bar").unwrap());

    settings
        .proxies
        .push(ProxyConfig::from_str("baz:3001/qux").unwrap());

    assert_eq!(settings.to_string(),
                    String::from("\n\x1b[95mᴥ\x1b[0m 127.0.0.1:80/foo \x1b[94m➡\x1b[0m :3000/bar\n\x1b[95mᴥ\x1b[0m 127.0.0.1:80/baz \x1b[94m➡\x1b[0m :3001/qux\n"));

    Ok(())
}

#[test]
fn test_invalid_proxy_config() -> Result<(), Box<dyn Error>> {
    let p = "invalid proxy config";
    let o = ProxyConfig::from_str(p).unwrap_err().to_string();

    assert_eq!(
        o,
        String::from("Parse error: Unable to parse proxy definition.")
    );

    Ok(())
}

#[test]
fn test_fail_invalid_port() -> Result<(), Box<dyn Error>> {
    let p1 = ":foo";
    let p2 = ":bar/baz";
    let p3 = "qux:thud/fred";

    let c1 = ProxyConfig::from_str(p1).unwrap_err().to_string();
    let c2 = ProxyConfig::from_str(p2).unwrap_err().to_string();
    let c3 = ProxyConfig::from_str(p3).unwrap_err().to_string();

    assert_eq!(
        c1,
        String::from(
            "Parse error: Parse int error: invalid digit found in string"
        )
    );
    assert_eq!(
        c2,
        String::from(
            "Parse error: Parse int error: invalid digit found in string"
        )
    );
    assert_eq!(
        c3,
        String::from(
            "Parse error: Parse int error: invalid digit found in string"
        )
    );

    Ok(())
}

#[test]
fn test_missing_config_file() -> Result<(), Box<dyn Error>> {
    let settings = Settings::try_from(PathBuf::from("tests/missing.yml"));

    let err = settings.unwrap_err().to_string();

    assert_eq!(err, String::from("IO error: Standard IO error: No such file or directory (os error 2)"));

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
            config: None,
            host: String::from("127.0.0.1"),
            local_port: 7878,
            tls: true,
            pem: Some(PathBuf::from_str("foo/bar.pem").unwrap()),
            key: Some(PathBuf::from_str("bar/baz.key").unwrap()),
            proxies: vec![ProxyConfig {
                local_path: String::from("/"),
                remote_port: 3000,
                remote_path: String::from("/")
            }]
        }
    );

    Ok(())
}

#[test]
fn test_settings_with_config_file() -> Result<(), Box<dyn Error>> {
    let cli_args = vec![
        OsString::from("empty first value to discard"),
        OsString::from("--config=tests/config.yml"),
    ];

    let settings = get_settings(cli_args)
        .expect("Should be able to parse cli args to settings");

    assert_eq!(
        settings,
        Settings {
            config: Some(PathBuf::from("tests/config.yml")),
            host: String::from("localhost"),
            local_port: 7878,
            tls: true,
            pem: Some(PathBuf::from("tests/ssl/localhost.crt")),
            key: Some(PathBuf::from("tests/ssl/localhost.key")),
            proxies: vec![
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/"),
                },
                ProxyConfig {
                    local_path: String::from("/api"),
                    remote_port: 3000,
                    remote_path: String::from("/api"),
                },
                ProxyConfig {
                    local_path: String::from("/local/v1"),
                    remote_port: 3000,
                    remote_path: String::from("/remote/v1"),
                }
            ]
        }
    );

    Ok(())
}

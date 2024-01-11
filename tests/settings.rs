use std::{error::Error, path::PathBuf, str::FromStr};

use joubini::{
    cli::Cli,
    settings::{ProxyConfig, Settings},
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

    let p6 = "invalid proxy config";
    let o6 = ProxyConfig::from_str(p6);

    assert!(o6.is_err());

    Ok(())
}

#[test]
fn test_parse_settings_from_config_file_with_optional_fields(
) -> Result<(), Box<dyn Error>> {
    let settings_file_path = PathBuf::from_str("tests/config.yml").unwrap();

    let settings = Settings::try_from(settings_file_path).unwrap();

    assert_eq!(
        settings,
        Settings {
            host: String::from("localhost"),
            local_port: 7878,
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
            host: String::from("localhost"),
            local_port: 80,
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
        host: String::from("127.0.0.1"),
        local_port: 7878,
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
            host: String::from("127.0.0.1"),
            local_port: 7878,
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
        host: String::from("localhost_1"),
        local_port: 7878,
        proxies: vec![ProxyConfig {
            local_path: String::from("/local_one"),
            remote_port: 3001,
            remote_path: String::from("/remote_one"),
        }],
    };

    let mut settings_2 = Settings {
        host: String::from("localhost_2"),
        local_port: 7879,
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
            host: String::from("localhost_2"),
            local_port: 7879,
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

// #[test]
// fn test_get_settings() -> Result<(), Box<dyn Error>> {
//     todo!()
// }

#[test]
fn test_create_new_settings() -> Result<(), Box<dyn Error>> {
    let new_settings = Settings::new();
    assert_eq!(
        new_settings,
        Settings {
            host: String::from("localhost"),
            local_port: 80,
            proxies: vec![]
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
            host: String::from("localhost"),
            local_port: 80,
            proxies: vec![]
        }
    );

    Ok(())
}

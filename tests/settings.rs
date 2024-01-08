use std::{error::Error, str::FromStr};

use joubini::settings::ProxyConfig;

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

// #[test]
// fn test_parse_settings_from_config_file() -> Result<(), Box<dyn Error>> {
//     todo!()
// }
//
// #[test]
// fn test_parse_settings_from_cli_arguments() -> Result<(), Box<dyn Error>> {
//     todo!()
// }
//
// #[test]
// fn test_merge_settings_structs() -> Result<(), Box<dyn Error>> {
//     todo!()
// }

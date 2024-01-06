use std::{error::Error, str::FromStr};

use joubini::proxy::Proxy;

#[test]
fn test_proxy_from_str() -> Result<(), Box<dyn Error>> {
    let p1 = Proxy::from_str(":3000").expect("Unable to parse proxy config");
    let p2 = Proxy::from_str("foo:3001").expect("Unable to parse proxy string");
    let p3 =
        Proxy::from_str("bar:3002/bar").expect("Unable to parse proxy config");
    let p4 = Proxy::from_str("foo:3003/bar/baz/qux")
        .expect("Unable to parse proxy config from string");
    let p5 = Proxy::from_str("foo/bar/baz:3004/qux")
        .expect("Unable to parse proxy settings from provided string");

    assert_eq!(
        p1,
        Proxy {
            local_path: String::from("/"),
            remote_path: String::from("/"),
            remote_port: 3000,
        }
    );

    assert_eq!(
        p2,
        Proxy {
            local_path: String::from("/foo"),
            remote_path: String::from("/"),
            remote_port: 3001,
        }
    );

    assert_eq!(
        p3,
        Proxy {
            local_path: String::from("/bar"),
            remote_path: String::from("/bar"),
            remote_port: 3002,
        }
    );

    assert_eq!(
        p4,
        Proxy {
            local_path: String::from("/foo"),
            remote_path: String::from("/bar/baz/qux"),
            remote_port: 3003,
        }
    );

    assert_eq!(
        p5,
        Proxy {
            local_path: String::from("/foo/bar/baz"),
            remote_path: String::from("/qux"),
            remote_port: 3004,
        }
    );

    Ok(())
}

#[test]
fn test_error_on_invalid_config_string() -> Result<(), Box<dyn Error>> {
    let proxy = Proxy::from_str("invalid_config");

    assert!(proxy.is_err());

    Ok(())
}

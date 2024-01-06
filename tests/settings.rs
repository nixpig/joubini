use joubini::{cli::Cli, proxy::Proxy, settings::Settings};
use std::{error::Error, str::FromStr};

#[test]
fn test_basic_settings() -> Result<(), Box<dyn Error>> {
    let settings: Settings = Cli {
        hostname: String::from("localhost"),
        port: 80,
        proxies: vec![
            String::from(":3000"),
            String::from("foo:3001"),
            String::from("bar:3002/bar"),
            String::from("foo:3003/bar/baz/qux"),
            String::from("foo/bar/baz:3004/qux"),
        ],
    }
    .try_into()
    .expect("Unable to parse Cli config to Settings");

    assert_eq!(
        settings,
        Settings {
            hostname: String::from("localhost"),
            local_port: 80,
            proxies: vec![
                Proxy::from_str(":3000").expect("Unable to parse proxy config"),
                Proxy::from_str("foo:3001")
                    .expect("Unable to parse proxy string"),
                Proxy::from_str("bar:3002/bar")
                    .expect("Unable to parse proxy config"),
                Proxy::from_str("foo:3003/bar/baz/qux")
                    .expect("Unable to parse proxy config from string"),
                Proxy::from_str("foo/bar/baz:3004/qux").expect(
                    "Unable to parse proxy settings from provided string"
                ),
            ]
        }
    );

    Ok(())
}

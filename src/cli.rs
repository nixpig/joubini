use std::path::PathBuf;

#[derive(clap::Parser, Debug)]
#[clap(
    author = "@nixpig",
    version = env!("CARGO_PKG_VERSION"),
    about = "A super-simple and minimally configurable HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets."
)]
pub struct Cli {
    #[clap(short = 'H', long = "host", name = "host", help = "Hostname or IP")]
    pub host: Option<String>,

    #[clap(
        short = 'P',
        long = "port",
        name = "local_port",
        help = "Local port for reverse proxy server to listen on"
    )]
    pub local_port: Option<u16>,

    #[clap(
        short = 'p',
        long = "proxy",
        name = "proxy_config",
        help = "Configuration for proxy in format '<:local_port?></local_path?><:remote_port!></remote_path?>'"
    )]
    pub proxies: Vec<String>,

    #[clap(
        short = 'C',
        long = "config",
        name = "config_file",
        help = "Path to configuration file"
    )]
    pub config: Option<PathBuf>,

    #[clap(
        short = 'T',
        long = "tls",
        default_value = "false",
        help = "Serve over TLS"
    )]
    pub tls: bool,

    #[clap(
        long = "pem",
        help = "Path to SSL certificate as `.pem` or `.crt`. Required if `--tls` flag is enabled."
    )]
    pub pem: Option<PathBuf>,

    #[clap(
        long = "key",
        help = "Path to TLS/SSL certificate key as `.key`. Required if `--tls` flag is enabled."
    )]
    pub key: Option<PathBuf>,
}

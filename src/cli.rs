#[derive(clap::Parser, Debug)]
#[clap(
    author = "@nixpig",
    version = "0.0.1",
    about = "A super-simple and minimally configurable HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets."
)]
pub struct Cli {
    #[clap(
        short = 'H',
        long = "host",
        name = "host",
        help = "Hostname or IP",
        default_value = "localhost"
    )]
    pub host: String,

    #[clap(
        short = 'P',
        long = "port",
        name = "local_port",
        help = "Local port for reverse proxy server to listen on",
        default_value = "80"
    )]
    pub local_port: u16,

    #[clap(
        short = 'p',
        long = "proxy",
        name = "proxy_config",
        help = "Configuration for proxy in format '<:local_port?></local_path?><:remote_port!></remote_path?>'"
    )]
    pub proxies: Vec<String>,
}

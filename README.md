[![Workflow Status](https://github.com/nixpig/joubini/actions/workflows/general.yml/badge.svg?branch=main)](https://github.com/nixpig/joubini/actions/workflows/general.yml?query=branch%3Amain)
[![Coverage Status](https://coveralls.io/repos/github/nixpig/joubini/badge.svg?branch=main)](https://coveralls.io/github/nixpig/joubini?branch=main)

# 🐙 joubini

A super-simple and minimally configurable reverse HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets.

## ⚠️ WORK IN PROGRESS

**It's probably not a good idea to actually use this for anything at this point. Maybe soon 🤷**

## Features

- Hop-by-hop headers (as defined in [RFC2616](https://datatracker.ietf.org/doc/html/rfc2616#section-13.5.1)) are removed by default.

### Examples

## Installation

### Build from source

1. `git clone https://github.com/nixpig/joubini.git`
1. `cd joubini`
1. `cargo build --release`
1. `mv ./target/release/joubini ~/.local/bin/`

## Usage

```shell
$ joubini --help

A super-simple and minimally configurable HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets.

Usage: joubini [OPTIONS]

Options:
  -H, --host <host>           Hostname or IP [default: localhost]
  -P, --port <local_port>     Local port for reverse proxy server to listen on [default: 80]
  -p, --proxy <proxy_config>  Configuration for proxy in format '</local_path?><:remote_port!></remote_path?>'
  -c, --config <config_file>  Path to configuration file
  -h, --help                  Print help
  -V, --version               Print version

```

### Note

Ordering of proxy configurations matters.

❌ This (probably) **will not** work as intended:
`joubini --proxy=myapp/api:3001/api --proxy=myapp:3000/ui`

✅ This (probably) **will** work as intended:
`joubini --proxy=myapp:3000/ui --proxy=myapp/api:3001/api`

### Config file (optional)

Proxies defined in the config file follow the same pattern as via CLI, i.e. `</local_path?><:remote_port!></remote_path?>`

```yaml
# joubini.yml
host: 127.0.0.1
port: 80
proxies:
  - :3000 # http://localhost -> http://localhost:3000
  - api:3001/api # http://localhost/api -> http://localhost:3001/api
  - admin:3002/dashboard # http://localhost/admin -> http://localhost:3002/dashboard
  - db:5432 # http://localhost/db -> http://localhost:5432
```

## Motivation

I just wanted an interesting little project to work on in Rust which involves some basic networking stuff and that would actually be useful.

## Alternatives

- [Caddy](https://caddyserver.com/)
- [NGINX](https://www.nginx.com/)
- [Apache (httpd)](https://httpd.apache.org/)

## Contribute

Any suggestions, feel free to open an [issue](https://github.com/nixpig/joubini/issues).

## Development

In order to bind to port 80 (or any port below 1024), you'll need to grant access to the binary to do so.

Replace `$PATH_TO_PROJECT` in command below with the _absolute_ path to the project.

```shell
sudo setcap CAP_NET_BIND_SERVICE=+eip $PATH_TO_PROJECT/target/debug/joubini

```

## License

[MIT](https://github.com/nixpig/joubini?tab=MIT-1-ov-file#readme)

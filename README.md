[![Workflow Status](https://github.com/nixpig/joubini/actions/workflows/general.yml/badge.svg?branch=main)](https://github.com/nixpig/joubini/actions/workflows/general.yml?query=branch%3Amain)
[![Coverage Status](https://coveralls.io/repos/github/nixpig/joubini/badge.svg?branch=main)](https://coveralls.io/github/nixpig/joubini?branch=main)

# 🐙 joubini

A super-simple and minimally configurable reverse HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets.

## ⚠️ WORK IN PROGRESS

**It's probably not a good idea to actually use this for anything at this point. Maybe soon 🤷**

## Features

```shell
$ joubini --help

A super-simple and minimally configurable HTTP reverse proxy for local development with support for HTTP/1.1, HTTP/2, TLS/SSL and web sockets.

Usage: joubini [OPTIONS]

Options:
  -p, --proxy <proxy_config>  Configuration for proxy in format '<local_path>:<remote_port>/<remote_path>'
  -P, --port <port>           Local port to listen on. [default: 80]
  -h, --help                  Print help
  -V, --version               Print version

```

## Examples

## Installation

### Build from source

1. `git clone https://github.com/nixpig/joubini.git`
1. `cd joubini`
1. `cargo build --release`
1. `mv ./target/release/joubini ~/.local/bin/`

## Usage

This is how I'd like it to work...either by CLI args or config file:

### Note

Ordering of proxy configurations matters.

❌ This (probably) **will not** work as intended:
`joubini --proxy=myapp:3000/ui --proxy=myapp/api:3001/api`

✅ This (probably) **will** work as intended:
`joubini --proxy=myapp/api:3001/api --proxy=myapp:3000/ui`

### CLI arguments

```shell
joubini

	# defaults to 80 if not specified (see note below about granting access to ports below 1024)
	-P | --port=7878

	# http://localhost -> http://localhost:3000
	-p | --proxy=:3000

	# http://localhost/api -> http://localhost:3001/api
	-p | --proxy=api:3001/api

	# http://localhost/admin -> http://localhost:3002/dashboard
	-p | --proxy=admin:3002/dashboard

	# http://localhost/db -> http://localhost:5432
	-p | --proxy=db:5432

	# http://localhost/deep -> http://localhost:3003/deep/nested/path
	-p | --proxy=deep:3003/deep/nested/path

```

### Config file

```yaml
# joubini.yml

- proxies:
    # http://localhost -> http://localhost:3000
    - :3000

    # http://localhost/api -> http://localhost:3001/api
    - api:3001/api

    # http://localhost/admin -> http://localhost:3002/dashboard
    - admin:3002/dashboard

    # http://localhost/db -> http://localhost:5432
    - db:5432
```

## Motivation

Just wanted an interesting little project to work on in Rust which involves some basic networking stuff and that would actually be useful.

## Alternatives

- [Caddy](https://caddyserver.com/)
- [NGINX](https://www.nginx.com/)
- [Apache (httpd)](https://httpd.apache.org/)

## Contribute

Any suggestions, feel free to open an [issue](https://github.com/nixpig/joubini/issues).

### Development

In order to bind to port 80 (or any port below 1024), you'll need to grant access to the binary to do so.

Replace `$PATH_TO_PROJECT` in command below with the _absolute_ path to the project.

```shell
sudo setcap CAP_NET_BIND_SERVICE=+eip $PATH_TO_PROJECT/target/debug/joubini

```

## License

[MIT](https://github.com/nixpig/joubini?tab=MIT-1-ov-file#readme)

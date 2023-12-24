# 🐙 joubini

A super-simple and minimally configurable reverse HTTP reverse proxy for local development.

> ⚠️ WORK IN PROGRESS

## Examples

## Installation

## Usage

This is how I'd like it to work...either by CLI args or config file:

### CLI arguments

```shell
joubini

	# defaults to localhost unless another provided
	-n | --hostname=localhost

	# http://localhost -> http://localhost:3000
	-p | --proxy=:3000

	# http://localhost/api -> http://localhost:3001/api
	-p | --proxy=api:3001/api

	# http://localhost/admin -> http://localhost:3002/dashboard
	-p | --proxy=admin:3002/dashboard

	# http://localhost/db -> http://localhost:5432
	-p | --proxy=db:5432

	# defaults to http unless ssl provided below then auto-uses https
	-c | --ssl-cert="/path/to/public/cert.pem"
	-k | --ssl-key="/path/to/priv/key.pem"

	# upgrade to use web sockets
	-u | --wss-upgrade=true

```

### Config file

```yaml
# joubini.yml

- localhost:
  proxy:
    # http://localhost -> http://localhost:3000
    - :3000

    # http://localhost/api -> http://localhost:3001/api
    - api:3001/api

    # http://localhost/admin -> http://localhost:3002/dashboard
    - admin:3002/dashboard

    # http://localhost/db -> http://localhost:5432
    - db:5432

  # defaults to http unless ssl provided below then auto-uses https
  ssl:
    cert: /path/to/public/cert.pem
    key: /path/to/priv/key.pem

  # upgrade to use web sockets
  wss: true
```

## Motivation

Just wanted an interesting little project to work on in Rust which involves some basic networking stuff and that would actually be useful.

## Alternatives

- [Caddy](https://caddyserver.com/)
- [NGINX](https://www.nginx.com/)
- [Apache (httpd)](https://httpd.apache.org/)

## Contribute

If there's any changes you want made, feel free to open an [issue](https://github.com/nixpig/joubini/issues).

## License

[MIT](https://github.com/nixpig/joubini?tab=MIT-1-ov-file#readme)

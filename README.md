# 🐙 joubini

A super-simple and minimally configurable reverse HTTP reverse proxy for local development.

> ⚠️ WORK IN PROGRESS

## Examples

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
- servers:
  localhost:
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

## Installation

## Usage

## Motivation

## Alternatives

## Contribute

## License

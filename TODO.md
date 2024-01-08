To implement:

- [ ] Only allow to run locally (i.e. localhost/127.0.0.1)
- [ ] Get config from CLI args
- [ ] Get config from YAML config file
- [ ] Merge configs and pass into 'startup' function
- [ ] Proxy requests for all HTTP methods to 'target'
  - [ ] Strip hop-by-hop headers
  - [ ] Add `x-forwarded-for` header
- [ ] Implement TLS/SSL support
- [ ] Implement upgrade for web sockets

Maybe implement configuration syntax like:

{:local_port?}{/local_path?}{:remote_port!}{/remote_path?}

E.g. all permutations:

```text
:80:3000	  :local_port:remote_port
:80:3000/api	  :local_port:remote_port/remote_path
:80/api:3000	  :local_port/local_path:remote_port
:80/api:3000/api  :local_port/local_path:remote_port/remote_path
api:3000	  local_path:remote_port
api:3000/api	  local_path:remote_port/remote_path
:3000/api	  :remote_port/remote_path
```

---

Things to consider:

- [ ] Request body and content types, e.g. application/json, x-www-form-urlencoded, multipart/form-data
- [ ] Query parameters
- [ ] More 'complex' scenarios than just a JSON REST API request.
- [ ] Resolution of local hostname to IP address (e.g. localhost -> 127.0.0.1)

To implement:

- [ ] Only allow to run locally (i.e. localhost/127.0.0.1)
- [x] Get config from CLI args
- [x] Get config from YAML config file
- [x] Merge configs and pass into 'startup' function
- [ ] Proxy requests for all HTTP methods to 'target'
  - [ ] Strip hop-by-hop headers
  - [ ] Add `x-forwarded-for` header (if already exists, add comma-space then uri)
- [ ] Implement TLS/SSL support
- [ ] Implement upgrade for web sockets
- [ ] HTTP 1.1
- [ ] HTTP 2
- [ ] HTTP 3??

---

Things to consider:

- [ ] Request body and content types, e.g. application/json, x-www-form-urlencoded, multipart/form-data
- [ ] Authorization headers?
- [ ] Query parameters
- [ ] More 'complex' scenarios than just a JSON REST API request.
- [ ] Resolution of local hostname to IP address (e.g. localhost -> 127.0.0.1)

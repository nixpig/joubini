To implement:

- [ ] Get config from CLI args
- [ ] Get config from YAML config file
- [ ] Merge configs and pass into 'startup' function
- [ ] Proxy requests for all HTTP methods to 'target'
  - [ ] Strip hop-by-hop headers
  - [ ] Add `x-forwarded-for` header
- [ ] Implement TLS/SSL support
- [ ] Implement upgrade for web sockets

---

Things to consider:

- [ ] Request body and content types, e.g. application/json, x-www-form-urlencoded, multipart/form-data
- [ ] Query parameters
- [ ] More 'complex' scenarios than just a JSON REST API request.

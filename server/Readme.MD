# Running

This command lets you run the server with with `on_request` and `on_response` trace from tower

```
➜ RUST_LOG="INFO,tower_http::trace::on_request=DEBUG,tower_http::trace::on_response=DEBUG" cargo run
```

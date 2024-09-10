# Authd Client Library
Use this library in other applications that interact with the auth daemon. You will need to include the core library as well.

In your Rust project, add the following to your `Cargo.toml`:
```toml
[dependencies]
ebay_authd_core = { git = "https://github.com/Tesyl-sro/ebay_authd" }
ebay_authd_client = { git = "https://github.com/Tesyl-sro/ebay_authd" }
```

You should also specify a tag to lock to a specific version:
```json
{ git = "https://github.com/Tesyl-sro/ebay_authd", tag = "v1.0.5" }
```

Now you can use the client library in your project.
```rust
use ebay_authd_client::Client;
use ebay_authd_core::request::Request;
use std::os::unix::net::UnixStream;

fn main() {
    let stream = UnixStream::connect("/tmp/ebay_authd.sock").unwrap();
    let mut client = Client::new(stream).unwrap();

    client.message(Request::ForceRefresh).unwrap();
    // connection is closed automatically on drop
}
```
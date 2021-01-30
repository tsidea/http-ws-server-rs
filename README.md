# http-ws-server-rs
An example of how to create a server that is able to service both http requests and websocket connections on the same port.

## Dependencies
This example uses:
- [tokio](https://docs.rs/tokio/1.0.2/tokio/), for the runtime
- [hyper](https://docs.rs/hyper/0.14.2/hyper/), for http
- [tungstenite](https://docs.rs/tungstenite/0.12.0/tungstenite/), for websocket
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/0.13.0/tokio_tungstenite/) for tokio bindings
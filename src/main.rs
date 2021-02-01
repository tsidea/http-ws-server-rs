use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{header, upgrade, StatusCode, Body, Request, Response, Server, server::conn::AddrStream};
use hyper::service::{make_service_fn, service_fn};
use tokio_tungstenite::WebSocketStream;
use tungstenite::{handshake, Error};
use futures::stream::StreamExt;

async fn handle_request(mut request: Request<Body>, remote_addr: SocketAddr) -> Result<Response<Body>, Infallible> {
    match (request.uri().path(), request.headers().contains_key(header::UPGRADE)) {
        //if the request is ws_echo and the request headers contains an Upgrade key
        ("/ws_echo", true) => {
        
            //assume request is a handshake, so create the handshake response
            let response = 
            match handshake::server::create_response_with_body(&request, || Body::empty()) {
                Ok(response) => {
                    //in case the handshake response creation succeeds,
                    //spawn a task to handle the websocket connection
                    tokio::spawn(async move {
                        //using the hyper feature of upgrading a connection
                        match upgrade::on(&mut request).await {
                            //if successfully upgraded
                            Ok(upgraded) => {
                                //create a websocket stream from the upgraded object
                                let ws_stream = WebSocketStream::from_raw_socket(
                                    //pass the upgraded object
                                    //as the base layer stream of the Websocket
                                    upgraded,
                                    tokio_tungstenite::tungstenite::protocol::Role::Server,
                                    None,
                                ).await;

                                //we can split the stream into a sink and a stream
                                let (ws_write, ws_read) = ws_stream.split();

                                //forward the stream to the sink to achieve echo
                                match ws_read.forward(ws_write).await {
                                    Ok(_) => {},
                                    Err(Error::ConnectionClosed) => 
                                        println!("Connection closed normally"),
                                    Err(e) => 
                                        println!("error creating echo stream on \
                                                    connection from address {}. \
                                                    Error is {}", remote_addr, e),
                                };
                            },
                            Err(e) =>
                                println!("error when trying to upgrade connection \
                                        from address {} to websocket connection. \
                                        Error is: {}", remote_addr, e),
                        }
                    });
                    //return the response to the handshake request
                    response
                },
                Err(error) => {
                    //probably the handshake request is not up to spec for websocket
                    println!("Failed to create websocket response \
                                to request from address {}. \
                                Error is: {}", remote_addr, error);
                    let mut res = Response::new(Body::from(format!("Failed to create websocket: {}", error)));
                    *res.status_mut() = StatusCode::BAD_REQUEST;
                    return Ok(res);
                }
            };
        
            Ok::<_, Infallible>(response)
        },
        ("/ws_echo", false) => {
            //handle the case where the url is /ws_echo, but does not have an Upgrade field
            Ok(Response::new(Body::from(format!("Getting even warmer, \
                                                try connecting to this url \
                                                using a websocket client.\n"))))
        },
        (url@_, false) => {
            //handle any other url without an Upgrade header field
            Ok(Response::new(Body::from(format!("This {} url doesn't do \
                                                much, try accessing the \
                                                /ws_echo url instead.\n", url))))
        },
        (_, true) => {
            //handle any other url with an Upgrade header field
            Ok(Response::new(Body::from(format!("Getting warmer, but I'm \
                                                only letting you connect \
                                                via websocket over on \
                                                /ws_echo, try that url.\n"))))
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    //hyper server boilerplate code from https://hyper.rs/guides/server/hello-world/

    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("Listening on {} for http or websocket connections.", addr);

    // A `Service` is needed for every connection, so this
    // creates one from our `handle_request` function.
    let make_svc = make_service_fn(|conn: & AddrStream| {
        let remote_addr = conn.remote_addr();
        async move {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(move |request: Request<Body>|
                handle_request(request, remote_addr)
            ))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

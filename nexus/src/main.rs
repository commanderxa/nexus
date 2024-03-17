use std::{
    env, process,
    sync::{mpsc::channel, Arc},
};

use state::connection::ConnectionState;
use tokio::{
    net::{TcpListener, UdpSocket},
    sync::Mutex,
};

use env_logger::Env;
use scylla::Session;

use crate::{db::session_setup, handler::handle_udp, result::Result, routes::get_routes};

use handler::handle_stream;

mod db;
mod errors;
mod filters;
mod handler;
mod handlers;
mod jwt;
mod ops;
mod result;
mod routes;
mod state;
mod tls;

#[tokio::main]
async fn main() -> Result<()> {
    // Logger
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Set up ctrl^c handler
    let (tx, rx) = channel();
    ctrlc::set_handler(move || tx.send(()).expect("Could not send signal on channel."))
        .expect("Error setting Ctrl-C handler");

    // If received ctrl^c => exit the program
    tokio::spawn(async move {
        rx.recv().expect("Could not receive from channel.");
        log::info!("Exiting...");
        process::exit(0);
    });

    // DB
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let session = session_setup(&uri).await;
    let session_copy: Arc<Mutex<Session>> = Arc::clone(&Arc::new(Mutex::new(session)));
    let session_web = session_copy.clone();

    // Active connections state
    let state: Arc<Mutex<ConnectionState>> = Arc::new(Mutex::new(ConnectionState::new()));

    // HTTP server
    let routes = get_routes(session_web);
    tokio::spawn(async move {
        warp::serve(routes)
            .tls()
            .cert_path("./certs/cert.pem")
            .key_path("./certs/key.pem")
            .run(([127, 0, 0, 1], 8082))
            .await;
    });

    // TCP Server
    log::info!(
        "TCP server listener set up at `{}`",
        std::env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_owned())
    );
    let listener =
        TcpListener::bind(env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_owned()))
            .await
            .expect("Error binding port");

    // UDP Server
    log::info!(
        "UDP server set up at `{}`",
        std::env::var("C_ADDR").unwrap_or_else(|_| "127.0.0.1:8083".to_owned())
    );
    let sock =
        UdpSocket::bind(std::env::var("C_ADDR").unwrap_or_else(|_| "127.0.0.1:8083".to_owned()))
            .await
            .unwrap();

    let udp_state = Arc::clone(&state);
    tokio::spawn(async move {
        match handle_udp(sock, udp_state).await {
            Ok(_) => (),
            Err(_) => {
                log::error!("Error handling request");
            }
        }
    });

    // TCP Listener
    while let Ok((stream, socket)) = listener.accept().await {
        let session: Arc<Mutex<Session>> = session_copy.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            match handle_stream(stream, socket, session, state).await {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Error handling request");
                }
            }
        });
    }

    Ok(())

    // TCP TLS Listener
    // let config = get_tls_config()?;
    // let acceptor = TlsAcceptor::from(Arc::new(config));
    // loop {
    //     while let Ok((stream, _socket)) = listener.accept().await {
    //         // let acceptor = acceptor.clone();
    //         let session: Arc<Mutex<Session>> = session_copy.clone();
    //         let fut = async move {
    //             let mut tls_stream = acceptor.accept(stream).await?;
    //             match handle_stream(tls_stream, session, state).await {
    //                 Ok(_) => (),
    //                 Err(_) => panic!("Error handling request."),
    //             }
    //             Ok(()) as Result<()>
    //         };
    //         tokio::spawn(async move {
    //             if let Err(err) = fut.await {
    //                 log::error!("{:?}", err);
    //                 panic!("Error handling request.");
    //             }
    //         });
    //     }
    // }
}

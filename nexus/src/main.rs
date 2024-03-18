use std::{
    process,
    sync::{mpsc::channel, Arc},
};

use api::run_http;
use env_logger::Env;
use scylla::Session;
use tokio::sync::Mutex;

use crate::{db::session_setup, result::Result};
use state::connection::ConnectionState;
use stream::{tcp::run_tcp, udp::run_udp};

mod api;
mod db;
mod errors;
mod handler;
mod ops;
mod result;
mod state;
mod stream;
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

    // DB session
    let _session = session_setup().await;
    let session: Arc<Mutex<Session>> = Arc::clone(&Arc::new(Mutex::new(_session)));
    
    // Active connections state
    let state: Arc<Mutex<ConnectionState>> = Arc::new(Mutex::new(ConnectionState::new()));

    // HTTP server
    run_http(session.clone()).await;
    // UDP Server
    run_udp(Arc::clone(&state)).await;
    // TCP Server
    run_tcp(session, state).await;

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

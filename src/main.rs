use std::{env, sync::Arc};

use state::connection::ConnectionState;
use tokio::{net::TcpListener, sync::Mutex};

use env_logger::Env;
use scylla::Session;

use crate::{result::Result, routes::get_routes};

use db::database;
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

    // DB
    let uri = std::env::var("SCYLLA_URI").unwrap_or_else(|_| "127.0.0.1:9042".to_string());
    let session = database::create_session(&uri).await?;
    database::initialize(&session).await?;
    log::info!("DB set up");
    let session_copy: Arc<Mutex<Session>> = Arc::clone(&Arc::new(Mutex::new(session)));
    let session_web = session_copy.clone();

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

    log::info!("TCP server listener set up");
    let listener =
        TcpListener::bind(env::var("ADDR").unwrap_or_else(|_| "127.0.0.1:8081".to_owned()))
            .await
            .expect("Error binding port");

    // Active TCP state
    let state: Arc<Mutex<ConnectionState>> = Arc::new(Mutex::new(ConnectionState::new()));

    // TCP Listener
    while let Ok((stream, _socket)) = listener.accept().await {
        let session: Arc<Mutex<Session>> = session_copy.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            match handle_stream(stream, session, state).await {
                Ok(_) => (),
                Err(_) => {
                    log::error!("Error handling request");
                    panic!("Error handling request");
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

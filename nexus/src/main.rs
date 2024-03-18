use std::{
    process,
    sync::{mpsc::channel, Arc},
};

use api::run_http;
use dotenv::dotenv;
use env_logger::Env;
use scylla::Session;
use storage::minio_setup;
use tokio::sync::Mutex;

use db::session_setup;
use result::Result;
use state::connection::ConnectionState;
use stream::{tcp::run_tcp, udp::run_udp};

mod api;
mod db;
mod errors;
mod handler;
mod ops;
mod result;
mod state;
mod storage;
mod stream;
mod tls;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
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
    let session: Arc<Mutex<Session>> = Arc::new(Mutex::new(_session));

    // Storage client
    let _storage_client = minio_setup().await;

    // Active connections state
    let state: Arc<Mutex<ConnectionState>> = Arc::new(Mutex::new(ConnectionState::new()));

    // HTTP server
    run_http(session.clone()).await;
    // UDP Server
    run_udp(Arc::clone(&state)).await;
    // TCP Server
    run_tcp(session, state).await;

    Ok(())
}

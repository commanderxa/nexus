use std::{collections::HashMap, error::Error, sync::Arc};

use futures::{SinkExt, StreamExt};
use scylla::Session;
use tokio::{
    net::TcpStream,
    sync::{mpsc::UnboundedSender, Mutex},
};

use orbis::{
    errors::stream::StreamError,
    models::command::Command,
    requests::{EmptyRequestBody, Request},
};
use tokio_util::codec::{Framed, LinesCodec};
use uuid::Uuid;

use crate::{
    handlers::users::get_uuid_by_token,
    ops::message::send_message,
    state::{connection::ConnectionState, peer::Peer},
};

/// This function handles stream and peer.
///
/// It sends and receives the messages.
///
/// Also regulates the program flow based on the received message type.
pub async fn handle_stream(
    stream: TcpStream,
    session: Arc<Mutex<Session>>,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), StreamError> {
    // this will allow to process lines instead of bytes in a stream
    let mut lines: Framed<TcpStream, LinesCodec> = Framed::new(stream, LinesCodec::new());
    // reading initial request
    let buf = match lines.next().await {
        // received something
        Some(buf) => buf,
        // stream is excited
        None => return Err(StreamError::FailedToReadLine),
    }
    .unwrap();

    // parsing the initial request
    let req_empty: Request<EmptyRequestBody> = serde_json::from_str(&buf).unwrap();
    let token = req_empty.token;

    // getting uuid of the user
    let user_uuid = get_uuid_by_token(session.clone(), &token).await;
    if user_uuid.is_err() {
        // if error => send it to the client and cancel the stream
        lines.send("Invalid JWT").await.unwrap();
        return Ok(());
    }
    let user_uuid = user_uuid.unwrap();

    // adding user to the active state
    let mut peer = add_peer(state.clone(), lines, user_uuid, token.clone())
        .await
        .unwrap();

    // infinite loop to sustain stream between server and client
    loop {
        tokio::select! {
            // received message from peer
            Some(msg) = peer.rx.recv() => {
                // send the message to the receiver
                peer.lines.send(&msg).await.unwrap();
            },
            // received message from user
            result = peer.lines.next() => match result {
                Some(Ok(msg)) => {
                    // parsing the command
                    let req_empty: Request<EmptyRequestBody> = serde_json::from_str(&msg).unwrap();
                    let req_command = req_empty.command;

                    // matches the operation from command
                    match req_command {
                        Command::Message => send_message(msg, session.clone(), state.clone()).await.unwrap(),
                        Command::AudioStream => todo!(),
                        Command::VideoStream => todo!(),
                    }
                },
                // error receiving a message
                Some(Err(e)) => {
                    log::error!("Error occured for {0}\n\tMessage: {e}", &peer.user_uuid);
                },
                // stream was exhausted by cancelling
                None => break,
            }
        }
    }

    {
        // removes user from acrive state when the stream is canceled
        remove_peer(state.clone(), user_uuid, token).await;
    }

    Ok(())
}

/// Adds user to the active state
///
/// Requires:
/// - ConnectionState
/// - User UUID
/// - Token
async fn add_peer(
    state: Arc<Mutex<ConnectionState>>,
    lines: Framed<TcpStream, LinesCodec>,
    user_uuid: Uuid,
    token: String,
) -> Result<Peer, Box<dyn Error>> {
    let (mut peer, tx) = Peer::new(lines, user_uuid, token);

    // locking the state
    let mut state = state.lock().await;
    // checking whether there is already exist an active session for this user
    match state.peers.get_mut(&user_uuid) {
        // if exists => adding a new session
        Some(_v) => {
            state
                .peers
                .get_mut(&user_uuid)
                .unwrap()
                .insert(peer.token.clone(), tx);
        }
        // if doesn't exist => add a user entry, then add the session
        None => {
            let hm_empty: HashMap<String, UnboundedSender<String>> = HashMap::new();
            // inserting new user to the peers
            state.peers.insert(user_uuid.clone(), hm_empty);

            // creating a new session for this new peer
            state
                .peers
                .get_mut(&user_uuid)
                .unwrap()
                .insert(peer.token.clone(), tx);
        }
    }

    for p in state.peers.iter() {
        println!("- {}", p.0);
        for pt in p.1.iter() {
            let splitted = pt.0.split(".").collect::<Vec<&str>>();
            println!("\t* {}", splitted[2]);
        }
    }

    // OK answer to the client
    peer.lines
        .send("Connection is established!".to_owned())
        .await?;

    Ok(peer)
}

/// Removes user from the active state
///
/// Requires:
/// - ConnectionsState
/// - User UUID
/// - Token
async fn remove_peer(state: Arc<Mutex<ConnectionState>>, user_uuid: Uuid, token: String) {
    state
        .clone()
        .lock()
        .await
        .peers
        .get_mut(&user_uuid)
        .unwrap()
        .remove(&token);
}

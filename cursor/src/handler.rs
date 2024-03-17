use std::{error::Error, net::SocketAddr, sync::Arc};

use futures::{SinkExt, StreamExt};
use hashbrown::HashMap;
use scylla::Session;
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::{mpsc, Mutex},
};

use cursorlib::{
    errors::stream::StreamError,
    models::{calls::audio_call::AudioCall, command::Command},
    request::{EmptyRequestBody, Request},
    response::{Response, ResponseStatus, ResponseStatusCode},
};
use tokio_util::codec::{Framed, LinesCodec};
use uuid::Uuid;

use crate::{
    handlers::users::get_uuid_by_token,
    ops::{audio_call::connect_audio, message::send_message},
    state::{
        connection::{ConnectionState, SessionSocket},
        peer::Peer,
    },
};

/// This function handles stream and peer.
///
/// It sends and receives the messages.
///
/// Also regulates the program flow based on the received message type.
pub async fn handle_stream(
    stream: TcpStream,
    socket_addr: SocketAddr,
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

    // parsing the initial request to get jwt
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

    // creating new UUID for new peer
    let peer_uuid = Uuid::new_v4();

    // adding user to the active state
    let mut peer = add_peer(state.clone(), lines, user_uuid, peer_uuid, socket_addr)
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
                        Command::Message => send_message((msg, peer.peer_uuid), session.clone(), state.clone()).await.unwrap(),
                        Command::AudioCall => connect_audio(msg, session.clone(), state.clone(), peer_uuid).await.unwrap(),
                        Command::VideoCall => todo!(),
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
        remove_peer(state.clone(), user_uuid, peer_uuid).await;
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
    peer_uuid: Uuid,
    socket_addr: SocketAddr,
) -> Result<Peer, Box<dyn Error>> {
    let (mut peer, tx) = Peer::new(lines, user_uuid, peer_uuid);

    // locking the state
    let mut state = state.lock().await;

    // defining session socket which is then inserted into the connection state
    let session_socket = SessionSocket::new(socket_addr, tx);
    // checking whether there is already exist an active session for this user
    match state.peers.get_mut(&user_uuid) {
        // if exists => adding a new session
        Some(_v) => {
            state
                .peers
                .get_mut(&user_uuid)
                .unwrap()
                .insert(peer_uuid, session_socket);
        }
        // if doesn't exist => add a user entry, then add the session
        None => {
            let hm_empty: HashMap<Uuid, SessionSocket> = HashMap::new();
            // inserting new user to the peers
            state.peers.insert(user_uuid.clone(), hm_empty);

            // creating a new session for this new peer
            state
                .peers
                .get_mut(&user_uuid)
                .unwrap()
                .insert(peer_uuid, session_socket);
        }
    }

    // OK answer to the client
    let response = serde_json::to_string(&Response::new(
        ResponseStatus::Ok,
        ResponseStatusCode::ConnectionEstablished,
    ))
    .unwrap();
    peer.lines.send(response).await?;

    Ok(peer)
}

/// Removes user from the active state
///
/// Requires:
/// - ConnectionsState
/// - User UUID
/// - Token
async fn remove_peer(state: Arc<Mutex<ConnectionState>>, user_uuid: Uuid, session_id: Uuid) {
    state
        .clone()
        .lock()
        .await
        .peers
        .get_mut(&user_uuid)
        .unwrap()
        .remove(&session_id);
}

/// Handles UDP stream for calls
pub async fn handle_udp(
    sock: UdpSocket,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), Box<dyn Error>> {
    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<(AudioCall, SocketAddr)>(1_000);

    let mut buf = [0; 1024];
    tokio::select! {
        // received a call from a peer
        Some((call, addr)) = rx.recv() => {
            // transmit the call frame to the session receiver
            let len = s.send_to(&call.message, addr).await.unwrap();
            println!("{:?} bytes sent", len);
        }
        // received a call frame from a user
        result = r.recv_from(&mut buf) => match result {
            Ok((len, addr)) => {
                println!("{:?} bytes received from {:?}", len, addr);

                // deserializing call and extracting the receiver
                let call = AudioCall::from_bytes(buf.to_vec());
                let receiver = &call.sides.get_receiver().to_owned();
                let receiver_peer = &call.peers.get_receiver().to_owned().unwrap();
                let recv_addr = state.lock().await.peers
                                    .get(receiver).unwrap()
                                    .get(receiver_peer).unwrap()
                                    .socket_addr;

                // sending the call frame to the receiver session peer
                tx.send((call.clone(), recv_addr)).await.unwrap();
            }
            Err(e) => {
                log::error!("Got error while receiving UDP Stream! {e}");
            },
        }
    }

    Ok(())
}

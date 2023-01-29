use std::{error::Error, sync::Arc};

use chrono::Duration;
use orbis::{
    models::calls::audio_call::AudioCall,
    request::{call::CallRequest, IndexToken, Request},
};
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{errors::db::DbError, filters::auth::check_token, state::connection::ConnectionState};

pub async fn connect_audio(
    call: String,
    session: Arc<Mutex<scylla::Session>>,
    state: Arc<Mutex<ConnectionState>>,
    peer_uuid: Uuid,
) -> Result<(), Box<dyn Error>> {
    let call_request: Request<CallRequest<AudioCall>> = serde_json::from_str(&call).unwrap();

    // verifying whether the token is valid
    let token_verify = check_token(session.clone(), call_request.token).await;
    if token_verify.is_err() {
        return Ok(());
    }

    // extract the call
    let mut call = call_request.body.call;
    // check if the call is secret
    if !call.secret {
        // if it is not a secret and
        if call_request.body.index == IndexToken::Start {
            // if this is an initial call request
            // => add to the DB
            if add_call(session, &call).await.is_err() {
                log::error!("Error adding call to the DB!");
            }
        } else {
            // if this is a cancel call request
            // => update the call entry in the DB
            if update_call(session, &call).await.is_err() {
                log::error!("Error updating call to the DB!");
            }
        }
    }

    if call_request.body.index == IndexToken::Start {
        call.peers.set_sender(peer_uuid);
    } else if call_request.body.index == IndexToken::Accept {
        call.peers.set_receiver(peer_uuid);
    }
    let call_str = serde_json::to_string(&call).unwrap();

    // iterating over all peers, searching for a receiver
    for peer in state
        .lock()
        .await
        .peers
        .get_mut(&call.sides.get_receiver())
        .unwrap()
        .iter_mut()
    {
        println!("Call is sent to the receiver: {}", peer.0);
        // sending the call to the receiver
        let _ = peer.1.tcp_sender.send(call_str.clone());
    }

    Ok(())
}

/// Adds a call to the DB
pub async fn add_call(
    session: Arc<Mutex<Session>>,
    call: &AudioCall,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "INSERT INTO litera.calls (uuid, sender, receiver, call_type, duration, accepted, created_at) VALUES(?, ?, ?, ?, ?, ?, ?);",
        )
        .await
        .unwrap();

    match session
        .lock()
        .await
        .execute(
            &prepared,
            (
                call.uuid,
                call.sides.get_sender(),
                call.sides.get_receiver(),
                call.get_type().get_index() as i8,
                0,
                false,
                Timestamp(Duration::seconds(call.get_created_at().timestamp())),
            ),
        )
        .await
    {
        Ok(result) => Ok(result),
        Err(_e) => {
            log::debug!("{_e:?}");
            Err(DbError::FailedToAdd)
        }
    }
}

/// Updates the call (duration, accepted) in the DB
pub async fn update_call(
    session: Arc<Mutex<Session>>,
    call: &AudioCall,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare("UPDATE litera.calls SET duration = ?, accepted = ? WHERE uuid = ?;")
        .await
        .unwrap();

    match session
        .lock()
        .await
        .execute(&prepared, (call.duration(), call.accepted, call.uuid))
        .await
    {
        Ok(result) => Ok(result),
        Err(_e) => {
            log::debug!("{_e:?}");
            Err(DbError::FailedToAdd)
        }
    }
}

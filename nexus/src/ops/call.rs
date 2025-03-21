use std::{error::Error, sync::Arc};

use chrono::Duration;
use nexuslib::{
    models::call::media_call::MediaCall,
    request::{call::CallRequest, index_token::IndexToken, Request},
};
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::{errors::db::DbError, state::connection::ConnectionState};

pub async fn connect_call(
    call: String,
    session: Arc<Mutex<scylla::Session>>,
    state: Arc<Mutex<ConnectionState>>,
    peer_uuid: Uuid,
) -> Result<(), Box<dyn Error>> {
    let mut call_request: Request<CallRequest<MediaCall>> = serde_json::from_str(&call).unwrap();

    println!("{:#?}", call_request);

    // extract the call
    // let mut call = call_request.body.call;
    // check if the call is secret
    if !call_request.body.call.secret {
        // if it is not a secret and
        if call_request.body.index == IndexToken::Start {
            // if this is an initial call request
            // => add to the DB
            if add_call(session, &call_request.body.call).await.is_err() {
                log::error!("Error adding call to the DB!");
            }
        } else if call_request.body.index == IndexToken::Accept
            || call_request.body.index == IndexToken::End
        {
            // if this is a cancel call request
            // => update the call entry in the DB
            if update_call(session, &call_request.body.call).await.is_err() {
                log::error!("Error updating call in the DB!");
            }
        }
    }

    if call_request.body.index == IndexToken::Start {
        call_request.body.call.peers.set_sender(peer_uuid);
    } else if call_request.body.index == IndexToken::Accept {
        call_request.body.call.peers.set_receiver(peer_uuid);
    }
    let call_str = serde_json::to_string(&call_request.body).unwrap();

    match call_request.body.index {
        // if index message is START => notify all receiver sessions
        IndexToken::Start => {
            // notify all receiver sessions
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_receiver())
                .unwrap()
                .iter_mut()
            {
                // sending the call to the receiver sessions
                let _ = peer.1.tcp_sender.send(call_str.clone());
            }

            // notify all sender sessions except the one that had sent the message
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_sender())
                .unwrap()
                .iter_mut()
            {
                if peer.0 != &peer_uuid {
                    // sending the call to the sender session
                    let _ = peer.1.tcp_sender.send(call_str.clone());
                }
            }
        }
        // if index message is ACCEPT => notify the receiver session and other
        // as receiver as also the other sender sessions that the call is accepted
        IndexToken::Accept => {
            // notify the sender sessions
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_sender())
                .unwrap()
                .iter_mut()
            {
                if *peer.0 != call_request.body.call.peers.get_sender().unwrap() {
                    // sending the call to the other sender sessions
                    call_request.body.index = IndexToken::Accepted;
                    let call_str = serde_json::to_string(&call_request.body).unwrap();
                    let _ = peer.1.tcp_sender.send(call_str.clone());
                } else {
                    // sending the call to the sender session
                    let _ = peer.1.tcp_sender.send(call_str.clone());
                }
            }

            // notify the other receiver sessions
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_receiver())
                .unwrap()
                .iter_mut()
            {
                // sending the call to the other receiver sessions
                if *peer.0 != call_request.body.call.peers.get_receiver().unwrap() {
                    call_request.body.index = IndexToken::Accepted;
                    let call_str = serde_json::to_string(&call_request.body).unwrap();
                    let _ = peer.1.tcp_sender.send(call_str.clone());
                }
            }
        }
        // server never receives such index message
        // if the call is ended from any side => notify everyone
        IndexToken::End => {
            // notify the sender sessions
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_sender())
                .unwrap()
                .iter_mut()
            {
                // sending the call to the sender session
                let _ = peer.1.tcp_sender.send(call_str.clone());
            }

            // notify the other receiver sessions
            for peer in state
                .lock()
                .await
                .peers
                .get_mut(&call_request.body.call.sides.get_receiver())
                .unwrap()
                .iter_mut()
            {
                let _ = peer.1.tcp_sender.send(call_str.clone());
            }
        }
        // includes IndexToken::Accepted
        // server sends it itself hence it will never receive this token
        _ => (),
    }

    Ok(())
}

/// Adds a call to the DB
pub async fn add_call(
    session: Arc<Mutex<Session>>,
    call: &MediaCall,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "INSERT INTO nexus.calls (uuid, sender, receiver, duration, accepted, secret, created_at) VALUES(?, ?, ?, ?, ?, ?, ?, ?);",
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
                0i64,
                false,
                call.secret,
                Timestamp(Duration::try_seconds(call.get_created_at().timestamp()).unwrap()),
            ),
        )
        .await
    {
        Ok(result) => Ok(result),
        Err(_e) => {
            log::error!("{_e:?}");
            Err(DbError::FailedToAdd)
        }
    }
}

/// Updates the call (duration, accepted) in the DB
pub async fn update_call(
    session: Arc<Mutex<Session>>,
    call: &MediaCall,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare("UPDATE nexus.calls SET duration = ?, accepted = ? WHERE uuid = ? AND created_at = ? IF EXISTS;")
        .await
        .unwrap();

    match session
        .lock()
        .await
        .execute(
            &prepared,
            (
                call.duration(),
                call.accepted,
                call.uuid,
                Timestamp(Duration::try_seconds(call.get_created_at().timestamp()).unwrap()),
            ),
        )
        .await
    {
        Ok(result) => Ok(result),
        Err(_e) => {
            log::error!("{_e:?}");
            Err(DbError::FailedToUpdate)
        }
    }
}

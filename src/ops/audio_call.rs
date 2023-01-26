use std::{error::Error, sync::Arc};

use chrono::Duration;
use orbis::{
    models::calls::audio_call::AudioCall,
    requests::{audio_call::AudioCallRequest, Request},
};
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;

use crate::{errors::db::DbError, filters::auth::check_token, state::connection::ConnectionState};

pub async fn connect_audio(
    message: String,
    session: Arc<Mutex<scylla::Session>>,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), Box<dyn Error>> {
    let phantom_call: Request<AudioCallRequest> = serde_json::from_str(&message).unwrap();

    // verifying whether the token is valid
    let token_verify = check_token(session.clone(), phantom_call.token).await;
    if token_verify.is_err() {
        return Ok(());
    }

    let call = phantom_call.body.call;

    if !call.secret {
        if add_call(session, &call).await.is_err() {
            log::error!("Error adding call to the DB!");
        }
    }

    let call_str = &serde_json::to_string(&message).unwrap();

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
        // sending the message to the receiver
        let _ = peer.1.send(call_str.to_owned());
    }

    Ok(())
}

/// Adds message to the DB
pub async fn add_call(
    session: Arc<Mutex<Session>>,
    message: &AudioCall,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "INSERT INTO litera.calls (uuid, sender, receiver, call_type, created_at) VALUES(?, ?, ?, ?, ?);",
        )
        .await
        .unwrap();

    match session
        .lock()
        .await
        .execute(
            &prepared,
            (
                message.uuid,
                message.sides.get_sender(),
                message.sides.get_receiver(),
                message.get_type().get_index() as i8,
                Timestamp(Duration::seconds(message.get_created_at().timestamp())),
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

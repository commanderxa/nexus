use std::{error::Error, fmt::Debug, sync::Arc};

use chrono::Duration;
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;

use nexuslib::{
    models::message::{text::TextMessage, MessageContent},
    request::{message::MessageRequest, Request},
    Message,
};
use uuid::Uuid;

use crate::{errors::db::DbError, state::connection::ConnectionState};

/// Sends a message to other user
///
/// Requires:
/// - Session
/// - Message
pub async fn send_message(
    message: (String, Uuid),
    session: Arc<Mutex<Session>>,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), Box<dyn Error>> {
    let (message, peer_uuid) = message;
    let message: Request<MessageRequest<TextMessage>> = serde_json::from_str(&message).unwrap();

    // when message arrives on the server, mark it as `sent`
    let mut message = message.body.message;
    message.status.set_sent();

    // checks if the message is not ment to be sent directly (secretly)
    if !message.secret {
        // add the message to the DB
        if add_message(session, &message).await.is_err() {
            log::error!("Error adding message to the DB!");
        }
    }

    // parsing the message
    let msg = &serde_json::to_string(&message).unwrap();

    // iterating over all peers, searching for the receiver sessions
    for peer in state
        .lock()
        .await
        .peers
        .get_mut(&message.sides.get_receiver())
        .unwrap()
        .iter_mut()
    {
        // sending the message to the receiver
        let _ = peer.1.tcp_sender.send(msg.to_owned());
    }

    // iterating over all peers, searching for the sender sessions
    // except the one that has sent this message
    for peer in state
        .lock()
        .await
        .peers
        .get_mut(&message.sides.get_sender())
        .unwrap()
        .iter_mut()
    {
        if peer.0 != &peer_uuid {
            // sending the message to the sender
            let _ = peer.1.tcp_sender.send(msg.to_owned());
        }
    }

    Ok(())
}

/// Adds message to the DB
pub async fn add_message<T: MessageContent + Debug>(
    session: Arc<Mutex<Session>>,
    message: &Message<T>,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "
            INSERT INTO nexus.messages 
            (uuid, text, nonce, media, sender, receiver, sent, read, edited, created_at) 
            VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
        ",
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
                message.content.get_text().unwrap(),
                message.get_nonce(),
                "".to_owned(), // UUIDs array of media (in a form of string, empy for now)
                message.sides.get_sender(),
                message.sides.get_receiver(),
                message.status.get_sent(),
                message.status.get_read(),
                message.status.get_edited(),
                Timestamp(Duration::try_seconds(message.get_created_at().timestamp()).unwrap()),
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

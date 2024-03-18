use std::{error::Error, sync::Arc};

use chrono::Duration;
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;

use nexuslib::{
    models::message::{
        media::MediaAttachment, r#type::MessageType, text::TextMessage, EmptyMessageBody,
    },
    request::{message::MessageRequest, Request},
};
use uuid::Uuid;

use crate::{
    api::filters::auth::check_token, errors::db::DbError, state::connection::ConnectionState,
};

/// Streams a file into the server
///
/// Requires:
/// - Session
/// - Message
pub async fn stream_file(
    message: (String, Uuid),
    session: Arc<Mutex<Session>>,
    state: Arc<Mutex<ConnectionState>>,
) -> Result<(), Box<dyn Error>> {
    let (message, peer_uuid) = message;
    let phantom_message: Request<MessageRequest<EmptyMessageBody>> =
        serde_json::from_str(&message).unwrap();

    // verifying whether the token is valid
    let token_verify = check_token(session.clone(), &phantom_message.token).await;
    if token_verify.is_err() {
        return Ok(());
    }

    /* THIS SECTION TO BE MODIFIED LATER */
    let message = match phantom_message.body.message.get_message_type() {
        MessageType::Text => {
            let text_message: Request<MessageRequest<TextMessage>> =
                serde_json::from_str(&message).unwrap();
            text_message
        }
        MessageType::File => todo!(),
    };
    /* --- */

    // when message arrives on the server, mark it as `sent`
    let mut message = message.body.message;

    // add file uploading logic
    todo!();

    message.status.set_sent();

    // checks if the message is not ment to be sent directly (secretly)
    if !message.secret {
        for attachment in message.media.clone().unwrap().attachments {
            // add the message to the DB
            if add_file(
                session.clone(),
                &attachment,
                message.sides.get_sender().clone(),
                message.get_created_at().timestamp(),
            )
            .await
            .is_err()
            {
                log::error!("Error adding message to the DB!");
            }
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
pub async fn add_file(
    session: Arc<Mutex<Session>>,
    attachment: &MediaAttachment,
    sender: Uuid,
    created_at: i64,
) -> Result<QueryResult, DbError> {
    let prepared: PreparedStatement = session
        .lock()
        .await
        .prepare(
            "
            INSERT INTO nexus.messages 
            (uuid, name, path, type, sender, created_at) 
            VALUES(?, ?, ?, ?, ?, ?);
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
                attachment.uuid,
                "".to_owned(),
                "".to_owned(),
                sender,
                attachment.get_type().get_index() as i8,
                Timestamp(Duration::try_seconds(created_at).unwrap()),
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

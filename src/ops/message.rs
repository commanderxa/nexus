use std::{fmt::Debug, sync::Arc};

use chrono::Duration;
use scylla::{
    frame::value::Timestamp, prepared_statement::PreparedStatement, QueryResult, Session,
};
use tokio::sync::Mutex;

use orbis::{
    models::message::{
        message::{EmptyMessageBody, MessageContent},
        msg_type::MessageType,
        text::TextMessage,
    },
    requests::{message::MessageRequest, Request},
    Message,
};

use crate::{errors::db::DbError, filters::auth::check_token, state::connection::ConnectionState};

/// Sends a message to other user
///
/// Requires:
/// - Session
/// - Buffer
pub async fn send_message(
    buf: String,
    session: Arc<Mutex<Session>>,
    state: Arc<Mutex<ConnectionState>>,
) -> tokio::io::Result<()> {
    let phantom_message: Request<MessageRequest<EmptyMessageBody>> =
        serde_json::from_str(&buf).unwrap();

    // verifying whether the token is valid
    let token_verify = check_token(session.clone(), phantom_message.token).await;
    if token_verify.is_err() {
        return Ok(());
    }

    /* THIS SECTION TO BE MODIFIED LATER */
    let message = match phantom_message.body.message.get_msg_type() {
        MessageType::Text => {
            let text_message: Request<MessageRequest<TextMessage>> =
                serde_json::from_str(&buf).unwrap();
            text_message
        }
        MessageType::Audio => todo!(),
        MessageType::Video => todo!(),
        MessageType::File => todo!(),
    };
    /* --- */

    // when message arrives on the server, mark it as `sent`
    let mut message = message.body.message;
    message.status.set_sent();

    // checks if the message is ment to be sent directly (secretly)
    if !message.secret {
        // add the message to the DB
        if add_message(session, &message).await.is_err() {
            log::error!("Error adding message to the DB!");
        }
    }

    // parsing the message
    let msg = &serde_json::to_string(&message).unwrap();

    // iterating over all peers, searching for a receiver
    for peer in state
        .lock()
        .await
        .peers
        .get_mut(&message.sides.get_receiver())
        .unwrap()
        .iter_mut()
    {
        println!("Message is sent to the receiver: {}", peer.0);
        // sending the message to the receiver
        let _ = peer.1.send(msg.to_owned());
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
            "INSERT INTO litera.messages (uuid, text, nonce, filename, filepath, sender, receiver, sent, read, edited, msg_type, created_at) VALUES(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
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
                "".to_owned(),
                "".to_owned(),
                message.sides.get_sender(),
                message.sides.get_receiver(),
                message.status.get_sent(),
                message.status.get_read(),
                message.status.get_edited(),
                message.get_msg_type().get_index() as i8,
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

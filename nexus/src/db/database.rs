use scylla::{Session, SessionBuilder};

use crate::Result;

use super::db_queries::*;

pub async fn create_session(uri: &str) -> Result<Session> {
    SessionBuilder::new()
        .known_node(uri)
        .build()
        .await
        .map_err(From::from)
}

pub async fn initialize(session: &Session) -> Result<()> {
    create_keyspace(session).await?;
    create_user_table(session).await?;
    create_keys_table(session).await?;
    create_session_table(session).await?;
    create_message_table(session).await?;
    create_call_table(session).await?;
    Ok(())
}

async fn create_keyspace(session: &Session) -> Result<()> {
    session
        .query(CREATE_KEYSPACE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_user_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_USER_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_keys_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_CHAT_KEYS_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_session_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_SESSION_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_message_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_MESSAGE_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_call_table(session: &Session) -> Result<()> {
    session
        .query(CREATE_CALL_TABLE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

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
    create_tables(session).await?;
    Ok(())
}

async fn create_keyspace(session: &Session) -> Result<()> {
    session
        .query(CREATE_KEYSPACE_QUERY, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

async fn create_tables(session: &Session) -> Result<()> {
    let tables = [
        CREATE_USER_TABLE_QUERY,
        CREATE_SECRET_KEYS_TABLE_QUERY,
        CREATE_SESSION_TABLE_QUERY,
        CREATE_MESSAGE_TABLE_QUERY,
        CREATE_CALL_TABLE_QUERY,
        CREATE_MEDIA_TABLE_QUERY,
    ];

    for table in tables {
        create_entity(session, table).await?;
    }

    Ok(())
}

/// Function to create a single entity
async fn create_entity(session: &Session, query: &str) -> Result<()> {
    session
        .query(query, ())
        .await
        .map(|_| ())
        .map_err(From::from)
}

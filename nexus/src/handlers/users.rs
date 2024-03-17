use std::{convert::Infallible, sync::Arc};

use chrono::Duration;
use scylla::{
    batch::Batch, frame::value::Timestamp, prepared_statement::PreparedStatement, IntoTypedRows,
    Session,
};
use tokio::sync::Mutex;
use uuid::{self, Uuid};
use warp::{hyper::StatusCode, Reply};

use nexuslib::models::user::user::User;

use crate::{db::models_wrapper::UserDB, errors::db::DbError};

pub async fn list(session: Arc<Mutex<Session>>, _uid: ()) -> Result<impl warp::Reply, Infallible> {
    // Just return a JSON array of users
    let users = session
        .lock()
        .await
        .query("SELECT * from litera.users", &[])
        .await
        .unwrap()
        .rows
        .unwrap_or_default()
        .into_typed::<UserDB>()
        .map(|u| u.unwrap().get_user())
        .collect::<Vec<User>>();

    Ok(warp::reply::json(&users))
}

pub async fn get_by_uuid(
    id: String,
    _uid: (),
    session: Arc<Mutex<Session>>,
) -> Result<impl warp::Reply, Infallible> {
    // Just return a JSON object of user
    let user_uuid = Uuid::parse_str(&id).unwrap();
    let user = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.users WHERE uuid = ? ALLOW FILTERING;",
            (user_uuid,),
        )
        .await
        .unwrap()
        .first_row()
        .unwrap()
        .into_typed::<UserDB>()
        .unwrap()
        .get_user();

    Ok(warp::reply::json(&user))
}

pub async fn get_by_username(
    username: String,
    _uid: (),
    session: Arc<Mutex<Session>>,
) -> Result<impl warp::Reply, Infallible> {
    // Just return a JSON object of user
    let user = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.users WHERE username = ? ALLOW FILTERING;",
            (username,),
        )
        .await
        .unwrap()
        .first_row()
        .unwrap()
        .into_typed::<UserDB>()
        .unwrap()
        .get_user();

    Ok(warp::reply::json(&user))
}

pub async fn create(
    user: (User, [u8; 32]),
    session: Arc<Mutex<Session>>,
) -> Result<StatusCode, Infallible> {
    let secret = user.1;
    let user = user.0;

    log::debug!("create_user: {:?}", user);

    if check_user_by_uuid(session.clone(), &user.uuid)
        .await
        .is_err()
    {
        return Ok(StatusCode::BAD_REQUEST);
    }

    if check_user_by_username(session.clone(), &user.username)
        .await
        .is_err()
    {
        return Ok(StatusCode::BAD_REQUEST);
    }

    // create batch
    let mut batch: Batch = Default::default();

    // prepare statements
    let prepared_user: PreparedStatement = session.lock().await
        .prepare(
            "INSERT INTO litera.users (uuid, username, password, role, public_key, created_at) VALUES(?, ?, ?, ?, ?, ?);",
        )
        .await
        .unwrap();

    let prepared_secret: PreparedStatement = session
        .lock()
        .await
        .prepare("INSERT INTO litera.secret_keys (user, private_key) VALUES(?, ?);")
        .await
        .unwrap();

    // append all statements to the batch
    batch.append_statement(prepared_user);
    batch.append_statement(prepared_secret);

    // define values to insert
    let user_values = (
        user.uuid,
        &user.username.to_owned(),
        &user.password.to_owned(),
        (&user.role.to_owned().get_index()).to_owned() as i8,
        &user.public_key_str().to_owned(),
        Timestamp(Duration::try_seconds(user.created_at).unwrap()),
    );
    let secret_values = (user.uuid, secret.to_vec());
    let batch_values = (user_values, secret_values);

    match session.lock().await.batch(&batch, batch_values).await {
        Ok(_) => Ok(StatusCode::CREATED),
        Err(_e) => Ok(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn update(
    id: String,
    _uid: (),
    user: User,
    session: Arc<Mutex<Session>>,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("update_user: id={}, user={:?}", id, user);

    if check_user_by_uuid(session.clone(), &user.uuid)
        .await
        .is_ok()
    {
        return Ok(StatusCode::NOT_FOUND);
    }

    if session
        .lock()
        .await
        .query(
            "UPDATE litera.users SET username = ? WHERE username = ?;",
            (user.username.to_owned(),),
        )
        .await
        .is_err()
    {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::OK)
}

pub async fn delete(
    user_uuid: String,
    _uid: (),
    session: Arc<Mutex<Session>>,
) -> Result<impl warp::Reply, Infallible> {
    log::debug!("delete_user: user_uuid={}", user_uuid);
    let user_uuid = Uuid::parse_str(&user_uuid).unwrap();

    if check_user_by_uuid(session.clone(), &user_uuid)
        .await
        .is_ok()
    {
        return Ok(StatusCode::NOT_FOUND);
    }

    match session
        .lock()
        .await
        .query(
            "DELETE FROM litera.users WHERE users.uuid = ?;",
            (user_uuid,),
        )
        .await
    {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Ok(StatusCode::SERVICE_UNAVAILABLE),
    }
}

pub async fn check_user_by_uuid(
    session: Arc<Mutex<Session>>,
    user_uuid: &Uuid,
) -> Result<(), DbError> {
    if let Some(rows) = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.users WHERE uuid = ? ALLOW FILTERING;",
            (user_uuid,),
        )
        .await
        .unwrap()
        .rows
    {
        if rows.len() > 0 {
            return Err(DbError::AlreadyExists);
        }
    }

    Ok(())
}

pub async fn check_user_by_username(
    session: Arc<Mutex<Session>>,
    username: &str,
) -> Result<(), DbError> {
    if let Some(rows) = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.users WHERE username = ? ALLOW FILTERING;",
            (username.to_owned(),),
        )
        .await
        .unwrap()
        .rows
    {
        if rows.len() > 0 {
            return Err(DbError::AlreadyExists);
        }
    }

    Ok(())
}

pub async fn get_uuid_by_token(session: Arc<Mutex<Session>>, token: &str) -> Result<Uuid, DbError> {
    let user_sessions = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.sessions WHERE jwt = ? ALLOW FILTERING;",
            (token,),
        )
        .await
        .unwrap()
        .first_row();

    match user_sessions {
        Ok(row) => {
            let user_session = row
                .into_typed::<(
                    String,
                    Uuid,
                    chrono::Duration,
                    String,
                    String,
                    String,
                    String,
                )>()
                .unwrap();

            Ok(user_session.1)
        }
        Err(_) => Err(DbError::NotFound),
    }
}

pub async fn get_key(
    id: String,
    _uid: (),
    session: Arc<Mutex<Session>>,
) -> Result<warp::reply::Response, Infallible> {
    // parsing UUID
    let user_uuid = Uuid::parse_str(&id).unwrap();

    // preparing the query
    let prepared = session
        .lock()
        .await
        .prepare("SELECT * FROM litera.secret_keys WHERE user = ? ALLOW FILTERING;")
        .await
        .unwrap();

    // executing the query
    match session.lock().await.execute(&prepared, (user_uuid,)).await {
        // if Ok => return the key
        Ok(row) => {
            let key = row
                .first_row()
                .unwrap()
                .into_typed::<(Uuid, Vec<u8>)>()
                .unwrap();
            Ok(warp::reply::json(&key.1).into_response())
        }
        // if Err => return error
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

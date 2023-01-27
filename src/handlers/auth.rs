use std::{convert::Infallible, sync::Arc};

use chrono::{Duration, Utc};
use orbis::{
    models::user::user::User,
    request::auth::{AuthRequest, AuthRequestMeta, LogoutRequest},
};
use scylla::{frame::value::Timestamp, prepared_statement::PreparedStatement, Session};
use tokio::sync::Mutex;
use warp::{hyper::StatusCode, reject, Reply};

use crate::{
    db::models_wrapper::UserDB,
    errors::{db::DbError, jwt::JWTError},
    filters::auth::check_token,
    jwt::generate_jwt,
};

use super::users::create;

pub async fn login(
    session: Arc<Mutex<Session>>,
    body: AuthRequest,
) -> Result<warp::reply::Response, Infallible> {
    let result = validate_user(session, body).await;

    match result {
        Ok(token) => Ok(warp::reply::json(&token).into_response()),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
    }
}

pub async fn register(
    session: Arc<Mutex<Session>>,
    body: AuthRequest,
) -> Result<warp::reply::Response, Infallible> {
    let user = User::new(&body.username, &body.password, None);
    let created = create(user, session.clone()).await.unwrap();

    if !created.is_success() {
        return Ok(created.into_response());
    }

    let result = validate_user(session, body).await;

    match result {
        Ok(token) => Ok(warp::reply::json(&token).into_response()),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
        .into_response()),
    }
}

pub async fn logout(
    session: Arc<Mutex<Session>>,
    body: LogoutRequest,
) -> Result<warp::reply::Response, Infallible> {
    let _token_decoded = check_token(session.clone(), body.token.clone())
        .await
        .map_err(|_| reject::custom(JWTError::JWTTokenError))
        .unwrap();

    match session
        .lock()
        .await
        .query("DELETE FROM litera.sessions WHERE jwt = ?;", (body.token,))
        .await
    {
        Ok(_) => Ok(StatusCode::UNAUTHORIZED.into_response()),
        Err(_) => Ok(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
    }
}

pub async fn validate_user(
    session: Arc<Mutex<Session>>,
    body: AuthRequest,
) -> Result<String, DbError> {
    let user_row = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.users WHERE username = ? ALLOW FILTERING;",
            (body.username,),
        )
        .await
        .unwrap()
        .first_row();

    if user_row.is_err() {
        return Err(DbError::WrongCredentials);
    }
    let user = user_row.unwrap().into_typed::<UserDB>();

    if user.is_err() {
        return Err(DbError::FailedToConvertRow);
    }

    let user = user.unwrap().get_user();

    if user.password != body.password {
        return Err(DbError::WrongCredentials);
    }

    add_jwt_session(session, &user, body.meta).await
}

pub async fn add_jwt_session(
    session: Arc<Mutex<Session>>,
    user: &User,
    meta: AuthRequestMeta,
) -> Result<String, DbError> {
    let user = user.to_owned();
    let token = generate_jwt(&user.uuid.to_string(), user.role.clone());

    if token.is_err() {
        // return Err(token.err().unwrap());
    }

    let token = token.unwrap();

    let prepared_result= session.lock().await
        .prepare(
            "INSERT INTO litera.sessions (jwt, user, location, device_name, device_type, device_os, created_at) VALUES(?, ?, ?, ?, ?, ?, ?);",
        )
        .await;

    if prepared_result.is_err() {
        return Err(DbError::FailedToAdd);
    }

    let prepared: PreparedStatement = prepared_result.unwrap();
    let session_result = session
        .lock()
        .await
        .execute(
            &prepared,
            (
                &token.to_owned(),
                user.uuid,
                meta.location,
                meta.device_name,
                meta.device_type,
                meta.device_os,
                Timestamp(Duration::seconds(Utc::now().timestamp())),
            ),
        )
        .await;

    if session_result.is_err() {
        return Err(DbError::FailedToAdd);
    }

    Ok(token)
}

pub async fn validate_session(session: Arc<Mutex<Session>>, token: &str) -> Result<(), DbError> {
    let result = session
        .lock()
        .await
        .query(
            "SELECT * FROM litera.sessions WHERE jwt = ? ALLOW FILTERING;",
            (token,),
        )
        .await
        .unwrap()
        .rows()
        .unwrap();

    if result.len() <= 0 {
        return Err(DbError::NotFound);
    }

    Ok(())
}

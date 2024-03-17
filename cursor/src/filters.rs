use std::sync::Arc;

use nexuslib::models::user::role::Role;
use scylla::Session;
use tokio::sync::Mutex;
use warp::{header::headers_cloned, http::HeaderValue, hyper::HeaderMap, Filter, Rejection};

use self::auth::authorize;

pub mod auth;
pub mod users;

pub fn with_session(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (Arc<Mutex<Session>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || session.clone())
}

pub fn with_auth(
    session: Arc<Mutex<Session>>,
    role: Role,
) -> impl Filter<Extract = ((),), Error = Rejection> + Clone {
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (role.clone(), headers))
        .and(with_session(session))
        .and_then(authorize)
}

use std::{convert::Infallible, sync::Arc};

use scylla::Session;
use tokio::sync::Mutex;
use warp::{Filter, Reply};

use crate::{errors::handle_rejection, api::filters};

/// Routes
///
/// All server routes have to be registered here
pub fn get_routes(
    session: Arc<Mutex<Session>>,
) -> impl Filter<Extract = (impl Reply,), Error = Infallible> + Clone {
    /*

    ---  AUTH    ---
    POST                     /auth/login
    POST                     /auth/register
    POST                     /auth/logout

    ---  USERS   ---
    GET                      /users
    GET | PUT | DELETE       /users/:uuid
    POST                     /users/key/:uuid

    */
    warp::path("api")
        .and(filters::users::users(session.clone()).or(filters::auth::auth(session.clone())))
        .with(warp::cors().allow_any_origin())
        .recover(handle_rejection)
}

use std::sync::Arc;

use scylla::Session;
use tokio::sync::Mutex;

use self::routes::get_routes;

pub mod filters;
pub mod handlers;
pub mod jwt;
pub mod routes;

pub async fn run_http(session: Arc<Mutex<Session>>) {
    let routes = get_routes(session);

    tokio::spawn(async move {
        warp::serve(routes)
            .tls()
            .cert_path(&std::env::var("TLS_CERT_PATH").unwrap())
            .key_path(&std::env::var("TLS_KEY_PATH").unwrap())
            .run(([127, 0, 0, 1], 8082))
            .await;
    });
}

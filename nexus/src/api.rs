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
            .cert_path("./certs/cert.pem")
            .key_path("./certs/key.pem")
            .run(([127, 0, 0, 1], 8082))
            .await;
    });
}

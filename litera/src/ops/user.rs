use std::io::Result;

use orbis::models::user::user::User;
use reqwest::Client;
use uuid::Uuid;

/// Register a new user
///
/// Receives: stream: &mut TcpStream, user: User
///
/// Returns Result
#[allow(unused)]
pub async fn get_user(user_uuid: Uuid) -> Result<Option<User>> {
    let body = reqwest::get(format!(
        "https://127.0.0.1:8082/api/users/{0}",
        user_uuid.to_string()
    ))
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

    match serde_json::from_str::<User>(&body) {
        Ok(user) => Ok(Some(user)),
        Err(_) => Ok(None),
    }
}

pub async fn get_users(client: Client, token: String) -> Result<Vec<User>> {
    let body = client
        .get(format!("https://127.0.0.1:8082/api/users"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    match serde_json::from_str::<Vec<User>>(&body) {
        Ok(user) => Ok(user),
        Err(err) => {
            println!("{err}");
            Ok(vec![])
        }
    }
}

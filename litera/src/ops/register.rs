use std::io::Result;

use orbis::models::user::user::User;

/// Register a new user
///
/// Receives: stream: &mut TcpStream, user: User
///
/// Returns Result
/// 
#[allow(unused)]
pub async fn register(user: User) -> Result<()> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    println!("{:?}", user);

    match client
        .post("http://127.0.0.1:8082/auth/register")
        .json(&user)
        .send()
        .await
    {
        Ok(_) => println!("Added"),
        Err(err) => println!("{err}"),
    }

    Ok(())
}

use nexuslib::models::user::{role::Role, user::User};
use scylla::FromRow;
use uuid::Uuid;

pub struct UserDB(User);

impl UserDB {
    pub fn get_user(&self) -> User {
        return self.0.to_owned();
    }
}

impl FromRow for UserDB {
    fn from_row(
        row: scylla::frame::response::result::Row,
    ) -> Result<Self, scylla::cql_to_rust::FromRowError> {
        let (uuid, username, created_at, password, public_key, role) = <(
            Uuid,
            Option<String>,
            chrono::Duration,
            Option<String>,
            Option<String>,
            i8,
        )>::from_row(row)?;

        let role = serde_json::from_str::<Role>(&role.to_string()).unwrap();

        Ok(Self(User {
            uuid: uuid,
            username: username.unwrap(),
            password: password.unwrap(),
            role: role,
            public_key: public_key.unwrap(),
            created_at: created_at.num_seconds(),
        }))
    }
}

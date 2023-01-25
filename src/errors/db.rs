use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum DbError {
    FailedToAdd,
    AlreadyExists,
    FailedToConvertRow,
    WrongCredentials,
    NotFound,
}

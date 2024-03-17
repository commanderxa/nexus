use serde::Serialize;

#[derive(Debug, Serialize)]
pub enum DbError {
    FailedToAdd,
    FailedToUpdate,
    AlreadyExists,
    FailedToConvertRow,
    WrongCredentials,
    NotFound,
}

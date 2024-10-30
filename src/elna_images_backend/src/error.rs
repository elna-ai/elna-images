use candid::CandidType;

#[derive(Debug, thiserror::Error, PartialEq, CandidType)]
pub enum Error {
    #[error("Owner and Caller does not match")]
    UploaderMismatch,

    #[error("Collection doesn't exist")]
    NotFound,

    #[error("User not authorized")]
    Unauthorized,
    #[error("Unable to delete asset")]
    UnableToDelete,
}
impl From<Error> for String {
    fn from(error: Error) -> Self {
        // Convert the Error to a String representation
        error.to_string()
    }
}

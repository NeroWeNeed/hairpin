use http::uri::InvalidUri;
use tonic::Status;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The following uri is prohibited: {0}")]
    ProhibitedUri(String),
    #[error(transparent)]
    InvalidManifest(#[from] manifest::path::Error),
}
impl From<Error> for Status {
    fn from(value: Error) -> Self {
        match &value {
            Error::ProhibitedUri(_) => Status::permission_denied(value.to_string()),
            Error::InvalidManifest(_) => Status::invalid_argument(value.to_string()),
        }
    }
}
impl From<InvalidUri> for Error {
    fn from(value: InvalidUri) -> Self {
        Self::ProhibitedUri(value.to_string())
    }
}

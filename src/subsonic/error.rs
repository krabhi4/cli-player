use thiserror::Error;

#[derive(Error, Debug)]
pub enum SubsonicError {
    #[error("Subsonic API error {code}: {message}")]
    Api { code: i64, message: String },

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}

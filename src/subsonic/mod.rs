pub mod auth;
pub mod client;
pub mod error;
pub mod models;

pub use client::SubsonicClient;
pub use models::{Album, Artist, Genre, Playlist, Song};

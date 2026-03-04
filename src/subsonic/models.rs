use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Song {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_id: String,
    pub artist_id: String,
    pub duration: u64,
    pub track: u32,
    pub year: u32,
    pub genre: String,
    pub size: u64,
    pub suffix: String,
    pub bitrate: u32,
    pub cover_art_id: String,
}

impl Default for Song {
    fn default() -> Self {
        Self {
            id: String::new(),
            title: "Unknown".to_string(),
            artist: "Unknown Artist".to_string(),
            album: "Unknown Album".to_string(),
            album_id: String::new(),
            artist_id: String::new(),
            duration: 0,
            track: 0,
            year: 0,
            genre: String::new(),
            size: 0,
            suffix: String::new(),
            bitrate: 0,
            cover_art_id: String::new(),
        }
    }
}

impl Song {
    pub fn from_api(data: &Value) -> Self {
        Self {
            id: val_str(data, "id"),
            title: data["title"].as_str().unwrap_or("Unknown").to_string(),
            artist: data["artist"]
                .as_str()
                .unwrap_or("Unknown Artist")
                .to_string(),
            album: data["album"]
                .as_str()
                .unwrap_or("Unknown Album")
                .to_string(),
            album_id: val_str(data, "albumId"),
            artist_id: val_str(data, "artistId"),
            duration: data["duration"].as_u64().unwrap_or(0),
            track: data["track"].as_u64().unwrap_or(0) as u32,
            year: data["year"].as_u64().unwrap_or(0) as u32,
            genre: data["genre"].as_str().unwrap_or("").to_string(),
            size: data["size"].as_u64().unwrap_or(0),
            suffix: data["suffix"].as_str().unwrap_or("").to_string(),
            bitrate: data["bitRate"].as_u64().unwrap_or(0) as u32,
            cover_art_id: val_str(data, "coverArt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Album {
    pub id: String,
    pub name: String,
    pub artist: String,
    pub artist_id: String,
    pub song_count: u32,
    pub duration: u64,
    pub year: u32,
    pub genre: String,
    pub cover_art_id: String,
}

impl Default for Album {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Unknown Album".to_string(),
            artist: "Unknown Artist".to_string(),
            artist_id: String::new(),
            song_count: 0,
            duration: 0,
            year: 0,
            genre: String::new(),
            cover_art_id: String::new(),
        }
    }
}

impl Album {
    pub fn from_api(data: &Value) -> Self {
        let name = data["name"]
            .as_str()
            .or_else(|| data["album"].as_str())
            .unwrap_or("Unknown Album")
            .to_string();
        Self {
            id: val_str(data, "id"),
            name,
            artist: data["artist"]
                .as_str()
                .unwrap_or("Unknown Artist")
                .to_string(),
            artist_id: val_str(data, "artistId"),
            song_count: data["songCount"].as_u64().unwrap_or(0) as u32,
            duration: data["duration"].as_u64().unwrap_or(0),
            year: data["year"].as_u64().unwrap_or(0) as u32,
            genre: data["genre"].as_str().unwrap_or("").to_string(),
            cover_art_id: val_str(data, "coverArt"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub album_count: u32,
    pub cover_art_id: String,
}

impl Default for Artist {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Unknown Artist".to_string(),
            album_count: 0,
            cover_art_id: String::new(),
        }
    }
}

impl Artist {
    pub fn from_api(data: &Value) -> Self {
        let cover_art = data["coverArt"]
            .as_str()
            .or_else(|| data["artistImageUrl"].as_str())
            .unwrap_or("")
            .to_string();
        Self {
            id: val_str(data, "id"),
            name: data["name"]
                .as_str()
                .unwrap_or("Unknown Artist")
                .to_string(),
            album_count: data["albumCount"].as_u64().unwrap_or(0) as u32,
            cover_art_id: cover_art,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub song_count: u32,
    pub duration: u64,
    pub owner: String,
    pub public: bool,
}

impl Default for Playlist {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: "Unknown Playlist".to_string(),
            song_count: 0,
            duration: 0,
            owner: String::new(),
            public: false,
        }
    }
}

impl Playlist {
    pub fn from_api(data: &Value) -> Self {
        Self {
            id: val_str(data, "id"),
            name: data["name"]
                .as_str()
                .unwrap_or("Unknown Playlist")
                .to_string(),
            song_count: data["songCount"].as_u64().unwrap_or(0) as u32,
            duration: data["duration"].as_u64().unwrap_or(0),
            owner: data["owner"].as_str().unwrap_or("").to_string(),
            public: data["public"].as_bool().unwrap_or(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genre {
    pub name: String,
    pub song_count: u32,
    pub album_count: u32,
}

impl Default for Genre {
    fn default() -> Self {
        Self {
            name: "Unknown".to_string(),
            song_count: 0,
            album_count: 0,
        }
    }
}

impl Genre {
    pub fn from_api(data: &Value) -> Self {
        Self {
            name: data["value"].as_str().unwrap_or("Unknown").to_string(),
            song_count: data["songCount"].as_u64().unwrap_or(0) as u32,
            album_count: data["albumCount"].as_u64().unwrap_or(0) as u32,
        }
    }
}

/// Extract a string value from JSON, converting numbers to strings.
fn val_str(data: &Value, key: &str) -> String {
    match &data[key] {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        _ => String::new(),
    }
}

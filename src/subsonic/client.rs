use std::collections::HashMap;
use std::time::Duration;

use reqwest::Client;
use serde_json::Value;

use super::auth::auth_params;
use super::error::SubsonicError;
use super::models::{Album, Artist, Genre, Playlist, Song};

pub struct SubsonicClient {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl SubsonicClient {
    pub fn new(base_url: &str, username: &str, password: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    fn get_auth_params(&self) -> HashMap<String, String> {
        auth_params(&self.username, &self.password)
    }

    async fn request(
        &self,
        endpoint: &str,
        extra_params: &[(&str, &str)],
    ) -> Result<Value, SubsonicError> {
        let url = format!("{}/rest/{}", self.base_url, endpoint);
        let mut params = self.get_auth_params();
        for (k, v) in extra_params {
            params.insert(k.to_string(), v.to_string());
        }

        let resp = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    SubsonicError::Connection(format!("Request timed out: {e}"))
                } else if e.is_connect() {
                    SubsonicError::Connection(format!("Cannot connect to {}: {e}", self.base_url))
                } else {
                    SubsonicError::Request(e)
                }
            })?;

        let data: Value = resp.json().await?;
        let sub_response = &data["subsonic-response"];

        if sub_response["status"].as_str() == Some("failed") {
            let error = &sub_response["error"];
            return Err(SubsonicError::Api {
                code: error["code"].as_i64().unwrap_or(-1),
                message: error["message"]
                    .as_str()
                    .unwrap_or("Unknown error")
                    .to_string(),
            });
        }

        Ok(sub_response.clone())
    }

    // ── Connection ──────────────────────────────────────────────────

    pub async fn ping(&self) -> Result<bool, SubsonicError> {
        match self.request("ping.view", &[]).await {
            Ok(resp) => Ok(resp["status"].as_str() == Some("ok")),
            Err(_) => Ok(false),
        }
    }

    // ── Browsing ────────────────────────────────────────────────────

    pub async fn get_artists(&self) -> Result<Vec<Artist>, SubsonicError> {
        let resp = self.request("getArtists.view", &[]).await?;
        let mut result = Vec::new();
        if let Some(indices) = resp["artists"]["index"].as_array() {
            for index in indices {
                if let Some(artists) = index["artist"].as_array() {
                    for a in artists {
                        result.push(Artist::from_api(a));
                    }
                }
            }
        }
        Ok(result)
    }

    pub async fn get_artist(&self, artist_id: &str) -> Result<(Artist, Vec<Album>), SubsonicError> {
        let resp = self.request("getArtist.view", &[("id", artist_id)]).await?;
        let artist_data = &resp["artist"];
        let artist = Artist::from_api(artist_data);
        let albums = artist_data["album"]
            .as_array()
            .map(|arr| arr.iter().map(Album::from_api).collect())
            .unwrap_or_default();
        Ok((artist, albums))
    }

    pub async fn get_album(&self, album_id: &str) -> Result<(Album, Vec<Song>), SubsonicError> {
        let resp = self.request("getAlbum.view", &[("id", album_id)]).await?;
        let album_data = &resp["album"];
        let album = Album::from_api(album_data);
        let songs = album_data["song"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok((album, songs))
    }

    pub async fn get_album_list(
        &self,
        list_type: &str,
        size: u32,
        offset: u32,
    ) -> Result<Vec<Album>, SubsonicError> {
        let size_str = size.to_string();
        let offset_str = offset.to_string();
        let resp = self
            .request(
                "getAlbumList2.view",
                &[
                    ("type", list_type),
                    ("size", &size_str),
                    ("offset", &offset_str),
                ],
            )
            .await?;
        let albums = resp["albumList2"]["album"]
            .as_array()
            .map(|arr| arr.iter().map(Album::from_api).collect())
            .unwrap_or_default();
        Ok(albums)
    }

    pub async fn get_song(&self, song_id: &str) -> Result<Song, SubsonicError> {
        let resp = self.request("getSong.view", &[("id", song_id)]).await?;
        Ok(Song::from_api(&resp["song"]))
    }

    pub async fn get_random_songs(
        &self,
        size: u32,
        genre: &str,
    ) -> Result<Vec<Song>, SubsonicError> {
        let size_str = size.to_string();
        let mut params = vec![("size", size_str.as_str())];
        if !genre.is_empty() {
            params.push(("genre", genre));
        }
        let resp = self.request("getRandomSongs.view", &params).await?;
        let songs = resp["randomSongs"]["song"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok(songs)
    }

    // ── Search ──────────────────────────────────────────────────────

    pub async fn search(
        &self,
        query: &str,
        artist_count: u32,
        album_count: u32,
        song_count: u32,
    ) -> Result<(Vec<Artist>, Vec<Album>, Vec<Song>), SubsonicError> {
        let ac = artist_count.to_string();
        let alc = album_count.to_string();
        let sc = song_count.to_string();
        let resp = self
            .request(
                "search3.view",
                &[
                    ("query", query),
                    ("artistCount", &ac),
                    ("albumCount", &alc),
                    ("songCount", &sc),
                ],
            )
            .await?;
        let results = &resp["searchResult3"];
        let artists = results["artist"]
            .as_array()
            .map(|a| a.iter().map(Artist::from_api).collect())
            .unwrap_or_default();
        let albums = results["album"]
            .as_array()
            .map(|a| a.iter().map(Album::from_api).collect())
            .unwrap_or_default();
        let songs = results["song"]
            .as_array()
            .map(|a| a.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok((artists, albums, songs))
    }

    // ── Genres ──────────────────────────────────────────────────────

    pub async fn get_genres(&self) -> Result<Vec<Genre>, SubsonicError> {
        let resp = self.request("getGenres.view", &[]).await?;
        let genres = resp["genres"]["genre"]
            .as_array()
            .map(|arr| arr.iter().map(Genre::from_api).collect())
            .unwrap_or_default();
        Ok(genres)
    }

    pub async fn get_songs_by_genre(
        &self,
        genre: &str,
        count: u32,
        offset: u32,
    ) -> Result<Vec<Song>, SubsonicError> {
        let count_str = count.to_string();
        let offset_str = offset.to_string();
        let resp = self
            .request(
                "getSongsByGenre.view",
                &[
                    ("genre", genre),
                    ("count", &count_str),
                    ("offset", &offset_str),
                ],
            )
            .await?;
        let songs = resp["songsByGenre"]["song"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok(songs)
    }

    // ── Playlists ───────────────────────────────────────────────────

    pub async fn get_playlists(&self) -> Result<Vec<Playlist>, SubsonicError> {
        let resp = self.request("getPlaylists.view", &[]).await?;
        let playlists = resp["playlists"]["playlist"]
            .as_array()
            .map(|arr| arr.iter().map(Playlist::from_api).collect())
            .unwrap_or_default();
        Ok(playlists)
    }

    pub async fn get_playlist(
        &self,
        playlist_id: &str,
    ) -> Result<(Playlist, Vec<Song>), SubsonicError> {
        let resp = self
            .request("getPlaylist.view", &[("id", playlist_id)])
            .await?;
        let pl_data = &resp["playlist"];
        let playlist = Playlist::from_api(pl_data);
        let songs = pl_data["entry"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok((playlist, songs))
    }

    pub async fn create_playlist(
        &self,
        name: &str,
        song_ids: &[&str],
    ) -> Result<String, SubsonicError> {
        let mut params: Vec<(&str, &str)> = vec![("name", name)];
        for id in song_ids {
            params.push(("songId", id));
        }
        let resp = self.request("createPlaylist.view", &params).await?;
        let id = resp["playlist"]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(id)
    }

    // ── Starred / Favourites ────────────────────────────────────────

    pub async fn get_starred(
        &self,
    ) -> Result<(Vec<Artist>, Vec<Album>, Vec<Song>), SubsonicError> {
        let resp = self.request("getStarred2.view", &[]).await?;
        let starred = &resp["starred2"];
        let artists = starred["artist"]
            .as_array()
            .map(|a| a.iter().map(Artist::from_api).collect())
            .unwrap_or_default();
        let albums = starred["album"]
            .as_array()
            .map(|a| a.iter().map(Album::from_api).collect())
            .unwrap_or_default();
        let songs = starred["song"]
            .as_array()
            .map(|a| a.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok((artists, albums, songs))
    }

    pub async fn star(&self, id: &str) -> Result<(), SubsonicError> {
        self.request("star.view", &[("id", id)]).await?;
        Ok(())
    }

    pub async fn unstar(&self, id: &str) -> Result<(), SubsonicError> {
        self.request("unstar.view", &[("id", id)]).await?;
        Ok(())
    }

    // ── Scrobbling ──────────────────────────────────────────────────

    pub async fn scrobble(&self, song_id: &str, submission: bool) -> Result<(), SubsonicError> {
        let sub_str = if submission { "true" } else { "false" };
        self.request("scrobble.view", &[("id", song_id), ("submission", sub_str)])
            .await?;
        Ok(())
    }

    pub async fn now_playing(&self, song_id: &str) -> Result<(), SubsonicError> {
        self.scrobble(song_id, false).await
    }

    // ── Lyrics ──────────────────────────────────────────────────────

    pub async fn get_lyrics(&self, artist: &str, title: &str) -> Result<String, SubsonicError> {
        let mut params = Vec::new();
        if !artist.is_empty() {
            params.push(("artist", artist));
        }
        if !title.is_empty() {
            params.push(("title", title));
        }
        let resp = self.request("getLyrics.view", &params).await?;
        let lyrics = resp["lyrics"]["value"]
            .as_str()
            .unwrap_or("")
            .to_string();
        Ok(lyrics)
    }

    // ── Similar / Discovery ─────────────────────────────────────────

    pub async fn get_similar_songs(
        &self,
        song_id: &str,
        count: u32,
    ) -> Result<Vec<Song>, SubsonicError> {
        let count_str = count.to_string();
        let resp = self
            .request(
                "getSimilarSongs2.view",
                &[("id", song_id), ("count", &count_str)],
            )
            .await?;
        let songs = resp["similarSongs2"]["song"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok(songs)
    }

    pub async fn get_top_songs(
        &self,
        artist_name: &str,
        count: u32,
    ) -> Result<Vec<Song>, SubsonicError> {
        let count_str = count.to_string();
        let resp = self
            .request(
                "getTopSongs.view",
                &[("artist", artist_name), ("count", &count_str)],
            )
            .await?;
        let songs = resp["topSongs"]["song"]
            .as_array()
            .map(|arr| arr.iter().map(Song::from_api).collect())
            .unwrap_or_default();
        Ok(songs)
    }

    // ── URL Builders ────────────────────────────────────────────────

    pub fn stream_url(&self, song_id: &str) -> String {
        let params = self.get_auth_params();
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencod(k), urlencod(v)))
            .collect();
        pairs.push(format!("id={}", urlencod(song_id)));
        pairs.push("format=raw".to_string());
        format!("{}/rest/stream.view?{}", self.base_url, pairs.join("&"))
    }

    pub fn cover_art_url(&self, cover_id: &str, size: u32) -> String {
        if cover_id.is_empty() {
            return String::new();
        }
        let params = self.get_auth_params();
        let mut pairs: Vec<String> = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencod(k), urlencod(v)))
            .collect();
        pairs.push(format!("id={}", urlencod(cover_id)));
        pairs.push(format!("size={size}"));
        format!(
            "{}/rest/getCoverArt.view?{}",
            self.base_url,
            pairs.join("&")
        )
    }
}

/// Percent-encode a string for use in URL query parameters.
fn urlencod(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(b as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", b));
            }
        }
    }
    result
}

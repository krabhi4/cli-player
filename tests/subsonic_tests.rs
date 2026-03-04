use cli_music_player::subsonic::models::{Album, Artist, Genre, Playlist, Song};
use serde_json::json;

// ── Song Model Tests ────────────────────────────────────────────

#[test]
fn test_song_from_api() {
    let data = json!({
        "id": "123",
        "title": "Test Song",
        "artist": "Test Artist",
        "album": "Test Album",
        "albumId": "album1",
        "artistId": "artist1",
        "duration": 240,
        "track": 5,
        "year": 2024,
        "genre": "Rock",
        "size": 5242880,
        "suffix": "mp3",
        "bitRate": 320,
        "coverArt": "cover1"
    });

    let song = Song::from_api(&data);
    assert_eq!(song.id, "123");
    assert_eq!(song.title, "Test Song");
    assert_eq!(song.artist, "Test Artist");
    assert_eq!(song.album, "Test Album");
    assert_eq!(song.album_id, "album1");
    assert_eq!(song.artist_id, "artist1");
    assert_eq!(song.duration, 240);
    assert_eq!(song.track, 5);
    assert_eq!(song.year, 2024);
    assert_eq!(song.genre, "Rock");
    assert_eq!(song.size, 5242880);
    assert_eq!(song.suffix, "mp3");
    assert_eq!(song.bitrate, 320);
    assert_eq!(song.cover_art_id, "cover1");
}

#[test]
fn test_song_from_api_defaults() {
    let data = json!({
        "id": "456"
    });

    let song = Song::from_api(&data);
    assert_eq!(song.id, "456");
    assert_eq!(song.title, "Unknown");
    assert_eq!(song.artist, "Unknown Artist");
    assert_eq!(song.album, "Unknown Album");
    assert_eq!(song.duration, 0);
    assert_eq!(song.track, 0);
    assert_eq!(song.bitrate, 0);
}

#[test]
fn test_song_from_api_numeric_id() {
    let data = json!({
        "id": 789,
        "title": "Numeric ID Song"
    });

    let song = Song::from_api(&data);
    assert_eq!(song.id, "789");
}

// ── Album Model Tests ───────────────────────────────────────────

#[test]
fn test_album_from_api() {
    let data = json!({
        "id": "alb1",
        "name": "Test Album",
        "artist": "Test Artist",
        "artistId": "art1",
        "songCount": 12,
        "duration": 3600,
        "year": 2023,
        "genre": "Pop",
        "coverArt": "cov1"
    });

    let album = Album::from_api(&data);
    assert_eq!(album.id, "alb1");
    assert_eq!(album.name, "Test Album");
    assert_eq!(album.artist, "Test Artist");
    assert_eq!(album.song_count, 12);
    assert_eq!(album.duration, 3600);
    assert_eq!(album.year, 2023);
}

#[test]
fn test_album_from_api_fallback_name() {
    // When "name" is missing, should fall back to "album"
    let data = json!({
        "id": "alb2",
        "album": "Fallback Name"
    });

    let album = Album::from_api(&data);
    assert_eq!(album.name, "Fallback Name");
}

#[test]
fn test_album_from_api_defaults() {
    let data = json!({ "id": "alb3" });

    let album = Album::from_api(&data);
    assert_eq!(album.name, "Unknown Album");
    assert_eq!(album.artist, "Unknown Artist");
    assert_eq!(album.song_count, 0);
}

// ── Artist Model Tests ──────────────────────────────────────────

#[test]
fn test_artist_from_api() {
    let data = json!({
        "id": "art1",
        "name": "Test Artist",
        "albumCount": 5,
        "coverArt": "artcover1"
    });

    let artist = Artist::from_api(&data);
    assert_eq!(artist.id, "art1");
    assert_eq!(artist.name, "Test Artist");
    assert_eq!(artist.album_count, 5);
    assert_eq!(artist.cover_art_id, "artcover1");
}

#[test]
fn test_artist_from_api_fallback_image() {
    let data = json!({
        "id": "art2",
        "name": "Artist",
        "artistImageUrl": "https://example.com/image.jpg"
    });

    let artist = Artist::from_api(&data);
    assert_eq!(artist.cover_art_id, "https://example.com/image.jpg");
}

#[test]
fn test_artist_from_api_defaults() {
    let data = json!({ "id": "art3" });

    let artist = Artist::from_api(&data);
    assert_eq!(artist.name, "Unknown Artist");
    assert_eq!(artist.album_count, 0);
}

// ── Playlist Model Tests ────────────────────────────────────────

#[test]
fn test_playlist_from_api() {
    let data = json!({
        "id": "pl1",
        "name": "My Playlist",
        "songCount": 25,
        "duration": 5400,
        "owner": "admin",
        "public": true
    });

    let playlist = Playlist::from_api(&data);
    assert_eq!(playlist.id, "pl1");
    assert_eq!(playlist.name, "My Playlist");
    assert_eq!(playlist.song_count, 25);
    assert_eq!(playlist.duration, 5400);
    assert_eq!(playlist.owner, "admin");
    assert!(playlist.public);
}

#[test]
fn test_playlist_from_api_defaults() {
    let data = json!({ "id": "pl2" });

    let playlist = Playlist::from_api(&data);
    assert_eq!(playlist.name, "Unknown Playlist");
    assert_eq!(playlist.song_count, 0);
    assert!(!playlist.public);
}

// ── Genre Model Tests ───────────────────────────────────────────

#[test]
fn test_genre_from_api() {
    let data = json!({
        "value": "Rock",
        "songCount": 150,
        "albumCount": 20
    });

    let genre = Genre::from_api(&data);
    assert_eq!(genre.name, "Rock");
    assert_eq!(genre.song_count, 150);
    assert_eq!(genre.album_count, 20);
}

#[test]
fn test_genre_from_api_defaults() {
    let data = json!({});

    let genre = Genre::from_api(&data);
    assert_eq!(genre.name, "Unknown");
    assert_eq!(genre.song_count, 0);
}

// ── Auth Tests ──────────────────────────────────────────────────

#[test]
fn test_auth_params_contains_required_fields() {
    use cli_music_player::subsonic::auth::auth_params;

    let params = auth_params("testuser", "testpass");
    assert_eq!(params["u"], "testuser");
    assert!(params.contains_key("t"));
    assert!(params.contains_key("s"));
    assert_eq!(params["v"], "1.16.1");
    assert_eq!(params["c"], "CLIMusicPlayer");
    assert_eq!(params["f"], "json");
}

#[test]
fn test_auth_params_unique_salt() {
    use cli_music_player::subsonic::auth::auth_params;

    let params1 = auth_params("user", "pass");
    let params2 = auth_params("user", "pass");
    // Salt should be different each time
    assert_ne!(params1["s"], params2["s"]);
    // Token should be different due to different salt
    assert_ne!(params1["t"], params2["t"]);
}

#[test]
fn test_auth_params_md5_format() {
    use cli_music_player::subsonic::auth::auth_params;

    let params = auth_params("user", "pass");
    let token = &params["t"];
    // MD5 hex should be 32 characters
    assert_eq!(token.len(), 32);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

// ── URL Builder Tests ──────────────────────────────────────────

#[test]
fn test_stream_url_contains_required_params() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com", "user", "pass");
    let url = client.stream_url("song123");

    assert!(url.starts_with("https://music.example.com/rest/stream.view?"));
    assert!(url.contains("id=song123"));
    assert!(url.contains("format=raw"));
    assert!(url.contains("u=user"));
    assert!(url.contains("v="));
    assert!(url.contains("c="));
    assert!(url.contains("t="));
    assert!(url.contains("s="));
}

#[test]
fn test_stream_url_encodes_special_chars() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com", "user@domain", "p@ss w0rd!");
    let url = client.stream_url("song-with spaces&special=chars");

    // Special characters should be percent-encoded
    assert!(!url.contains(' '));
    assert!(url.contains("song-with%20spaces%26special%3Dchars"));
    assert!(url.contains("u=user%40domain"));
}

#[test]
fn test_cover_art_url_empty_id() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com", "user", "pass");
    let url = client.cover_art_url("", 300);

    assert!(url.is_empty());
}

#[test]
fn test_cover_art_url_valid() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com", "user", "pass");
    let url = client.cover_art_url("cover123", 300);

    assert!(url.starts_with("https://music.example.com/rest/getCoverArt.view?"));
    assert!(url.contains("id=cover123"));
    assert!(url.contains("size=300"));
}

#[test]
fn test_stream_url_trailing_slash_stripped() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com/", "user", "pass");
    let url = client.stream_url("song1");

    // Should not have double slash
    assert!(!url.contains("//rest"));
}

#[test]
fn test_client_accessors() {
    use cli_music_player::subsonic::SubsonicClient;

    let client = SubsonicClient::new("https://music.example.com/", "myuser", "mypass");
    assert_eq!(client.base_url(), "https://music.example.com");
    assert_eq!(client.username(), "myuser");
    assert_eq!(client.password(), "mypass");
}

// ── Model Edge Cases ──────────────────────────────────────────

#[test]
fn test_song_from_api_empty_json() {
    let data = json!({});
    let song = Song::from_api(&data);
    // Should use defaults, not panic
    assert_eq!(song.id, "");
    assert_eq!(song.title, "Unknown");
}

#[test]
fn test_song_from_api_wrong_types() {
    let data = json!({
        "id": true,
        "title": 42,
        "duration": "not a number"
    });
    // Should handle gracefully
    let song = Song::from_api(&data);
    assert_eq!(song.duration, 0); // Fallback
}

#[test]
fn test_album_from_api_negative_values() {
    let data = json!({
        "id": "alb1",
        "songCount": -5,
        "year": -1
    });
    let album = Album::from_api(&data);
    // Should not panic; u64/i64 handling varies
    assert_eq!(album.id, "alb1");
}

#[test]
fn test_genre_from_api_unicode() {
    let data = json!({
        "value": "ロック",
        "songCount": 42
    });
    let genre = Genre::from_api(&data);
    assert_eq!(genre.name, "ロック");
    assert_eq!(genre.song_count, 42);
}

#[test]
fn test_song_clone_and_default() {
    let song1 = Song::default();
    let song2 = song1.clone();
    assert_eq!(song1.id, song2.id);
    assert_eq!(song1.title, song2.title);
}

#[test]
fn test_auth_params_empty_password() {
    use cli_music_player::subsonic::auth::auth_params;

    let params = auth_params("user", "");
    // Should still produce valid params
    assert_eq!(params["u"], "user");
    assert!(params.contains_key("t"));
    assert!(params.contains_key("s"));
}

#[test]
fn test_auth_params_unicode_credentials() {
    use cli_music_player::subsonic::auth::auth_params;

    let params = auth_params("пользователь", "пароль");
    assert_eq!(params["u"], "пользователь");
    // Token should be valid hex
    assert_eq!(params["t"].len(), 32);
}

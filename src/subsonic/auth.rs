use md5::{Digest, Md5};
use rand::Rng;
use std::collections::HashMap;

pub const API_VERSION: &str = "1.16.1";
pub const CLIENT_NAME: &str = "CLIMusicPlayer";

/// Generate authentication parameters using the token+salt method.
pub fn auth_params(username: &str, password: &str) -> HashMap<String, String> {
    let salt: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let mut hasher = Md5::new();
    hasher.update(format!("{password}{salt}").as_bytes());
    let token = hex::encode(hasher.finalize());

    let mut params = HashMap::new();
    params.insert("u".to_string(), username.to_string());
    params.insert("t".to_string(), token);
    params.insert("s".to_string(), salt);
    params.insert("v".to_string(), API_VERSION.to_string());
    params.insert("c".to_string(), CLIENT_NAME.to_string());
    params.insert("f".to_string(), "json".to_string());
    params
}

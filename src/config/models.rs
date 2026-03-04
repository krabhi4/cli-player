use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub name: String,
    pub url: String,
    pub username: String,
    pub encrypted_password: String,
}

impl ServerConfig {
    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "name": self.name,
            "url": self.url,
            "username": self.username,
            "_encrypted_password": self.encrypted_password,
        })
    }

    pub fn from_value(data: &Value) -> Option<Self> {
        Some(Self {
            name: data["name"].as_str()?.to_string(),
            url: data["url"].as_str()?.to_string(),
            username: data["username"].as_str()?.to_string(),
            encrypted_password: data["_encrypted_password"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EQPreset {
    pub name: String,
    pub gains: Vec<f64>,
}

impl EQPreset {
    pub fn to_value(&self) -> Value {
        serde_json::json!({
            "name": self.name,
            "gains": self.gains,
        })
    }

    pub fn from_value(data: &Value) -> Option<Self> {
        Some(Self {
            name: data["name"].as_str()?.to_string(),
            gains: data["gains"]
                .as_array()?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect(),
        })
    }
}

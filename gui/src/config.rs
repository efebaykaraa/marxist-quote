use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use sha1::{Sha1, Digest};

/// Mirrors engyls::Appearance — must stay in sync for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appearance {
    pub font: String,
    pub font_size: f32,
    pub text_color: String,
    pub bg_color: String,
    pub bg_enabled: bool,
    pub stroke_color: String,
    pub stroke_enabled: bool,
    pub stroke_width: f32,
    pub shadow_color: String,
    pub shadow_enabled: bool,
    pub shadow_offset: f32,
    pub quote_x: i32,
    pub quote_y: i32,
    pub author_x: i32,
    pub author_y: i32,
    #[serde(default = "default_quote_max_width")]
    pub quote_max_width: i32,
    #[serde(default = "default_quote_max_height")]
    pub quote_max_height: i32,
    #[serde(default = "default_max_quote_chars")]
    pub max_quote_chars: usize,
}

fn default_quote_max_width() -> i32 { 800 }
fn default_quote_max_height() -> i32 { 300 }
fn default_max_quote_chars() -> usize { 500 }

/// Mirrors engyls::DisplayArgs — must stay in sync for JSON compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsConfig {
    pub appearance: Appearance,
}

impl Default for SettingsConfig {
    fn default() -> Self {
        Self {
            appearance: Appearance {
                font: "Inter".into(),
                font_size: 24.0,
                text_color: "#ffffff".into(),
                bg_color: "#00000080".into(),
                bg_enabled: false,
                stroke_color: "#000000".into(),
                stroke_enabled: true,
                stroke_width: 2.0,
                shadow_color: "#000000ff".into(),
                shadow_enabled: true,
                shadow_offset: 2.0,
                quote_x: 100,
                quote_y: 100,
                author_x: 100,
                author_y: 200,
                quote_max_width: 800,
                quote_max_height: 300,
                max_quote_chars: 500,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    pub name: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorsConfig {
    pub authors: Vec<Author>,
}

impl Default for AuthorsConfig {
    fn default() -> Self {
        Self {
            authors: vec![
                Author { name: "Karl Marx".into(), weight: 3 },
                Author { name: "Friedrich Engels".into(), weight: 2 },
                Author { name: "Vladimir Lenin".into(), weight: 2 },
            ],
        }
    }
}

pub struct ConfigManager;

impl ConfigManager {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("marxist_quote")
    }

    pub fn authors_path() -> PathBuf {
        Self::config_dir().join("authors.json")
    }

    pub fn settings_path() -> PathBuf {
        Self::config_dir().join("settings.json")
    }

    fn load_hashed_json<T: for<'de> Deserialize<'de> + Default>(path: &PathBuf) -> (T, String) {
        if let Ok(contents) = std::fs::read_to_string(path) {
            let mut json_part = String::new();
            let mut file_hash = String::new();
            
            for line in contents.lines() {
                if line.starts_with("hash:") {
                    file_hash = line["hash:".len()..].to_string();
                } else {
                    json_part.push_str(line);
                    json_part.push('\n');
                }
            }
            
            if let Ok(data) = serde_json::from_str(&json_part) {
                return (data, file_hash);
            }
        }
        (T::default(), String::new())
    }

    fn save_hashed_json<T: Serialize>(path: &PathBuf, data: &T) -> anyhow::Result<String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json_str = serde_json::to_string_pretty(data)?;
        
        let mut hasher = Sha1::new();
        hasher.update(json_str.as_bytes());
        let new_hash = format!("{:x}", hasher.finalize());
        
        let final_content = format!("{}\nhash:{}", json_str, new_hash);
        std::fs::write(path, final_content)?;
        
        Ok(new_hash)
    }

    pub fn load_authors() -> (AuthorsConfig, String) {
        Self::load_hashed_json(&Self::authors_path())
    }

    pub fn load_settings() -> (SettingsConfig, String) {
        Self::load_hashed_json(&Self::settings_path())
    }

    pub fn save_authors(data: &AuthorsConfig) -> anyhow::Result<String> {
        Self::save_hashed_json(&Self::authors_path(), data)
    }

    pub fn save_settings(data: &SettingsConfig) -> anyhow::Result<String> {
        Self::save_hashed_json(&Self::settings_path(), data)
    }
}

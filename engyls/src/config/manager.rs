use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sha1::{Sha1, Digest};
use crate::config::types::{AuthorsConfig, DisplayArgs};

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
            let _ = std::fs::create_dir_all(parent);
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

    pub fn load_settings() -> (DisplayArgs, String) {
        Self::load_hashed_json(&Self::settings_path())
    }

    pub fn save_authors(data: &AuthorsConfig) -> anyhow::Result<String> {
        Self::save_hashed_json(&Self::authors_path(), data)
    }

    pub fn save_settings(data: &DisplayArgs) -> anyhow::Result<String> {
        Self::save_hashed_json(&Self::settings_path(), data)
    }
}

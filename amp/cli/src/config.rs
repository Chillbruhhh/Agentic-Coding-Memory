use std::path::PathBuf;
use anyhow::Result;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_url: String,
    pub session_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let server_url = env::var("AMP_SERVER_URL")
            .unwrap_or_else(|_| "http://localhost:8105".to_string());
        
        let data_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("amp");
        
        let session_dir = data_dir.join("sessions");
        
        // Ensure directories exist
        std::fs::create_dir_all(&session_dir)?;
        
        Ok(Self {
            server_url,
            session_dir,
            data_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        let config = Config::from_env().unwrap();
        assert!(!config.server_url.is_empty());
        assert!(config.session_dir.exists());
    }
}

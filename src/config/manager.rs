use crate::Result;
use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Dropbox API access token
    pub dropbox_token: String,
    /// Local sync folder path (default: ~/Dropbox)
    pub sync_folder: PathBuf,
    /// Polling interval in seconds for large folders (default: 300)
    pub polling_interval: u64,
    /// Maximum file size for inotify monitoring in bytes (default: 20,000 files)
    pub max_files_for_inotify: usize,
    /// Large file threshold in bytes (default: 100MB)
    pub large_file_threshold: u64,
    /// Log level (default: info)
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        
        Self {
            dropbox_token: String::new(),
            sync_folder: home.join("Dropbox"),
            polling_interval: 300, // 5 minutes
            max_files_for_inotify: 20_000,
            large_file_threshold: 100 * 1024 * 1024, // 100MB
            log_level: "info".to_string(),
        }
    }
}

/// Configuration manager for the application
pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Load configuration from file or create default
    pub fn load() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("dropbox-sync-daemon");
        
        let config_path = config_dir.join("config.json");
        
        // Create config directory if it doesn't exist
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {}", e))?;
        
        let config = if config_path.exists() {
            info!("Loading configuration from {}", config_path.display());
            Self::load_from_file(&config_path)?
        } else {
            info!("Creating default configuration at {}", config_path.display());
            let default_config = AppConfig::default();
            Self::save_to_file(&config_path, &default_config)?;
            default_config
        };
        
        debug!("Configuration loaded: sync_folder={}, polling_interval={}s", 
               config.sync_folder.display(), config.polling_interval);
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    /// Load configuration from a specific file
    fn load_from_file(path: &PathBuf) -> Result<AppConfig> {
        let config = Config::builder()
            .add_source(File::from(path.as_path()))
            .add_source(Environment::with_prefix("DROPBOX_SYNC"))
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build config: {}", e))?;
        
        config.try_deserialize()
            .map_err(|e| anyhow::anyhow!("Failed to deserialize config: {}", e))
    }
    
    /// Save configuration to file
    fn save_to_file(path: &PathBuf, config: &AppConfig) -> Result<()> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))?;
        
        std::fs::write(path, json)
            .map_err(|e| anyhow::anyhow!("Failed to write config file: {}", e))?;
        
        debug!("Configuration saved to {}", path.display());
        Ok(())
    }
    
    /// Save current configuration
    pub fn save(&self) -> Result<()> {
        Self::save_to_file(&self.config_path, &self.config)
    }
    
    /// Get a reference to the configuration
    pub fn config(&self) -> &AppConfig {
        &self.config
    }
    
    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }
    
    /// Update the Dropbox token
    pub fn set_dropbox_token(&mut self, token: String) -> Result<()> {
        self.config.dropbox_token = token;
        self.save()
    }
    
    /// Update the sync folder path
    pub fn set_sync_folder(&mut self, path: PathBuf) -> Result<()> {
        self.config.sync_folder = path;
        self.save()
    }
    
    /// Get the config directory path
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("dropbox-sync-daemon"))
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))
    }
    
    /// Get the data directory path for logs
    pub fn data_dir() -> Result<PathBuf> {
        dirs::data_local_dir()
            .map(|p| p.join("dropbox-sync-daemon"))
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))
    }
}

impl std::ops::Deref for ConfigManager {
    type Target = AppConfig;
    
    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl std::ops::DerefMut for ConfigManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.polling_interval, 300);
        assert_eq!(config.max_files_for_inotify, 20_000);
        assert_eq!(config.large_file_threshold, 100 * 1024 * 1024);
        assert_eq!(config.log_level, "info");
    }
    
    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(config.polling_interval, deserialized.polling_interval);
        assert_eq!(config.max_files_for_inotify, deserialized.max_files_for_inotify);
        assert_eq!(config.large_file_threshold, deserialized.large_file_threshold);
        assert_eq!(config.log_level, deserialized.log_level);
    }
} 
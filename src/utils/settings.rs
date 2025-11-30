use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub openrouter_api_key: Option<String>,
    pub theme: String,
    pub theme_mode: ThemeMode, // dark or light
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Dark,
    Light,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            openrouter_api_key: None,
            theme: "dracula".to_string(),
            theme_mode: ThemeMode::Dark,
        }
    }
}

impl Settings {
    /// Get the platform-specific settings directory
    pub fn settings_dir() -> Result<PathBuf, String> {
        let config_dir = if cfg!(target_os = "windows") {
            // Windows: %APPDATA%\gtllm
            dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("gtllm")
        } else if cfg!(target_os = "macos") {
            // macOS: ~/Library/Application Support/gtllm
            dirs::config_dir()
                .ok_or("Could not find config directory")?
                .join("gtllm")
        } else {
            // Linux/Unix: $HOME/.gtllm
            dirs::home_dir()
                .ok_or("Could not find home directory")?
                .join(".gtllm")
        };

        Ok(config_dir)
    }

    /// Get the full path to the settings file
    pub fn settings_path() -> Result<PathBuf, String> {
        Ok(Self::settings_dir()?.join("settings.toml"))
    }

    /// Load settings from the config file
    pub fn load() -> Result<Self, String> {
        let path = Self::settings_path()?;

        if !path.exists() {
            // Return default settings if file doesn't exist
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: Settings = toml::from_str(&contents)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        Ok(settings)
    }

    /// Save settings to the config file
    pub fn save(&self) -> Result<(), String> {
        let dir = Self::settings_dir()?;

        // Create directory if it doesn't exist
        if !dir.exists() {
            fs::create_dir_all(&dir)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }

        let path = Self::settings_path()?;
        let contents = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, contents)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        // Set proper permissions on Unix-like systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?
                .permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&path, perms)
                .map_err(|e| format!("Failed to set file permissions: {}", e))?;
        }

        Ok(())
    }

    /// Check if API key is configured
    pub fn has_api_key(&self) -> bool {
        self.openrouter_api_key.is_some()
    }

    /// Get API key
    pub fn get_api_key(&self) -> Option<&str> {
        self.openrouter_api_key.as_deref()
    }

    /// Set API key
    pub fn set_api_key(&mut self, api_key: String) {
        self.openrouter_api_key = Some(api_key);
    }

    /// Clear API key
    pub fn clear_api_key(&mut self) {
        self.openrouter_api_key = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.openrouter_api_key, None);
        assert_eq!(settings.theme, "dracula");
        assert_eq!(settings.theme_mode, ThemeMode::Dark);
    }

    #[test]
    fn test_has_api_key() {
        let mut settings = Settings::default();
        assert!(!settings.has_api_key());

        settings.set_api_key("test-key".to_string());
        assert!(settings.has_api_key());

        settings.clear_api_key();
        assert!(!settings.has_api_key());
    }
}

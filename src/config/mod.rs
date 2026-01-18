use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub name: String,
    pub email: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user: User {
                name: String::from("Unknown"),
                email: String::from("unknown@example.com"),
            },
        }
    }
}

impl Config {
    /// Read config from .kitcat/config
    pub fn read() -> io::Result<Self> {
        let config_path = Path::new(".kitcat/config");

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path)?;
        toml::from_str(&content).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse config: {}", e),
            )
        })
    }

    /// Write config to .kitcat/config
    pub fn write(&self) -> io::Result<()> {
        let config_path = Path::new(".kitcat/config");
        let content = toml::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {}", e),
            )
        })?;

        fs::write(config_path, content)?;
        Ok(())
    }

    /// Get user name and email in format "Name <email>"
    pub fn get_user_string(&self) -> String {
        format!("{} <{}>", self.user.name, self.user.email)
    }
}

/// Set a config value
pub fn set_config(key: &str, value: &str) -> io::Result<()> {
    let mut config = Config::read()?;

    match key {
        "user.name" => config.user.name = value.to_string(),
        "user.email" => config.user.email = value.to_string(),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown config key: {}", key),
            ));
        }
    }

    config.write()?;
    println!("Set {} = {}", key, value);
    Ok(())
}

/// Get a config value
pub fn get_config(key: &str) -> io::Result<String> {
    let config = Config::read()?;

    let value = match key {
        "user.name" => &config.user.name,
        "user.email" => &config.user.email,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Unknown config key: {}", key),
            ));
        }
    };

    Ok(value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.user.name, "Unknown");
        assert_eq!(config.user.email, "unknown@example.com");
    }

    #[test]
    fn test_user_string() {
        let config = Config {
            user: User {
                name: "John Doe".to_string(),
                email: "john@example.com".to_string(),
            },
        };
        assert_eq!(config.get_user_string(), "John Doe <john@example.com>");
    }
}

use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::exit,
};

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::storage::Storage;

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigToml {
    pub ui: Ui,
    pub statuses: Vec<StatusConfig>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Ui {
    pub show_help: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StatusConfig {
    pub label: String,
    pub color: String,
    pub terminal: bool,
    pub origin: Option<bool>,
}

impl StatusConfig {
    pub fn to_color(&self) -> Color {
        match self.color.to_lowercase().as_str() {
            "red" => Color::Red,
            "green" => Color::Green,
            "blue" => Color::Blue,
            "yellow" => Color::Yellow,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            "black" => Color::Black,
            "light_red" => Color::LightRed,
            "light_green" => Color::LightGreen,
            "light_blue" => Color::LightBlue,
            "light_yellow" => Color::LightYellow,
            "light_magenta" => Color::LightMagenta,
            "light_cyan" => Color::LightCyan,
            "gray" => Color::Gray,
            "dark_gray" => Color::DarkGray,
            _ => Color::Gray,
        }
    }
}

pub struct Config;

static CONFIG_FILE_NAME: &str = "config";

impl Config {
    pub fn get_default() -> ConfigToml {
        ConfigToml {
            ui: Ui { show_help: true },
            statuses: vec![
                StatusConfig {
                    label: "UpNext".to_string(),
                    color: "light_magenta".to_string(),
                    terminal: false,
                    origin: Some(true),
                },
                StatusConfig {
                    label: "OnGoing".to_string(),
                    color: "yellow".to_string(),
                    terminal: false,
                    origin: None,
                },
                StatusConfig {
                    label: "Done".to_string(),
                    color: "light_green".to_string(),
                    terminal: true,
                    origin: None,
                },
            ],
        }
    }

    fn get_config_path() -> PathBuf {
        let mut path = PathBuf::new();
        path.push(Storage::get_data_dir().as_path());
        path.push(format!("{CONFIG_FILE_NAME}.toml"));

        return path;
    }

    pub fn read() -> ConfigToml {
        let path = Config::get_config_path();
        let config_raw = match fs::read_to_string(&path) {
            Ok(c) => c,
            // If config.toml file doesn't exist, create it by default
            Err(_) => {
                let default_config = toml::to_string(&Config::get_default()).unwrap();

                let mut file = File::create(&path).unwrap();
                let _ = file.write_all(default_config.as_bytes());

                default_config
            }
        };

        let data: ConfigToml = match toml::from_str(&config_raw) {
            Ok(c) => c,
            // If config.toml is not valid, throw a error message
            Err(_) => {
                eprint!(
                    "{} - ERROR: The configuration file is invalid. Please check the wiki for correct formatting or delete the file",
                    env!("CARGO_PKG_NAME")
                );
                exit(1)
            }
        };

        // Validate the configuration
        Self::validate(&data);

        return data;
    }

    fn validate(config: &ConfigToml) {
        // Count states with origin = true
        let origin_count = config
            .statuses
            .iter()
            .filter(|s| s.origin == Some(true))
            .count();

        // Check that only one state has origin = true
        if origin_count > 1 {
            eprintln!(
                "{} - ERROR: Multiple states have 'origin = true'. Only one state can be the origin state.",
                env!("CARGO_PKG_NAME")
            );
            exit(1);
        }

        // If custom fields (non-default statuses) are configured, an origin must be present
        let has_custom_statuses = !config.statuses.is_empty()
            && !(config.statuses.len() == 3
                && config.statuses.iter().any(|s| s.label == "UpNext")
                && config.statuses.iter().any(|s| s.label == "OnGoing")
                && config.statuses.iter().any(|s| s.label == "Done"));

        if has_custom_statuses && origin_count == 0 {
            eprintln!(
                "{} - ERROR: Custom statuses are configured but no origin state is defined. Please set 'origin = true' on one status.",
                env!("CARGO_PKG_NAME")
            );
            exit(1);
        }
    }

    pub fn get_origin_status(config: &ConfigToml) -> String {
        config
            .statuses
            .iter()
            .find(|s| s.origin == Some(true))
            .map(|s| s.label.clone())
            .unwrap_or_else(|| "UpNext".to_string())
    }
}

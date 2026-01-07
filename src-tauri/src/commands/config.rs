use log::info;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tauri::State;

use crate::commands::PacketProcessingState;
use crate::settings::Settings;

/// Filter target mode for targeting specific processes or devices
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FilterTargetMode {
    #[default]
    All,
    Process,
    Device,
    Custom,
}

/// Filter target configuration for saving/loading
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterTarget {
    pub mode: FilterTargetMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_ip: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_filter: Option<String>,
    /// Include inbound traffic (default: true)
    #[serde(default = "default_true")]
    pub include_inbound: bool,
    /// Include outbound traffic (default: true)
    #[serde(default = "default_true")]
    pub include_outbound: bool,
}

fn default_true() -> bool {
    true
}

/// Hotkey binding configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotkeyBinding {
    pub action: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
}

/// Tap feature settings (frontend-only, stored in config for persistence)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TapSettings {
    /// Whether tap is enabled (should always default to false)
    #[serde(default)]
    pub enabled: bool,
    /// How often to tap in milliseconds
    #[serde(default = "default_tap_interval")]
    pub interval_ms: u64,
    /// How long to keep modules off in milliseconds
    #[serde(default = "default_tap_duration")]
    pub duration_ms: u64,
}

fn default_tap_interval() -> u64 {
    3000
}

fn default_tap_duration() -> u64 {
    600
}

/// Configuration file structure for storing application settings
///
/// Contains both the packet manipulation settings and the active filter string.
/// Used for serialization/deserialization when saving and loading configurations.
#[derive(Serialize, Deserialize)]
struct ConfigFile {
    /// Packet manipulation settings
    settings: Settings,
    /// `WinDivert` filter string
    filter: Option<String>,
    /// Filter target configuration (process, device, etc.)
    #[serde(default)]
    filter_target: Option<FilterTarget>,
    /// Hotkey bindings
    #[serde(default)]
    hotkeys: Option<Vec<HotkeyBinding>>,
    /// Tap feature settings
    #[serde(default)]
    tap: Option<TapSettings>,
}

/// Saves the current configuration to a named file
///
/// # Arguments
///
/// * `state` - The application state containing settings to save
/// * `name` - The name to use for the configuration file
/// * `filter_target` - Optional filter target configuration
///
/// # Returns
///
/// * `Ok(())` - If the configuration was saved successfully
/// * `Err(String)` - If there was an error saving the configuration
#[tauri::command]
pub async fn save_config(
    state: State<'_, PacketProcessingState>,
    name: String,
    filter_target: Option<FilterTarget>,
    hotkeys: Option<Vec<HotkeyBinding>>,
    tap: Option<TapSettings>,
) -> Result<(), String> {
    let settings = state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))?
        .clone();

    let filter = state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))?
        .clone();

    let config_path = get_config_path(&name)?;

    let config = ConfigFile {
        settings,
        filter,
        filter_target,
        hotkeys,
        tap,
    };

    let content = toml::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    let mut file = fs::File::create(&config_path)
        .map_err(|e| format!("Failed to create config file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to config file: {}", e))?;

    info!("Saved configuration to {}", name);

    Ok(())
}

/// Response structure for `load_config` that includes filter target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadConfigResponse {
    pub settings: Settings,
    pub filter: Option<String>,
    pub filter_target: Option<FilterTarget>,
    pub hotkeys: Option<Vec<HotkeyBinding>>,
    pub tap: Option<TapSettings>,
}

/// Loads a named configuration file and updates application state
///
/// # Arguments
///
/// * `state` - The application state to update with loaded settings
/// * `name` - The name of the configuration file to load
///
/// # Returns
///
/// * `Ok(LoadConfigResponse)` - The loaded settings and filter target
/// * `Err(String)` - If there was an error loading the configuration
#[tauri::command]
pub async fn load_config(
    state: State<'_, PacketProcessingState>,
    name: String,
) -> Result<LoadConfigResponse, String> {
    let config_path = get_config_path(&name)?;

    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: ConfigFile =
        toml::from_str(&content).map_err(|e| format!("Failed to deserialize config: {}", e))?;

    *state
        .settings
        .lock()
        .map_err(|e| format!("Failed to lock settings mutex: {}", e))? = config.settings.clone();

    *state
        .filter
        .lock()
        .map_err(|e| format!("Failed to lock filter mutex: {}", e))? = config.filter.clone();

    info!("Loaded configuration from {}", name);

    Ok(LoadConfigResponse {
        settings: config.settings,
        filter: config.filter,
        filter_target: config.filter_target,
        hotkeys: config.hotkeys,
        tap: config.tap,
    })
}

/// Lists all available configuration files
///
/// # Returns
///
/// * `Ok(Vec<String>)` - List of configuration names
/// * `Err(String)` - If there was an error reading the configs directory
#[tauri::command]
pub async fn list_configs() -> Result<Vec<String>, String> {
    let config_dir = get_config_dir()?;

    let mut configs = Vec::new();
    for entry in std::fs::read_dir(config_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().is_some_and(|ext| ext == "toml") {
            if let Some(name) = path.file_stem() {
                if let Some(name) = name.to_str() {
                    configs.push(name.to_string());
                }
            }
        }
    }

    Ok(configs)
}

/// Deletes a named configuration file
///
/// # Arguments
///
/// * `name` - The name of the configuration file to delete
///
/// # Returns
///
/// * `Ok(())` - If the configuration was deleted successfully
/// * `Err(String)` - If there was an error deleting the configuration
#[tauri::command]
pub async fn delete_config(name: String) -> Result<(), String> {
    let config_path = get_config_path(&name)?;

    if !config_path.exists() {
        return Err(format!("Configuration {} does not exist", name));
    }

    std::fs::remove_file(&config_path).map_err(|e| format!("Failed to delete config: {}", e))?;

    info!("Deleted configuration {}", name);

    Ok(())
}

/// Gets the path to the configs directory
///
/// Creates the directory if it doesn't exist.
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the configs directory
/// * `Err(String)` - If there was an error determining or creating the directory
fn get_config_dir() -> Result<PathBuf, String> {
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Could not determine executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Could not determine executable directory".to_string())?
        .to_path_buf();

    let config_dir = exe_dir.join("configs");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    Ok(config_dir)
}

/// Gets the full path to a named configuration file
///
/// # Arguments
///
/// * `name` - The name of the configuration
///
/// # Returns
///
/// * `Ok(PathBuf)` - Path to the configuration file
/// * `Err(String)` - If there was an error determining the path
fn get_config_path(name: &str) -> Result<PathBuf, String> {
    let mut path = get_config_dir()?;
    path.push(format!("{}.toml", name));
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_target_mode_default() {
        let mode: FilterTargetMode = Default::default();
        assert!(matches!(mode, FilterTargetMode::All));
    }

    #[test]
    fn test_filter_target_serde_defaults() {
        // When deserializing with missing fields, serde uses the default functions
        let json = r#"{"mode": "all"}"#;
        let target: FilterTarget = serde_json::from_str(json).unwrap();
        assert!(matches!(target.mode, FilterTargetMode::All));
        assert!(target.include_inbound);
        assert!(target.include_outbound);
        assert!(target.process_id.is_none());
        assert!(target.process_name.is_none());
    }

    #[test]
    fn test_filter_target_serialization() {
        let target = FilterTarget {
            mode: FilterTargetMode::Process,
            process_id: Some(1234),
            process_name: Some("test.exe".to_string()),
            device_ip: None,
            device_name: None,
            custom_filter: None,
            include_inbound: true,
            include_outbound: false,
        };

        let json = serde_json::to_string(&target).unwrap();
        let parsed: FilterTarget = serde_json::from_str(&json).unwrap();

        assert!(matches!(parsed.mode, FilterTargetMode::Process));
        assert_eq!(parsed.process_id, Some(1234));
        assert_eq!(parsed.process_name, Some("test.exe".to_string()));
        assert!(parsed.include_inbound);
        assert!(!parsed.include_outbound);
    }

    #[test]
    fn test_filter_target_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&FilterTargetMode::All).unwrap(),
            "\"all\""
        );
        assert_eq!(
            serde_json::to_string(&FilterTargetMode::Process).unwrap(),
            "\"process\""
        );
        assert_eq!(
            serde_json::to_string(&FilterTargetMode::Device).unwrap(),
            "\"device\""
        );
        assert_eq!(
            serde_json::to_string(&FilterTargetMode::Custom).unwrap(),
            "\"custom\""
        );
    }

    #[test]
    fn test_filter_target_mode_deserialization() {
        let all: FilterTargetMode = serde_json::from_str("\"all\"").unwrap();
        let process: FilterTargetMode = serde_json::from_str("\"process\"").unwrap();
        let device: FilterTargetMode = serde_json::from_str("\"device\"").unwrap();
        let custom: FilterTargetMode = serde_json::from_str("\"custom\"").unwrap();

        assert!(matches!(all, FilterTargetMode::All));
        assert!(matches!(process, FilterTargetMode::Process));
        assert!(matches!(device, FilterTargetMode::Device));
        assert!(matches!(custom, FilterTargetMode::Custom));
    }

    #[test]
    fn test_hotkey_binding_default() {
        let binding: HotkeyBinding = Default::default();
        assert!(binding.action.is_empty());
        assert!(binding.shortcut.is_none());
        assert!(!binding.enabled);
    }

    #[test]
    fn test_hotkey_binding_serialization() {
        let binding = HotkeyBinding {
            action: "toggleFilter".to_string(),
            shortcut: Some("F9".to_string()),
            enabled: true,
        };

        let json = serde_json::to_string(&binding).unwrap();
        let parsed: HotkeyBinding = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.action, "toggleFilter");
        assert_eq!(parsed.shortcut, Some("F9".to_string()));
        assert!(parsed.enabled);
    }

    #[test]
    fn test_tap_settings_serde_defaults() {
        // When deserializing with missing fields, serde uses the default functions
        let json = r#"{}"#;
        let tap: TapSettings = serde_json::from_str(json).unwrap();
        assert!(!tap.enabled);
        assert_eq!(tap.interval_ms, 3000);
        assert_eq!(tap.duration_ms, 600);
    }

    #[test]
    fn test_tap_settings_serialization() {
        let tap = TapSettings {
            enabled: true,
            interval_ms: 5000,
            duration_ms: 1000,
        };

        let json = serde_json::to_string(&tap).unwrap();
        let parsed: TapSettings = serde_json::from_str(&json).unwrap();

        assert!(parsed.enabled);
        assert_eq!(parsed.interval_ms, 5000);
        assert_eq!(parsed.duration_ms, 1000);
    }

    #[test]
    fn test_load_config_response_serialization() {
        let response = LoadConfigResponse {
            settings: Settings::default(),
            filter: Some("outbound".to_string()),
            filter_target: Some(FilterTarget::default()),
            hotkeys: Some(vec![HotkeyBinding {
                action: "toggleFilter".to_string(),
                shortcut: Some("F9".to_string()),
                enabled: true,
            }]),
            tap: Some(TapSettings::default()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let parsed: LoadConfigResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.filter, Some("outbound".to_string()));
        assert!(parsed.filter_target.is_some());
        assert!(parsed.hotkeys.is_some());
        assert_eq!(parsed.hotkeys.unwrap().len(), 1);
    }
}

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const MAX_HISTORY: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FilterHistory {
    entries: Vec<String>,
}

fn get_history_dir() -> Result<PathBuf, String> {
    // Prefer roaming AppData on Windows
    if let Ok(appdata) = std::env::var("APPDATA") {
        let dir = PathBuf::from(appdata).join("Myra");
        if !dir.exists() {
            fs::create_dir_all(&dir).map_err(|e| format!("Failed creating history dir: {}", e))?;
        }
        return Ok(dir);
    }
    // Fallback: alongside executable
    let exe_dir = std::env::current_exe()
        .map_err(|e| format!("Could not determine executable path: {}", e))?
        .parent()
        .ok_or_else(|| "Could not determine executable directory".to_string())?
        .to_path_buf();
    let dir = exe_dir.join("user-data");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| format!("Failed creating fallback dir: {}", e))?;
    }
    Ok(dir)
}

fn get_history_path() -> Result<PathBuf, String> {
    Ok(get_history_dir()?.join("filters.json"))
}

fn load_history() -> Result<FilterHistory, String> {
    let path = get_history_path()?;
    if !path.exists() {
        return Ok(FilterHistory::default());
    }
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Failed reading filter history: {}", e))?;
    let parsed: FilterHistory = serde_json::from_str(&content)
        .map_err(|e| format!("Failed parsing filter history: {}", e))?;
    Ok(parsed)
}

fn save_history(history: &FilterHistory) -> Result<(), String> {
    let path = get_history_path()?;
    let json = serde_json::to_string_pretty(history)
        .map_err(|e| format!("Failed serializing filter history: {}", e))?;
    let mut file = fs::File::create(&path)
        .map_err(|e| format!("Failed creating filter history file: {}", e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| format!("Failed writing filter history: {}", e))?
        ;
    Ok(())
}

pub fn add_to_history(filter: &str) -> Result<(), String> {
    if filter.trim().is_empty() { return Ok(()); }
    let mut history = load_history()?;
    // If same as most recent, do nothing
    if history.entries.first().map(|f| f == filter).unwrap_or(false) {
        return Ok(());
    }
    // Move existing to front, else insert at front
    history.entries.retain(|f| f != filter);
    history.entries.insert(0, filter.to_string());
    // Cap size
    if history.entries.len() > MAX_HISTORY {
        history.entries.truncate(MAX_HISTORY);
    }
    save_history(&history)
}

#[tauri::command]
pub async fn get_filter_history() -> Result<Vec<String>, String> {
    Ok(load_history()?.entries)
}

#[tauri::command]
pub async fn clear_filter_history() -> Result<(), String> {
    save_history(&FilterHistory::default())
}

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all)]

// main entry point
use log::{error, info, LevelFilter, SetLoggerError};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use winapi::um::securitybaseapi::FreeSid;

mod commands;
mod error;
mod network;
mod settings;
mod utils;

// Simple console logger implementation
struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut stdout = io::stdout();
            let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
            writeln!(
                stdout,
                "[{}] {} - {}: {}",
                timestamp,
                record.level(),
                record.target(),
                record.args()
            )
            .unwrap_or_else(|e| error!("Failed to write log: {}", e));
            stdout
                .flush()
                .unwrap_or_else(|e| error!("Failed to flush stdout: {}", e));
        }
    }

    fn flush(&self) {
        io::stdout()
            .flush()
            .unwrap_or_else(|e| error!("Failed to flush stdout: {}", e));
    }
}

static LOGGER: SimpleLogger = SimpleLogger;

/// Initialize the application logger
///
/// Sets up the SimpleLogger with Info level filtering
fn init_logger() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

/// Main entry point for the Myra application
fn main() {
    if let Err(e) = init_logger() {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }

    info!("Myra starting up");

    if !is_admin() {
        error!("Myra requires administrator privileges to capture network packets. Please run as administrator.");
        return;
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            commands::register_commands(app)?;
            info!("Packet manipulation system initialized");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::start_processing,
            commands::settings::stop_processing,
            commands::settings::get_status,
            commands::settings::update_settings,
            commands::settings::get_settings,
            commands::settings::update_filter,
            commands::settings::get_filter,
            commands::config::save_config,
            commands::config::load_config,
            commands::config::list_configs,
            commands::config::delete_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Verify that WinDivert files are present in the expected locations
///
/// Logs the presence or absence of critical WinDivert files
fn check_windivert_files() {
    let current_exe = env::current_exe().unwrap_or_else(|e| {
        error!("Failed to get current executable path: {}", e);
        PathBuf::new()
    });

    let exe_dir = current_exe.parent().unwrap_or_else(|| {
        error!("Failed to get parent directory of executable");
        std::path::Path::new(".")
    });

    let dll_path = exe_dir.join("WinDivert.dll");
    let sys_path = exe_dir.join("WinDivert64.sys");

    info!("Looking for WinDivert.dll at: {:?}", dll_path);
    info!("Looking for WinDivert64.sys at: {:?}", sys_path);
    info!("WinDivert.dll exists: {}", dll_path.exists());
    info!("WinDivert64.sys exists: {}", sys_path.exists());
}

/// Check if the current process is running with administrator privileges
///
/// Uses Windows API to determine if the current process has admin rights,
/// which are required for packet manipulation.
///
/// # Returns
///
/// `bool` - true if the process has administrator privileges
fn is_admin() -> bool {
    use winapi::um::securitybaseapi::AllocateAndInitializeSid;
    use winapi::um::securitybaseapi::CheckTokenMembership;
    use winapi::um::winnt::{
        DOMAIN_ALIAS_RID_ADMINS, SECURITY_BUILTIN_DOMAIN_RID, SECURITY_NT_AUTHORITY,
    };

    unsafe {
        let mut sid = std::ptr::null_mut();
        let sub_authorities = [SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS];

        if AllocateAndInitializeSid(
            &SECURITY_NT_AUTHORITY as *const _ as *mut _,
            2,
            sub_authorities[0],
            sub_authorities[1],
            0,
            0,
            0,
            0,
            0,
            0,
            &mut sid,
        ) == 0
        {
            return false;
        }

        let mut is_member = 0;
        let is_admin =
            CheckTokenMembership(std::ptr::null_mut(), sid, &mut is_member) != 0 && is_member != 0;

        FreeSid(sid);
        is_admin
    }
}

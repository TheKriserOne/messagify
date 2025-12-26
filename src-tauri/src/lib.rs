use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod messangers;

pub struct AppState {
    pub token: Mutex<Option<String>>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("messagify=debug,info")),
        )
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting Messagify application");

    tauri::Builder::default()
        .manage(AppState {
            token: Mutex::new(None),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(callbacks!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

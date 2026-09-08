use airplay_client::{AirPlayClient, Device};
use std::time::Duration;
mod backgroundservice;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn client_setup(timeout: u64) -> Result<Vec<Device>,airplay_client::Error> {
    let client = AirPlayClient::new()?;
    let devices = client.discover(Duration::from_secs(timeout)).await?;
    return Ok(devices);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_background_service::init_with_service(
            || backgroundservice::AirplayService::new(),
        ))
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use tauri::{Builder, Manager, State};
use tauri::async_runtime::Mutex;

mod backgroundservice;
mod airplayclient;

use backgroundservice::AirplayService;
use airplayclient::{DeviceManager,ResultTypes};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn get_client_list(device_manager: State<'_, Mutex<DeviceManager>>,timeout: u64) -> Result<ResultTypes,ResultTypes> {
    let mut device_manager_state = device_manager.lock().await;
    device_manager_state.discover_devices(timeout).await?;
    Ok(device_manager_state.get_device_entries().await)
}

#[tauri::command]
async fn connect_to_client_index(device_manager: State<'_, Mutex<DeviceManager>>,index: usize) -> Result<ResultTypes,ResultTypes> {
    let mut device_manager_state = device_manager.lock().await;
    device_manager_state.connect_to_device(index).await?;
    Ok(device_manager_state.get_device_entries().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
//        .plugin(tauri_plugin_opener::init())
//        .plugin(tauri_plugin_notification::init())
//        .plugin(tauri_plugin_background_service::init_with_service(
//            || AirplayService::new(),
//        ))
        .setup(|app: &mut tauri::App| {
            app.manage(Mutex::new(DeviceManager::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_client_list,connect_to_client_index])
        .run(tauri::generate_context!())
        .expect("error while fetching Airplay clients");
}

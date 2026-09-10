use serde::Deserialize;
use tauri::{AppHandle, Builder, Manager, State};
use tauri_plugin_store::StoreExt;
use tauri::async_runtime::Mutex;
use serde_json;

mod backgroundservice;
mod airplayclient;

use backgroundservice::AirplayService;
use airplayclient::{DeviceManager,PersistentIdentity,ResultTypes};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
async fn get_client_list(device_manager: State<'_, Mutex<DeviceManager>>,timeout: u64) -> Result<ResultTypes,ResultTypes> {
    let mut device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    device_manager_state.discover_devices(timeout).await?;
    device_manager_state.get_device_entries()
}

#[tauri::command]
async fn connect_to_client_index(device_manager: State<'_, Mutex<DeviceManager>>,index: usize) -> Result<ResultTypes,ResultTypes> {
    let mut device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    device_manager_state.connect_to_device_by_index(index).await?;
    device_manager_state.get_connected_device()
}

#[tauri::command]
async fn connect_to_client_index_persisted_id(app: AppHandle, device_manager: State<'_, Mutex<DeviceManager>>,index: usize) -> Result<ResultTypes,ResultTypes> {
    let Ok(store) = app.store("persisted_identities.json") else {
        return Err(airplayclient::string_to_client_result("Could not load store"))
    };
    let mut device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    let key: &str = device_manager_state.get_connected_device_identity_key()?;
    let Some(persisted_id) = store.get(key) else {
        return Err(airplayclient::string_to_client_result("Could not find persisted ID in store"))
    };
    let Ok(deser_persisted_id) = &PersistentIdentity::deserialize(persisted_id) else {
        return Err(airplayclient::string_to_client_result("Could not deserialize persisted ID"))
    };
    device_manager_state.connect_to_device_by_index_with_persisted_id(index,deser_persisted_id).await?;
    device_manager_state.get_connected_device()
}

#[tauri::command]
async fn connect_to_client_index_pin(app: AppHandle, device_manager: State<'_, Mutex<DeviceManager>>,index: usize,pin: &str) -> Result<ResultTypes,ResultTypes> {
    let mut device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    device_manager_state.connect_to_device_by_index_with_pin(index,pin).await?;
    device_manager_state.get_connected_device()
}

#[tauri::command]
async fn connect_to_client_index_pin_pairing(app: AppHandle, device_manager: State<'_, Mutex<DeviceManager>>,index: usize,pin: &str) -> Result<ResultTypes,ResultTypes> {
    let Ok(store) = app.store("persisted_identities.json") else {
        return Err(airplayclient::string_to_client_result("Could not load store"))
    };
    let mut device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    let persisted_identity = device_manager_state.connect_to_device_by_index_with_pin_pairing(index,pin).await?;// -- need to write logic to distinguish these cases
    store.set(device_manager_state.get_connected_device_identity_key()?, serde_json::json!(persisted_identity));
    device_manager_state.get_connected_device()
}

#[tauri::command]
async fn send_client_index_pin_prompt(device_manager: State<'_, Mutex<DeviceManager>>,index: usize) -> Result<ResultTypes,ResultTypes> {
    let device_manager_state: tokio::sync::MutexGuard<DeviceManager> = device_manager.lock().await;
    device_manager_state.send_pin_prompt_by_index(index).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
//        .plugin(tauri_plugin_notification::init())
//        .plugin(tauri_plugin_background_service::init_with_service(
//            || AirplayService::new(),
//        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app: &mut tauri::App| {
            app.manage(Mutex::new(DeviceManager::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_client_list,
            connect_to_client_index,
            connect_to_client_index_persisted_id,
            connect_to_client_index_pin,
            connect_to_client_index_pin_pairing,
            send_client_index_pin_prompt
        ])
        .run(tauri::generate_context!())
        .expect("error while fetching Airplay clients");
}

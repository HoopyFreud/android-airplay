use std::time::Duration;
use tauri_plugin_http::reqwest;

use serde::Serialize;
use strum_macros::AsRefStr;
use airplay_client::{AirPlayClient, Device};

pub use airplay_client::PersistentIdentity;

#[derive(Default)]
pub struct DeviceManager {
    airplay_client: AirPlayClient,
    device_list: Vec<Device>,
    connected_device_index: Option<usize>,
}

impl DeviceManager {
    //functions that compute
    pub async fn discover_devices(&mut self,timeout: u64) -> Result<(),ResultTypes> {
        self.device_list = self.airplay_client.discover(Duration::from_secs(timeout)).await.map_err(airplay_err_to_client_result)?;
        Ok(())
    }
    pub async fn connect_to_device_by_index(&mut self,index:usize) -> Result<(),ResultTypes> {
        self.airplay_client.connect(&self.device_list[index]).await.map_err(airplay_err_to_client_result)?;
        self.connected_device_index = Some(index);
        Ok(())
    }
    pub async fn connect_to_device_by_index_with_persisted_id(&mut self,index:usize,persisted_id: &PersistentIdentity) -> Result<(),ResultTypes> {
        self.airplay_client.connect_with_persistent_identity(&self.device_list[index],persisted_id).await.map_err(airplay_err_to_client_result)?;
        self.connected_device_index = Some(index);
        Ok(())
    }
    pub async fn connect_to_device_by_index_with_pin(&mut self,index:usize,pin:&str) -> Result<(),ResultTypes> {
        self.airplay_client.connect_with_pin(&self.device_list[index],pin).await.map_err(airplay_err_to_client_result)?;
        self.connected_device_index = Some(index);
        Ok(())
    }
    pub async fn connect_to_device_by_index_with_pin_pairing(&mut self,index:usize,pin:&str) -> Result<PersistentIdentity,ResultTypes> {
        let persisted_identity = self.airplay_client.connect_with_pin_pairing(&self.device_list[index],pin).await.map_err(airplay_err_to_client_result)?;
        self.connected_device_index = Some(index);
        Ok(persisted_identity)
    }
    pub async fn send_pin_prompt_by_index(&self,index:usize) -> Result<ResultTypes,ResultTypes> {
        let Some(address) = self.device_list[index].socket_addr() else {
            return Err(ResultTypes::Error("Could not find device address".to_string()))
        };
        let client: reqwest::Client = reqwest::Client::new();
        let response: reqwest::Response = client.post(format!("http://{}/pair-pin-start",address)).send().await.map_err(http_err_to_client_result)?;
        let response_data: String = response.text().await.map_err(http_err_to_client_result)?;
        Ok(ResultTypes::HttpResponse(response_data))
    }

    pub fn get_connected_device_identity_key(&self) -> Result<&str,ResultTypes> {
        let Some(device_index) = self.connected_device_index else {
            return Err(ResultTypes::Error("Device connection dropped".to_string()))
        };
        let Some(pairing_identity) = self.device_list[device_index].pairing_identity.as_deref() else {
            return Err(ResultTypes::Error("Device has no pairing identity".to_string()))
        };
        Ok(pairing_identity)
    }

    //functions that return values to requests
    pub fn get_device_entries(&self) -> Result<ResultTypes,ResultTypes> {
        if self.device_list.is_empty() {
            return Err(ResultTypes::Error("No devices found".to_string()))
        }
        let res: ResultTypes = ResultTypes::DeviceList(self.device_list.iter().enumerate().map(device_to_device_id_entry).collect());
        Ok(res)
    }
    pub fn get_connected_device(&self) -> Result<ResultTypes,ResultTypes> {
        let Some(index) = self.connected_device_index else {
            return Err(ResultTypes::Error("No connected device".to_string()))
        };
        let res: ResultTypes = ResultTypes::Device(device_to_device_id_entry((index,&self.device_list[index])));
        Ok(res)
    }
}

#[derive(Serialize)]
pub struct DeviceIDEntry {
    device_index: usize,
    device_name: String,
    device_id: Option<String>,
    device_requires_pin: bool
}

impl DeviceIDEntry {
    fn new(device_index: usize,device_name: String,device_id: Option<String>,device_requires_pin: bool) -> Self {
        Self {
            device_index: device_index,
            device_name: device_name,
            device_id: device_id,
            device_requires_pin: device_requires_pin
        }
    }
}

#[derive(AsRefStr,Serialize)]
pub enum ResultTypes {
    Error(String),
    DeviceList(Vec<DeviceIDEntry>),
    Device(DeviceIDEntry),
    HttpResponse(String)
}

fn device_to_device_id_entry((device_index,device): (usize, &Device)) -> DeviceIDEntry {
    //for some reason this is completely wrong in the docs, the check should be for 0b100 because requires_pin is at bit 3
    let device_requires_pin = device.status_flags & 0b100 != 0;
    DeviceIDEntry::new(device_index,device.name.clone(),device.pairing_identity.clone(),device_requires_pin)
}

fn airplay_err_to_client_result(err: airplay_client::Error) -> ResultTypes {
    ResultTypes::Error(err.to_string())
}

fn http_err_to_client_result(err: tauri_plugin_http::reqwest::Error) -> ResultTypes {
    ResultTypes::Error(err.to_string())
}

pub fn string_to_client_result(str: &str) -> ResultTypes {
    ResultTypes::Error(str.to_string())
}
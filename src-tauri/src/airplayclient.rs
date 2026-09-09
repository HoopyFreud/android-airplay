use std::time::Duration;

use serde::Serialize;
use strum_macros::AsRefStr;
use airplay_client::{AirPlayClient, Device};

#[derive(Default)]
pub struct DeviceManager {
    airplay_client: AirPlayClient,
    device_list: Vec<Device>,
    connected_device_index: Option<usize>,
}

impl DeviceManager {
    pub async fn discover_devices(&mut self,timeout: u64) -> Result<(),ResultTypes> {
        self.device_list = self.airplay_client.discover(Duration::from_secs(timeout)).await.map_err(err_to_client_result)?;
        Ok(())
    }
    pub async fn get_device_entries(&self) -> ResultTypes {
        ResultTypes::DeviceList(self.device_list.iter().enumerate().map(device_to_device_id_entry).collect())
    }
    pub async fn connect_to_device(&mut self,index:usize) -> Result<(),ResultTypes> {
        self.airplay_client.connect(&self.device_list[index]).await.map_err(err_to_client_result)?;
        self.connected_device_index = Some(index);
        Ok(())
    }
}

#[derive(Serialize)]
pub struct DeviceIDEntry {
    device_index: usize,
    device_name: String,
    device_id: String,
    device_ip: Option<String>
}

impl DeviceIDEntry {
    fn new(device_index: usize,device_name: String,device_id: String,device_ip: Option<String>) -> Self {
        Self {
            device_index: device_index,
            device_name: device_name,
            device_id: device_id,
            device_ip: device_ip
        }
    }
}

#[derive(AsRefStr,Serialize)]
pub enum ResultTypes {
    Error(String),
    DeviceList(Vec<DeviceIDEntry>),
    Device(DeviceIDEntry),
}

fn device_to_device_id_entry((device_index,device): (usize, &Device)) -> DeviceIDEntry {
    let address = match device.socket_addr() {
        None => None,
        Some(i) => Some(i.to_string())
    };
    DeviceIDEntry::new(device_index,device.name.clone(),device.id.to_mac_string(),address)
}

fn err_to_client_result<'a>(err: airplay_client::Error) -> ResultTypes {
    ResultTypes::Error(err.to_string())
}
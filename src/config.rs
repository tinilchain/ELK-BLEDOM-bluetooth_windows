use serde::{Serialize, Deserialize};
use std::fs;
use std::io;
use crate::bluetooth::BleDevice;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    devices: Vec<BleDevice>,
}

impl Config {
    pub fn load() -> io::Result<Self> {
        if let Ok(content) = fs::read_to_string("config.json") {
            Ok(serde_json::from_str(&content).unwrap_or_default())
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("config.json", content)
    }

    pub fn add_device(&mut self, device: BleDevice) {
        if !self.is_device_bound(&device.address) {
            self.devices.push(device);
        }
    }

    pub fn is_device_bound(&self, address: &str) -> bool {
        self.devices.iter().any(|d| d.address == address)
    }

    pub fn get_devices(&self) -> &[BleDevice] {
        &self.devices
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
} 
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Manager, Peripheral};
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BleDevice {
    pub name: String,
    pub address: String,
    #[serde(skip)]
    pub peripheral: Option<Peripheral>,
}

impl BleDevice {
    pub fn is_led_device(&self) -> bool {
        let name_lower = self.name.to_lowercase();
        name_lower.contains("ble") || name_lower.contains("led") || name_lower.contains("light")
    }
}

pub struct BluetoothManager {
    manager: Manager,
}

impl BluetoothManager {
    pub async fn new() -> Self {
        let manager = Manager::new().await.expect("无法初始化蓝牙管理器");
        Self { manager }
    }

    pub async fn scan_devices(&self) -> Result<Vec<BleDevice>, Box<dyn std::error::Error>> {
        let adapters = self.manager.adapters().await?;
        let central = adapters.into_iter().next().ok_or("未找到蓝牙适配器")?;

        central.start_scan(Default::default()).await?;
        std::thread::sleep(Duration::from_secs(2));

        let peripherals = central.peripherals().await?;
        let mut devices = Vec::new();

        for peripheral in peripherals {
            if let Ok(props) = peripheral.properties().await {
                if let Some(local_name) = props.unwrap().local_name {
                    if Self::is_led_device(&local_name) {
                        devices.push(BleDevice {
                            name: local_name,
                            address: peripheral.address().to_string(),
                            peripheral: Some(peripheral),
                        });
                    }
                }
            }
        }

        Ok(devices)
    }

    fn is_led_device(name: &str) -> bool {
        let name = name.to_lowercase();
        name.contains("ble") || name.contains("led") || name.contains("light")
    }
} 
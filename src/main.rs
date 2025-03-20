use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};
use std::sync::mpsc;
use std::f32::consts::PI;
use serde::{Serialize, Deserialize};
use device::{BleLedDevice, EFFECTS};
use bluetooth::{BluetoothManager, BleDevice};
use audio::AudioMonitor;
use tokio;

mod device;
mod audio;
mod bluetooth;
mod config;

/*
* 蓝牙 LED 控制器
* 用于通过蓝牙连接控制 LED 灯带
*/

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ColorMode {
    White,
    RGB,
    Breathing,
    Rainbow,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::White
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum LightState {
    Basic(ColorMode),
    Audio,
    Test,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DeviceConfig {
    pub devices: Vec<BleDevice>,
    pub default_color_mode: ColorMode,
    pub active_device: Option<String>,
}

impl DeviceConfig {
    fn load() -> Self {
        if let Ok(content) = fs::read_to_string("config.json") {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    fn save(&self) -> io::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write("config.json", content)
    }

    fn is_device_bound(&self, address: &str) -> bool {
        self.devices.iter().any(|d| d.address == address)
    }

    fn add_device(&mut self, device: BleDevice) {
        if !self.is_device_bound(&device.address) {
            self.devices.push(device);
        }
    }

    fn get_devices(&self) -> &[BleDevice] {
        &self.devices
    }
}

pub struct DeviceManager {
    config: DeviceConfig,
    devices: Vec<BleLedDevice>,
    last_audio_time: Instant,
    volume_receiver: mpsc::Receiver<f32>,
    audio_monitor: AudioMonitor,
    state: LightState,
}

impl DeviceManager {
    fn new(audio_monitor: AudioMonitor, volume_receiver: mpsc::Receiver<f32>) -> Self {
        Self {
            config: DeviceConfig::load(),
            devices: Vec::new(),
            last_audio_time: Instant::now(),
            volume_receiver,
            audio_monitor,
            state: LightState::Basic(ColorMode::White),
        }
    }

    fn start_audio_monitoring(&mut self) -> Result<(), Box<dyn Error>> {
        self.audio_monitor.start()
    }

    fn get_volume(&self) -> f32 {
        self.audio_monitor.get_volume()
    }

    async fn scan_devices(&mut self) -> Result<(), Box<dyn Error>> {
        if self.config.devices.is_empty() {
            println!("没有绑定的设备");
            return Ok(());
        }

        println!("扫描已绑定的设备...");
        for device_info in &self.config.devices {
            match BleLedDevice::connect(&device_info.address).await {
                Ok(device) => {
                    println!("成功连接到设备: {}", device_info.name);
                    self.devices.push(device);
                }
                Err(e) => {
                    eprintln!("连接设备失败: {}", e);
                }
            }
        }

        Ok(())
    }

    fn list_devices(&self) {
        println!("\n已绑定的设备:");
        for (i, device) in self.config.devices.iter().enumerate() {
            println!("{}. {} ({})", i + 1, device.name, device.address);
        }
    }

    fn clear_devices(&mut self) {
        self.config.devices.clear();
        if let Err(e) = self.config.save() {
            eprintln!("保存配置失败: {}", e);
        }
    }

    fn handle_color_mode(&mut self, mode: &ColorMode) {
        match mode {
            ColorMode::White => {
                for device in &mut self.devices {
                    device.set_color(255, 255, 255);
                }
            }
            ColorMode::RGB => {
                for device in &mut self.devices {
                    device.set_color(255, 0, 0);
                }
            }
            ColorMode::Breathing => {
                for device in &mut self.devices {
                    device.set_effect(EFFECTS.jump_red_green_blue);
                }
            }
            ColorMode::Rainbow => {
                for device in &mut self.devices {
                    device.set_effect(EFFECTS.jump_red_green_blue_yellow_cyan_magenta_white);
                }
            }
        }
    }

    fn handle_next_mode(&mut self) {
        if let LightState::Basic(mode) = &self.state {
            self.state = LightState::Basic(match *mode {
                ColorMode::White => ColorMode::RGB,
                ColorMode::RGB => ColorMode::Breathing,
                ColorMode::Breathing => ColorMode::Rainbow,
                ColorMode::Rainbow => ColorMode::White,
            });
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            if self.state == LightState::Audio {
                let volume = self.get_volume();
                for device in &mut self.devices {
                    device.set_brightness((volume * 255.0) as u8);
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (audio_monitor, volume_receiver) = AudioMonitor::new();
    let mut device_manager = DeviceManager::new(audio_monitor, volume_receiver);
    
    if let Err(e) = device_manager.start_audio_monitoring() {
        eprintln!("启动音频监控失败: {}", e);
        return Ok(());
    }

    let bluetooth_manager = BluetoothManager::new().await;
    println!("扫描蓝牙设备...");

    let devices = bluetooth_manager.scan_devices().await?;
    if devices.is_empty() {
        println!("未找到设备");
        return Ok(());
    }

    for device in devices {
        if !device_manager.config.is_device_bound(&device.address) {
            println!("发现新设备: {} ({})", device.name, device.address);
            device_manager.config.add_device(device);
            device_manager.config.save()?;
        }
    }

    if let Err(e) = device_manager.scan_devices().await {
        eprintln!("扫描设备失败: {}", e);
        return Ok(());
    }

    println!("运行中... 按 Ctrl+C 停止");
    device_manager.run().await?;

    Ok(())
}